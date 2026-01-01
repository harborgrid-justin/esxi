/**
 * Data Encryption - AES-256-GCM Encryption
 * Field-level and bulk data encryption with key management integration
 */

import * as crypto from 'crypto';
import { Encryption, EncryptionAlgorithm } from '../types';
import { keyManagement } from './KeyManagement';

export class DataEncryption {
  private readonly IV_LENGTH = 12; // GCM standard
  private readonly AUTH_TAG_LENGTH = 16;

  /**
   * Encrypt data with AES-256-GCM
   */
  async encrypt(data: string, purpose: string = 'data'): Promise<Encryption> {
    const key = await keyManagement.getActiveKey(purpose);
    if (!key) {
      throw new Error('No active encryption key found');
    }

    const keyMaterial = keyManagement.generateKeyMaterial(key.algorithm);
    const iv = crypto.randomBytes(this.IV_LENGTH);

    const cipher = crypto.createCipheriv('aes-256-gcm', keyMaterial, iv);

    let encrypted = cipher.update(data, 'utf8', 'base64');
    encrypted += cipher.final('base64');

    const authTag = cipher.getAuthTag();

    return {
      algorithm: key.algorithm,
      keyId: key.id,
      iv: iv.toString('base64'),
      authTag: authTag.toString('base64'),
      encryptedData: encrypted,
      metadata: {
        encryptedAt: new Date(),
        encryptedBy: 'system',
        version: key.version,
      },
    };
  }

  /**
   * Decrypt data
   */
  async decrypt(encryption: Encryption): Promise<string> {
    const key = await keyManagement.getKey(encryption.keyId);
    if (!key) {
      throw new Error('Encryption key not found');
    }

    const keyMaterial = keyManagement.generateKeyMaterial(key.algorithm);
    const iv = Buffer.from(encryption.iv, 'base64');
    const authTag = Buffer.from(encryption.authTag!, 'base64');

    const decipher = crypto.createDecipheriv('aes-256-gcm', keyMaterial, iv);
    decipher.setAuthTag(authTag);

    let decrypted = decipher.update(encryption.encryptedData, 'base64', 'utf8');
    decrypted += decipher.final('utf8');

    return decrypted;
  }

  /**
   * Bulk encrypt multiple fields
   */
  async encryptObject(
    obj: Record<string, any>,
    fields: string[]
  ): Promise<Record<string, any>> {
    const encrypted: Record<string, any> = { ...obj };

    for (const field of fields) {
      if (obj[field] !== undefined) {
        encrypted[field] = await this.encrypt(String(obj[field]));
      }
    }

    return encrypted;
  }

  /**
   * Bulk decrypt multiple fields
   */
  async decryptObject(
    obj: Record<string, any>,
    fields: string[]
  ): Promise<Record<string, any>> {
    const decrypted: Record<string, any> = { ...obj };

    for (const field of fields) {
      if (obj[field] && typeof obj[field] === 'object') {
        decrypted[field] = await this.decrypt(obj[field] as Encryption);
      }
    }

    return decrypted;
  }
}

export const dataEncryption = new DataEncryption();
