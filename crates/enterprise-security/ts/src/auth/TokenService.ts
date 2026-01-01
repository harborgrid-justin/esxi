/**
 * Token Service - JWT/OAuth Token Management
 * Enterprise token generation, validation, and lifecycle management
 */

import * as crypto from 'crypto';
import { nanoid } from 'nanoid';

// ============================================================================
// Types
// ============================================================================

export enum TokenType {
  ACCESS = 'ACCESS',
  REFRESH = 'REFRESH',
  ID = 'ID',
  API = 'API',
}

export interface TokenPayload {
  sub: string; // Subject (user ID)
  iss: string; // Issuer
  aud: string; // Audience
  exp: number; // Expiration time
  iat: number; // Issued at
  nbf?: number; // Not before
  jti: string; // JWT ID
  type: TokenType;
  scope?: string[];
  metadata?: Record<string, unknown>;
}

export interface TokenPair {
  accessToken: string;
  refreshToken: string;
  expiresIn: number;
  tokenType: string;
}

export interface TokenConfig {
  issuer: string;
  audience: string;
  accessTokenTTL: number; // seconds
  refreshTokenTTL: number; // seconds
  algorithm: string;
  secret: string;
  publicKey?: string;
  privateKey?: string;
}

export interface TokenValidationResult {
  valid: boolean;
  payload?: TokenPayload;
  error?: string;
}

// ============================================================================
// Token Service Implementation
// ============================================================================

export class TokenService {
  private config: TokenConfig;
  private revokedTokens: Set<string> = new Set();
  private tokenRegistry: Map<string, TokenPayload> = new Map();

  constructor(config: Partial<TokenConfig> = {}) {
    this.config = {
      issuer: config.issuer || 'harborgrid',
      audience: config.audience || 'harborgrid-api',
      accessTokenTTL: config.accessTokenTTL || 3600, // 1 hour
      refreshTokenTTL: config.refreshTokenTTL || 604800, // 7 days
      algorithm: config.algorithm || 'HS256',
      secret: config.secret || crypto.randomBytes(32).toString('hex'),
      publicKey: config.publicKey,
      privateKey: config.privateKey,
    };
  }

  /**
   * Generate token pair (access + refresh)
   */
  async generateTokenPair(
    userId: string,
    scope?: string[],
    metadata?: Record<string, unknown>
  ): Promise<TokenPair> {
    const accessToken = await this.generateToken(userId, TokenType.ACCESS, scope, metadata);
    const refreshToken = await this.generateToken(
      userId,
      TokenType.REFRESH,
      scope,
      metadata
    );

    return {
      accessToken,
      refreshToken,
      expiresIn: this.config.accessTokenTTL,
      tokenType: 'Bearer',
    };
  }

  /**
   * Generate token
   */
  async generateToken(
    userId: string,
    type: TokenType,
    scope?: string[],
    metadata?: Record<string, unknown>
  ): Promise<string> {
    const now = Math.floor(Date.now() / 1000);
    const ttl = type === TokenType.REFRESH
      ? this.config.refreshTokenTTL
      : this.config.accessTokenTTL;

    const payload: TokenPayload = {
      sub: userId,
      iss: this.config.issuer,
      aud: this.config.audience,
      exp: now + ttl,
      iat: now,
      nbf: now,
      jti: nanoid(),
      type,
      scope,
      metadata,
    };

    // Store in registry
    this.tokenRegistry.set(payload.jti, payload);

    // Create JWT
    const token = this.createJWT(payload);
    return token;
  }

  /**
   * Validate token
   */
  async validateToken(token: string): Promise<TokenValidationResult> {
    try {
      // Parse JWT
      const payload = this.parseJWT(token);

      // Verify signature
      if (!this.verifySignature(token)) {
        return { valid: false, error: 'Invalid signature' };
      }

      // Check expiration
      const now = Math.floor(Date.now() / 1000);
      if (payload.exp < now) {
        return { valid: false, error: 'Token expired' };
      }

      // Check not before
      if (payload.nbf && payload.nbf > now) {
        return { valid: false, error: 'Token not yet valid' };
      }

      // Check if revoked
      if (this.revokedTokens.has(payload.jti)) {
        return { valid: false, error: 'Token revoked' };
      }

      // Verify issuer and audience
      if (payload.iss !== this.config.issuer) {
        return { valid: false, error: 'Invalid issuer' };
      }

      if (payload.aud !== this.config.audience) {
        return { valid: false, error: 'Invalid audience' };
      }

      return { valid: true, payload };
    } catch (error) {
      return {
        valid: false,
        error: error instanceof Error ? error.message : 'Invalid token',
      };
    }
  }

  /**
   * Refresh access token using refresh token
   */
  async refreshAccessToken(refreshToken: string): Promise<string | null> {
    const validation = await this.validateToken(refreshToken);

    if (!validation.valid || !validation.payload) {
      return null;
    }

    if (validation.payload.type !== TokenType.REFRESH) {
      return null;
    }

    // Generate new access token
    const accessToken = await this.generateToken(
      validation.payload.sub,
      TokenType.ACCESS,
      validation.payload.scope,
      validation.payload.metadata
    );

    return accessToken;
  }

