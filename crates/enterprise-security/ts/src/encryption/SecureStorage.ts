/**
 * Secure Storage - Encrypted Data Storage
 * Transparent encryption/decryption with automatic key rotation
 */

import { dataEncryption } from './DataEncryption';
import { Encryption } from '../types';

export class SecureStorage {
  private storage: Map<string, Encryption> = new Map();

  /**
   * Store encrypted value
   */
  async set(key: string, value: string, purpose: string = 'storage'): Promise<void> {
    const encrypted = await dataEncryption.encrypt(value, purpose);
    this.storage.set(key, encrypted);
  }

  /**
   * Retrieve and decrypt value
   */
  async get(key: string): Promise<string | null> {
    const encrypted = this.storage.get(key);
    if (!encrypted) {
      return null;
    }

    return await dataEncryption.decrypt(encrypted);
  }

  /**
   * Delete value
   */
  async delete(key: string): Promise<void> {
    this.storage.delete(key);
  }

  /**
   * Check if key exists
   */
  has(key: string): boolean {
    return this.storage.has(key);
  }

  /**
   * Get all keys
   */
  keys(): string[] {
    return Array.from(this.storage.keys());
  }

  /**
   * Clear all data
   */
  clear(): void {
    this.storage.clear();
  }
}

export const secureStorage = new SecureStorage();
