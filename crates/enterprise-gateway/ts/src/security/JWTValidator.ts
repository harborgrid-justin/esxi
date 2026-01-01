/**
 * Enterprise API Gateway - JWT Validator
 *
 * JSON Web Token validation and verification
 */

import jwt from 'jsonwebtoken';
import type { JWTConfig, JWTPayload, GatewayRequest } from '../types';
import { AuthenticationError, AuthorizationError } from '../types';

export class JWTValidator {
  constructor(private config: JWTConfig) {}

  /**
   * Validate JWT from request
   */
  public async validate(
    request: GatewayRequest,
    requiredClaims?: Record<string, any>
  ): Promise<JWTPayload> {
    // Extract JWT from request
    const token = this.extractTokenFromRequest(request);

    if (!token) {
      throw new AuthenticationError('JWT not found in request');
    }

    // Verify JWT
    const payload = await this.verifyToken(token);

    // Check required claims
    if (requiredClaims) {
      this.validateClaims(payload, requiredClaims);
    }

    return payload;
  }

  /**
   * Verify JWT signature and claims
   */
  private async verifyToken(token: string): Promise<JWTPayload> {
    try {
      const secret = this.config.publicKey || this.config.secret;

      if (!secret) {
        throw new AuthenticationError('No secret or public key configured');
      }

      const options: jwt.VerifyOptions = {
        algorithms: [this.config.algorithm],
        issuer: this.config.issuer,
        audience: this.config.audience,
        clockTolerance: this.config.clockTolerance || 0,
      };

      const payload = jwt.verify(token, secret, options) as JWTPayload;

      return payload;
    } catch (error) {
      if (error instanceof jwt.TokenExpiredError) {
        throw new AuthenticationError('JWT has expired');
      } else if (error instanceof jwt.JsonWebTokenError) {
        throw new AuthenticationError(`Invalid JWT: ${error.message}`);
      } else if (error instanceof jwt.NotBeforeError) {
        throw new AuthenticationError('JWT not yet valid');
      }

      throw new AuthenticationError('JWT validation failed');
    }
  }

  /**
   * Validate custom claims
   */
  private validateClaims(payload: JWTPayload, requiredClaims: Record<string, any>): void {
    for (const [claim, expectedValue] of Object.entries(requiredClaims)) {
      const actualValue = payload[claim];

      if (actualValue === undefined) {
        throw new AuthorizationError(`Missing required claim: ${claim}`);
      }

      if (Array.isArray(expectedValue)) {
        if (!Array.isArray(actualValue) || !expectedValue.some((v) => actualValue.includes(v))) {
          throw new AuthorizationError(`Claim ${claim} does not match required values`);
        }
      } else if (actualValue !== expectedValue) {
        throw new AuthorizationError(`Claim ${claim} does not match required value`);
      }
    }
  }

  /**
   * Extract JWT from request
   */
  private extractTokenFromRequest(request: GatewayRequest): string | null {
    // Check Authorization header (Bearer token)
    const authHeader = request.headers['authorization'];
    if (authHeader) {
      const match = authHeader.toString().match(/^Bearer (.+)$/);
      if (match && match[1]) {
        return match[1];
      }
    }

    // Check query parameter
    const tokenQuery = request.query['token'];
    if (tokenQuery) {
      return Array.isArray(tokenQuery) ? tokenQuery[0]! : tokenQuery;
    }

    // Check cookie
    const cookieHeader = request.headers['cookie'];
    if (cookieHeader) {
      const match = cookieHeader.toString().match(/jwt=([^;]+)/);
      if (match && match[1]) {
        return match[1];
      }
    }

    return null;
  }

  /**
   * Sign a new JWT
   */
  public signToken(payload: JWTPayload, expiresIn: string | number = '1h'): string {
    const secret = this.config.secret;

    if (!secret) {
      throw new Error('No secret configured for signing');
    }

    const options: jwt.SignOptions = {
      algorithm: this.config.algorithm as jwt.Algorithm,
      issuer: this.config.issuer,
      audience: this.config.audience,
      expiresIn,
    };

    return jwt.sign(payload, secret, options);
  }

  /**
   * Decode JWT without verification (for inspection)
   */
  public decode(token: string): JWTPayload | null {
    try {
      const decoded = jwt.decode(token);
      return decoded as JWTPayload;
    } catch {
      return null;
    }
  }

  /**
   * Verify token expiration
   */
  public isExpired(token: string): boolean {
    const payload = this.decode(token);

    if (!payload || !payload.exp) {
      return true;
    }

    return Date.now() >= payload.exp * 1000;
  }

  /**
   * Get token expiration time
   */
  public getExpiration(token: string): number | null {
    const payload = this.decode(token);

    if (!payload || !payload.exp) {
      return null;
    }

    return payload.exp * 1000;
  }

  /**
   * Refresh token (re-sign with new expiration)
   */
  public refreshToken(token: string, expiresIn: string | number = '1h'): string {
    const payload = this.decode(token);

    if (!payload) {
      throw new AuthenticationError('Invalid token');
    }

    // Remove standard claims that will be regenerated
    delete payload.iat;
    delete payload.exp;
    delete payload.nbf;

    return this.signToken(payload, expiresIn);
  }
}
