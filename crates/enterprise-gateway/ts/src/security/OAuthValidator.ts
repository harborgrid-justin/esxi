/**
 * Enterprise API Gateway - OAuth2 Validator
 *
 * OAuth2 token validation and introspection
 */

import type { OAuthConfig, OAuthToken, GatewayRequest } from '../types';
import { AuthenticationError, AuthorizationError } from '../types';

export class OAuthValidator {
  private tokens: Map<string, OAuthToken> = new Map();

  constructor(private config: OAuthConfig) {}

  /**
   * Validate OAuth2 token
   */
  public async validate(
    request: GatewayRequest,
    requiredScopes?: string[]
  ): Promise<OAuthToken> {
    // Extract token from request
    const tokenValue = this.extractTokenFromRequest(request);

    if (!tokenValue) {
      throw new AuthenticationError('OAuth token not found in request');
    }

    // Validate token
    const token = await this.validateToken(tokenValue);

    // Check scopes
    if (requiredScopes && requiredScopes.length > 0) {
      const hasRequiredScopes = requiredScopes.every((scope) =>
        token.scope.includes(scope)
      );

      if (!hasRequiredScopes) {
        throw new AuthorizationError('Insufficient scopes for this operation');
      }
    }

    return token;
  }

  /**
   * Validate token (check expiration and optionally introspect)
   */
  private async validateToken(tokenValue: string): Promise<OAuthToken> {
    // Check local cache first
    const cachedToken = this.tokens.get(tokenValue);

    if (cachedToken) {
      // Check if expired
      const expiresAt = cachedToken.issuedAt + cachedToken.expiresIn * 1000;
      if (Date.now() < expiresAt) {
        return cachedToken;
      }

      // Remove expired token
      this.tokens.delete(tokenValue);
    }

    // Introspect token if endpoint is configured
    if (this.config.introspectionEndpoint) {
      const token = await this.introspectToken(tokenValue);
      this.tokens.set(tokenValue, token);
      return token;
    }

    throw new AuthenticationError('Invalid or expired OAuth token');
  }

  /**
   * Introspect token with OAuth server
   */
  private async introspectToken(token: string): Promise<OAuthToken> {
    // This is a simplified version - in production, make actual HTTP request
    // to the introspection endpoint

    // Simulated introspection response
    return {
      accessToken: token,
      tokenType: 'Bearer',
      expiresIn: 3600,
      scope: ['read', 'write'],
      issuedAt: Date.now(),
    };
  }

  /**
   * Extract token from request
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
    const tokenQuery = request.query['access_token'];
    if (tokenQuery) {
      return Array.isArray(tokenQuery) ? tokenQuery[0]! : tokenQuery;
    }

    return null;
  }

  /**
   * Exchange authorization code for token
   */
  public async exchangeCode(code: string, redirectUri: string): Promise<OAuthToken> {
    // This is a simplified version - in production, make actual HTTP request
    // to the token endpoint

    const token: OAuthToken = {
      accessToken: `access_${code}`,
      tokenType: 'Bearer',
      expiresIn: 3600,
      refreshToken: `refresh_${code}`,
      scope: this.config.scopes,
      issuedAt: Date.now(),
    };

    this.tokens.set(token.accessToken, token);
    return token;
  }

  /**
   * Refresh access token
   */
  public async refreshToken(refreshToken: string): Promise<OAuthToken> {
    // This is a simplified version - in production, make actual HTTP request
    // to the token endpoint

    const token: OAuthToken = {
      accessToken: `refreshed_${Date.now()}`,
      tokenType: 'Bearer',
      expiresIn: 3600,
      refreshToken,
      scope: this.config.scopes,
      issuedAt: Date.now(),
    };

    this.tokens.set(token.accessToken, token);
    return token;
  }

  /**
   * Revoke token
   */
  public revokeToken(token: string): void {
    this.tokens.delete(token);
  }

  /**
   * Clear expired tokens
   */
  public clearExpiredTokens(): number {
    const now = Date.now();
    let cleared = 0;

    for (const [tokenValue, token] of this.tokens.entries()) {
      const expiresAt = token.issuedAt + token.expiresIn * 1000;
      if (now >= expiresAt) {
        this.tokens.delete(tokenValue);
        cleared++;
      }
    }

    return cleared;
  }

  /**
   * Get statistics
   */
  public getStatistics(): {
    totalTokens: number;
    validTokens: number;
    expiredTokens: number;
  } {
    const now = Date.now();
    let valid = 0;
    let expired = 0;

    for (const token of this.tokens.values()) {
      const expiresAt = token.issuedAt + token.expiresIn * 1000;
      if (now < expiresAt) {
        valid++;
      } else {
        expired++;
      }
    }

    return {
      totalTokens: this.tokens.size,
      validTokens: valid,
      expiredTokens: expired,
    };
  }
}
