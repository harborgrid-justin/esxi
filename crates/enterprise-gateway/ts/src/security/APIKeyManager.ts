/**
 * Enterprise API Gateway - API Key Manager
 *
 * API key generation, validation, and management
 */

import { randomBytes, createHash } from 'crypto';
import type { APIKey, Consumer, GatewayRequest } from '../types';
import { AuthenticationError, AuthorizationError } from '../types';

export class APIKeyManager {
  private keys: Map<string, APIKey> = new Map();
  private keysByConsumer: Map<string, Set<string>> = new Map();

  /**
   * Generate a new API key
   */
  public generateKey(
    consumerId: string,
    name: string,
    scopes: string[] = [],
    expiresIn?: number
  ): APIKey {
    const key = this.generateSecureKey();
    const hashedKey = this.hashKey(key);

    const apiKey: APIKey = {
      id: randomBytes(16).toString('hex'),
      key: hashedKey,
      name,
      consumerId,
      scopes,
      enabled: true,
      expiresAt: expiresIn ? Date.now() + expiresIn : undefined,
      createdAt: Date.now(),
    };

    this.keys.set(hashedKey, apiKey);

    // Index by consumer
    const consumerKeys = this.keysByConsumer.get(consumerId) || new Set();
    consumerKeys.add(hashedKey);
    this.keysByConsumer.set(consumerId, consumerKeys);

    // Return the plain key only once (for display to user)
    return { ...apiKey, key };
  }

  /**
   * Validate an API key
   */
  public async validate(
    request: GatewayRequest,
    requiredScopes?: string[]
  ): Promise<APIKey> {
    // Extract API key from request
    const keyValue = this.extractKeyFromRequest(request);

    if (!keyValue) {
      throw new AuthenticationError('API key not found in request');
    }

    // Hash the provided key
    const hashedKey = this.hashKey(keyValue);

    // Look up the key
    const apiKey = this.keys.get(hashedKey);

    if (!apiKey) {
      throw new AuthenticationError('Invalid API key');
    }

    // Check if enabled
    if (!apiKey.enabled) {
      throw new AuthenticationError('API key is disabled');
    }

    // Check if expired
    if (apiKey.expiresAt && Date.now() > apiKey.expiresAt) {
      throw new AuthenticationError('API key has expired');
    }

    // Check scopes
    if (requiredScopes && requiredScopes.length > 0) {
      const hasRequiredScopes = requiredScopes.every((scope) =>
        apiKey.scopes.includes(scope)
      );

      if (!hasRequiredScopes) {
        throw new AuthorizationError('Insufficient scopes for this operation');
      }
    }

    return apiKey;
  }

  /**
   * Extract API key from request
   */
  private extractKeyFromRequest(request: GatewayRequest): string | null {
    // Check Authorization header (Bearer token)
    const authHeader = request.headers['authorization'];
    if (authHeader) {
      const match = authHeader.toString().match(/^Bearer (.+)$/);
      if (match && match[1]) {
        return match[1];
      }
    }

    // Check X-API-Key header
    const apiKeyHeader = request.headers['x-api-key'];
    if (apiKeyHeader) {
      return apiKeyHeader.toString();
    }

    // Check query parameter
    const apiKeyQuery = request.query['api_key'];
    if (apiKeyQuery) {
      return Array.isArray(apiKeyQuery) ? apiKeyQuery[0]! : apiKeyQuery;
    }

    return null;
  }

  /**
   * Generate a secure random API key
   */
  private generateSecureKey(): string {
    return randomBytes(32).toString('base64url');
  }

  /**
   * Hash an API key for storage
   */
  private hashKey(key: string): string {
    return createHash('sha256').update(key).digest('hex');
  }

  /**
   * Revoke an API key
   */
  public revokeKey(keyId: string): boolean {
    for (const [hashedKey, apiKey] of this.keys.entries()) {
      if (apiKey.id === keyId) {
        apiKey.enabled = false;
        this.keys.set(hashedKey, apiKey);
        return true;
      }
    }
    return false;
  }

  /**
   * Delete an API key
   */
  public deleteKey(keyId: string): boolean {
    for (const [hashedKey, apiKey] of this.keys.entries()) {
      if (apiKey.id === keyId) {
        this.keys.delete(hashedKey);

        // Remove from consumer index
        const consumerKeys = this.keysByConsumer.get(apiKey.consumerId);
        if (consumerKeys) {
          consumerKeys.delete(hashedKey);
          if (consumerKeys.size === 0) {
            this.keysByConsumer.delete(apiKey.consumerId);
          }
        }

        return true;
      }
    }
    return false;
  }

  /**
   * Get all keys for a consumer
   */
  public getConsumerKeys(consumerId: string): APIKey[] {
    const keyHashes = this.keysByConsumer.get(consumerId);
    if (!keyHashes) return [];

    const keys: APIKey[] = [];
    for (const hash of keyHashes) {
      const key = this.keys.get(hash);
      if (key) {
        keys.push(key);
      }
    }

    return keys;
  }

  /**
   * Update API key scopes
   */
  public updateScopes(keyId: string, scopes: string[]): boolean {
    for (const [hashedKey, apiKey] of this.keys.entries()) {
      if (apiKey.id === keyId) {
        apiKey.scopes = scopes;
        this.keys.set(hashedKey, apiKey);
        return true;
      }
    }
    return false;
  }

  /**
   * Extend API key expiration
   */
  public extendExpiration(keyId: string, extensionMs: number): boolean {
    for (const [hashedKey, apiKey] of this.keys.entries()) {
      if (apiKey.id === keyId) {
        if (apiKey.expiresAt) {
          apiKey.expiresAt += extensionMs;
        } else {
          apiKey.expiresAt = Date.now() + extensionMs;
        }
        this.keys.set(hashedKey, apiKey);
        return true;
      }
    }
    return false;
  }

  /**
   * Get statistics
   */
  public getStatistics(): {
    totalKeys: number;
    activeKeys: number;
    expiredKeys: number;
    consumers: number;
  } {
    let active = 0;
    let expired = 0;
    const now = Date.now();

    for (const key of this.keys.values()) {
      if (key.enabled && (!key.expiresAt || key.expiresAt > now)) {
        active++;
      } else if (key.expiresAt && key.expiresAt <= now) {
        expired++;
      }
    }

    return {
      totalKeys: this.keys.size,
      activeKeys: active,
      expiredKeys: expired,
      consumers: this.keysByConsumer.size,
    };
  }
}
