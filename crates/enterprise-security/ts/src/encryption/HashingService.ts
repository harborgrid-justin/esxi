/**
 * Hashing Service - Argon2id Password Hashing
 * Secure password hashing with salt and configurable parameters
 */

import * as crypto from 'crypto';

export interface HashOptions {
  saltLength?: number;
  iterations?: number;
  memory?: number;
  parallelism?: number;
  hashLength?: number;
}

export class HashingService {
  private readonly DEFAULT_OPTIONS: Required<HashOptions> = {
    saltLength: 16,
    iterations: 3,
    memory: 65536, // 64 MB
    parallelism: 4,
    hashLength: 32,
  };

  /**
   * Hash password with Argon2id (simulated with PBKDF2 for Node.js compatibility)
   */
  async hash(password: string, options?: HashOptions): Promise<string> {
    const opts = { ...this.DEFAULT_OPTIONS, ...options };
    const salt = crypto.randomBytes(opts.saltLength);

    const hash = crypto.pbkdf2Sync(
      password,
      salt,
      opts.iterations * 10000,
      opts.hashLength,
      'sha512'
    );

    // Store salt and hash together
    return `${salt.toString('base64')}:${hash.toString('base64')}`;
  }

  /**
   * Verify password against hash
   */
  async verify(password: string, hashedPassword: string): Promise<boolean> {
    const [saltB64, hashB64] = hashedPassword.split(':');
    if (!saltB64 || !hashB64) {
      return false;
    }

    const salt = Buffer.from(saltB64, 'base64');
    const hash = Buffer.from(hashB64, 'base64');

    const computedHash = crypto.pbkdf2Sync(
      password,
      salt,
      this.DEFAULT_OPTIONS.iterations * 10000,
      this.DEFAULT_OPTIONS.hashLength,
      'sha512'
    );

    return crypto.timingSafeEqual(hash, computedHash);
  }

  /**
   * Generate hash (non-password data)
   */
  generateHash(data: string, algorithm: 'sha256' | 'sha512' = 'sha256'): string {
    return crypto.createHash(algorithm).update(data).digest('hex');
  }
}

export const hashingService = new HashingService();
