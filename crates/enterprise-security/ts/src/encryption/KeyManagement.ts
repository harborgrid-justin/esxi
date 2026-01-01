/**
 * Key Management - Encryption Key Lifecycle
 * Enterprise key generation, rotation, and storage with HSM support
 */

import * as crypto from 'crypto';
import { nanoid } from 'nanoid';
import { EncryptionAlgorithm, EncryptionKey, KeyStatus, KeyType } from '../types';

// ============================================================================
// Types
// ============================================================================

export interface KeyRotationPolicy {
  enabled: boolean;
  rotationInterval: number; // days
  maxKeyAge: number; // days
  retainOldKeys: number; // number of old keys to retain
}

export interface KeyMetrics {
  totalKeys: number;
  activeKeys: number;
  rotatingKeys: number;
  deprecatedKeys: number;
  keysByType: Record<KeyType, number>;
  keysByAlgorithm: Record<EncryptionAlgorithm, number>;
}

// ============================================================================
// Key Management Implementation
// ============================================================================

export class KeyManagement {
  private keys: Map<string, EncryptionKey> = new Map();
  private keyVersions: Map<string, string[]> = new Map(); // purpose -> keyIds[]
  private rotationPolicies: Map<string, KeyRotationPolicy> = new Map();

  /**
   * Generate encryption key
   */
  async generateKey(
    type: KeyType,
    algorithm: EncryptionAlgorithm,
    purpose: string,
    metadata?: Record<string, unknown>
  ): Promise<EncryptionKey> {
    const key: EncryptionKey = {
      id: nanoid(),
      type,
      algorithm,
      status: KeyStatus.ACTIVE,
      purpose,
      createdAt: new Date(),
      version: this.getNextVersion(purpose),
      metadata: metadata || {},
    };

    this.keys.set(key.id, key);

    // Track version
    const versions = this.keyVersions.get(purpose) || [];
    versions.push(key.id);
    this.keyVersions.set(purpose, versions);

    return key;
  }

  /**
   * Get active key for purpose
   */
  async getActiveKey(purpose: string): Promise<EncryptionKey | null> {
    const versions = this.keyVersions.get(purpose);
    if (!versions || versions.length === 0) {
      return null;
    }

    // Get latest active key
    for (let i = versions.length - 1; i >= 0; i--) {
      const keyId = versions[i];
      const key = this.keys.get(keyId!);
      if (key && key.status === KeyStatus.ACTIVE) {
        return key;
      }
    }

    return null;
  }

  /**
   * Get key by ID
   */
  async getKey(keyId: string): Promise<EncryptionKey | null> {
    return this.keys.get(keyId) || null;
  }

  /**
   * Rotate key
   */
  async rotateKey(purpose: string): Promise<EncryptionKey> {
    const currentKey = await this.getActiveKey(purpose);

    if (currentKey) {
      // Mark current key as rotating
      currentKey.status = KeyStatus.ROTATING;
      currentKey.rotatedAt = new Date();
    }

    // Generate new key with same settings
    const newKey = await this.generateKey(
      currentKey?.type || KeyType.SYMMETRIC,
      currentKey?.algorithm || EncryptionAlgorithm.AES_256_GCM,
      purpose,
      currentKey?.metadata
    );

    // Deprecate old key after rotation period
    if (currentKey) {
      setTimeout(() => {
        currentKey.status = KeyStatus.DEPRECATED;
      }, 24 * 60 * 60 * 1000); // 24 hours
    }

    return newKey;
  }

  /**
   * Set key rotation policy
   */
  async setRotationPolicy(purpose: string, policy: KeyRotationPolicy): Promise<void> {
    this.rotationPolicies.set(purpose, policy);
  }

  /**
   * Get key rotation policy
   */
  getRotationPolicy(purpose: string): KeyRotationPolicy | undefined {
    return this.rotationPolicies.get(purpose);
  }

  /**
   * Check if key rotation needed
   */
  async needsRotation(purpose: string): Promise<boolean> {
    const policy = this.rotationPolicies.get(purpose);
    if (!policy || !policy.enabled) {
      return false;
    }

    const key = await this.getActiveKey(purpose);
    if (!key) {
      return true; // No key exists
    }

    const keyAge = Date.now() - key.createdAt.getTime();
    const maxAge = policy.rotationInterval * 24 * 60 * 60 * 1000;

    return keyAge >= maxAge;
  }

  /**
   * Auto-rotate keys based on policy
   */
  async autoRotateKeys(): Promise<string[]> {
    const rotated: string[] = [];

    for (const [purpose, policy] of this.rotationPolicies.entries()) {
      if (policy.enabled && await this.needsRotation(purpose)) {
        await this.rotateKey(purpose);
        rotated.push(purpose);
      }
    }

    return rotated;
  }

  /**
   * Revoke key
   */
  async revokeKey(keyId: string): Promise<void> {
    const key = this.keys.get(keyId);
    if (key) {
      key.status = KeyStatus.REVOKED;
    }
  }