  /**
   * Revoke token
   */
  async revokeToken(token: string): Promise<void> {
    try {
      const payload = this.parseJWT(token);
      this.revokedTokens.add(payload.jti);
      this.tokenRegistry.delete(payload.jti);
    } catch (error) {
      // Token already invalid
    }
  }

  /**
   * Revoke all user tokens
   */
  async revokeUserTokens(userId: string): Promise<void> {
    for (const [jti, payload] of this.tokenRegistry.entries()) {
      if (payload.sub === userId) {
        this.revokedTokens.add(jti);
        this.tokenRegistry.delete(jti);
      }
    }
  }

  /**
   * Decode token (without validation)
   */
  decodeToken(token: string): TokenPayload | null {
    try {
      return this.parseJWT(token);
    } catch {
      return null;
    }
  }

  /**
   * Get token metadata
   */
  getTokenMetadata(token: string): Record<string, unknown> | null {
    const payload = this.decodeToken(token);
    return payload?.metadata || null;
  }

  /**
   * Check token expiration
   */
  isTokenExpired(token: string): boolean {
    const payload = this.decodeToken(token);
    if (!payload) return true;

    const now = Math.floor(Date.now() / 1000);
    return payload.exp < now;
  }

  /**
   * Get token TTL (seconds remaining)
   */
  getTokenTTL(token: string): number {
    const payload = this.decodeToken(token);
    if (!payload) return 0;

    const now = Math.floor(Date.now() / 1000);
    const ttl = payload.exp - now;
    return Math.max(0, ttl);
  }

  /**
   * Clean up expired tokens
   */
  async cleanupExpiredTokens(): Promise<number> {
    let cleaned = 0;
    const now = Math.floor(Date.now() / 1000);

    for (const [jti, payload] of this.tokenRegistry.entries()) {
      if (payload.exp < now) {
        this.tokenRegistry.delete(jti);
        this.revokedTokens.delete(jti);
        cleaned++;
      }
    }

    return cleaned;
  }

  /**
   * Get active tokens count
   */
  getActiveTokensCount(): number {
    const now = Math.floor(Date.now() / 1000);
    let count = 0;

    for (const payload of this.tokenRegistry.values()) {
      if (payload.exp >= now && !this.revokedTokens.has(payload.jti)) {
        count++;
      }
    }

    return count;
  }

  /**
   * Get user tokens
   */
  getUserTokens(userId: string): TokenPayload[] {
    const tokens: TokenPayload[] = [];
    const now = Math.floor(Date.now() / 1000);

    for (const payload of this.tokenRegistry.values()) {
      if (
        payload.sub === userId &&
        payload.exp >= now &&
        !this.revokedTokens.has(payload.jti)
      ) {
        tokens.push(payload);
      }
    }

    return tokens;
  }

  // ============================================================================
  // Private Helper Methods
  // ============================================================================

  private createJWT(payload: TokenPayload): string {
    // Header
    const header = {
      alg: this.config.algorithm,
      typ: 'JWT',
    };

    const encodedHeader = this.base64urlEncode(JSON.stringify(header));
    const encodedPayload = this.base64urlEncode(JSON.stringify(payload));

    // Signature
    const signature = this.sign(`${encodedHeader}.${encodedPayload}`);

    return `${encodedHeader}.${encodedPayload}.${signature}`;
  }

  private parseJWT(token: string): TokenPayload {
    const parts = token.split('.');
    if (parts.length !== 3) {
      throw new Error('Invalid JWT format');
    }

    const payloadStr = this.base64urlDecode(parts[1]!);
    return JSON.parse(payloadStr) as TokenPayload;
  }

  private sign(data: string): string {
    if (this.config.algorithm.startsWith('HS')) {
      // HMAC
      const hmac = crypto.createHmac(
        this.config.algorithm.replace('HS', 'sha'),
        this.config.secret
      );
      hmac.update(data);
      return this.base64urlEncode(hmac.digest());
    } else if (this.config.algorithm.startsWith('RS')) {
      // RSA
      if (!this.config.privateKey) {
        throw new Error('Private key required for RSA');
      }
      const sign = crypto.createSign(this.config.algorithm.replace('RS', 'RSA-SHA'));
      sign.update(data);
      return this.base64urlEncode(sign.sign(this.config.privateKey));
    }

    throw new Error('Unsupported algorithm');
  }

  private verifySignature(token: string): boolean {
    const parts = token.split('.');
    if (parts.length !== 3) {
      return false;
    }

    const data = `${parts[0]}.${parts[1]}`;
    const signature = parts[2]!;
    const expectedSignature = this.sign(data);

    return signature === expectedSignature;
  }

  private base64urlEncode(data: string | Buffer): string {
    const buffer = typeof data === 'string' ? Buffer.from(data) : data;
    return buffer
      .toString('base64')
      .replace(/\+/g, '-')
      .replace(/\//g, '_')
      .replace(/=/g, '');
  }

  private base64urlDecode(data: string): string {
    let base64 = data.replace(/-/g, '+').replace(/_/g, '/');
    while (base64.length % 4) {
      base64 += '=';
    }
    return Buffer.from(base64, 'base64').toString('utf8');
  }
}

// Export singleton instance
export const tokenService = new TokenService();