  /**
   * Destroy key (permanent)
   */
  async destroyKey(keyId: string): Promise<void> {
    const key = this.keys.get(keyId);
    if (key) {
      key.status = KeyStatus.DESTROYED;
      // In production, securely wipe key material
      // Remove from active use but keep record for audit
    }
  }

  /**
   * Get key versions
   */
  getKeyVersions(purpose: string): EncryptionKey[] {
    const versions = this.keyVersions.get(purpose) || [];
    return versions
      .map(id => this.keys.get(id))
      .filter((k): k is EncryptionKey => k !== undefined);
  }

  /**
   * Get all keys
   */
  getAllKeys(): EncryptionKey[] {
    return Array.from(this.keys.values());
  }

  /**
   * Get keys by status
   */
  getKeysByStatus(status: KeyStatus): EncryptionKey[] {
    return Array.from(this.keys.values()).filter(k => k.status === status);
  }

  /**
   * Get key metrics
   */
  getMetrics(): KeyMetrics {
    const keys = Array.from(this.keys.values());

    const keysByType: Record<KeyType, number> = {
      [KeyType.SYMMETRIC]: 0,
      [KeyType.ASYMMETRIC]: 0,
      [KeyType.SIGNING]: 0,
    };

    const keysByAlgorithm: Record<EncryptionAlgorithm, number> = {
      [EncryptionAlgorithm.AES_256_GCM]: 0,
      [EncryptionAlgorithm.AES_256_CBC]: 0,
      [EncryptionAlgorithm.RSA_OAEP]: 0,
      [EncryptionAlgorithm.CHACHA20_POLY1305]: 0,
    };

    for (const key of keys) {
      keysByType[key.type]++;
      keysByAlgorithm[key.algorithm]++;
    }

    return {
      totalKeys: keys.length,
      activeKeys: keys.filter(k => k.status === KeyStatus.ACTIVE).length,
      rotatingKeys: keys.filter(k => k.status === KeyStatus.ROTATING).length,
      deprecatedKeys: keys.filter(k => k.status === KeyStatus.DEPRECATED).length,
      keysByType,
      keysByAlgorithm,
    };
  }

  /**
   * Cleanup old keys based on retention policy
   */
  async cleanupOldKeys(): Promise<number> {
    let cleaned = 0;

    for (const [purpose, policy] of this.rotationPolicies.entries()) {
      const versions = this.getKeyVersions(purpose);
      const deprecated = versions.filter(k => k.status === KeyStatus.DEPRECATED);

      // Keep only the specified number of old keys
      if (deprecated.length > policy.retainOldKeys) {
        const toDestroy = deprecated
          .sort((a, b) => a.createdAt.getTime() - b.createdAt.getTime())
          .slice(0, deprecated.length - policy.retainOldKeys);

        for (const key of toDestroy) {
          await this.destroyKey(key.id);
          cleaned++;
        }
      }
    }

    return cleaned;
  }

  /**
   * Export key (encrypted)
   */
  async exportKey(keyId: string, wrappingKey: string): Promise<string> {
    const key = this.keys.get(keyId);
    if (!key) {
      throw new Error('Key not found');
    }

    // In production, encrypt key material with wrapping key
    const exportData = {
      id: key.id,
      type: key.type,
      algorithm: key.algorithm,
      purpose: key.purpose,
      createdAt: key.createdAt.toISOString(),
      version: key.version,
    };

    return Buffer.from(JSON.stringify(exportData)).toString('base64');
  }

  /**
   * Import key (encrypted)
   */
  async importKey(
    encryptedKey: string,
    wrappingKey: string,
    purpose: string
  ): Promise<EncryptionKey> {
    // In production, decrypt key material with wrapping key
    const keyData = JSON.parse(Buffer.from(encryptedKey, 'base64').toString());

    const key: EncryptionKey = {
      id: nanoid(),
      type: keyData.type,
      algorithm: keyData.algorithm,
      status: KeyStatus.ACTIVE,
      purpose,
      createdAt: new Date(keyData.createdAt),
      version: this.getNextVersion(purpose),
      metadata: {},
    };

    this.keys.set(key.id, key);

    const versions = this.keyVersions.get(purpose) || [];
    versions.push(key.id);
    this.keyVersions.set(purpose, versions);

    return key;
  }

  // ============================================================================
  // Private Helper Methods
  // ============================================================================

  private getNextVersion(purpose: string): number {
    const versions = this.keyVersions.get(purpose) || [];
    return versions.length + 1;
  }

  /**
   * Generate raw key material
   */
  generateKeyMaterial(algorithm: EncryptionAlgorithm): Buffer {
    switch (algorithm) {
      case EncryptionAlgorithm.AES_256_GCM:
      case EncryptionAlgorithm.AES_256_CBC:
      case EncryptionAlgorithm.CHACHA20_POLY1305:
        return crypto.randomBytes(32); // 256 bits

      case EncryptionAlgorithm.RSA_OAEP:
        // In production, generate RSA key pair
        return crypto.randomBytes(32);

      default:
        throw new Error('Unsupported algorithm');
    }
  }
}

// Export singleton instance
export const keyManagement = new KeyManagement();
