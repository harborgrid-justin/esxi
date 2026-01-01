/**
 * MFA Manager - Multi-Factor Authentication
 * Support for TOTP, SMS, Email, and Hardware tokens
 */

import { nanoid } from 'nanoid';
import * as crypto from 'crypto';

// ============================================================================
// Types
// ============================================================================

export enum MFAMethod {
  TOTP = 'TOTP',
  SMS = 'SMS',
  EMAIL = 'EMAIL',
  HARDWARE_TOKEN = 'HARDWARE_TOKEN',
  BACKUP_CODES = 'BACKUP_CODES',
  PUSH_NOTIFICATION = 'PUSH_NOTIFICATION',
}

export enum MFAStatus {
  ENABLED = 'ENABLED',
  DISABLED = 'DISABLED',
  PENDING_SETUP = 'PENDING_SETUP',
  SUSPENDED = 'SUSPENDED',
}

export interface MFAConfig {
  userId: string;
  method: MFAMethod;
  status: MFAStatus;
  secret?: string;
  phoneNumber?: string;
  email?: string;
  backupCodes?: string[];
  createdAt: Date;
  lastUsedAt?: Date;
  metadata: Record<string, unknown>;
}

export interface MFAChallenge {
  id: string;
  userId: string;
  method: MFAMethod;
  code?: string;
  expiresAt: Date;
  attempts: number;
  maxAttempts: number;
  verified: boolean;
}

export interface TOTPConfig {
  secret: string;
  qrCode: string;
  backupCodes: string[];
}

// ============================================================================
// MFA Manager Implementation
// ============================================================================

export class MFAManager {
  private configs: Map<string, MFAConfig[]> = new Map();
  private challenges: Map<string, MFAChallenge> = new Map();
  private readonly CODE_LENGTH = 6;
  private readonly CODE_VALIDITY = 5 * 60 * 1000; // 5 minutes
  private readonly MAX_ATTEMPTS = 3;

  /**
   * Setup TOTP for user
   */
  async setupTOTP(userId: string): Promise<TOTPConfig> {
    // Generate secret (base32 encoded)
    const secret = this.generateSecret();

    // Generate QR code URL
    const qrCode = this.generateTOTPQRCode(userId, secret);

    // Generate backup codes
    const backupCodes = this.generateBackupCodes();

    // Store configuration (pending)
    const config: MFAConfig = {
      userId,
      method: MFAMethod.TOTP,
      status: MFAStatus.PENDING_SETUP,
      secret,
      backupCodes,
      createdAt: new Date(),
      metadata: {},
    };

    this.addConfig(userId, config);

    return {
      secret,
      qrCode,
      backupCodes,
    };
  }

  /**
   * Verify TOTP setup and enable
   */
  async verifyTOTPSetup(userId: string, code: string): Promise<boolean> {
    const config = this.getConfig(userId, MFAMethod.TOTP);
    if (!config || !config.secret) {
      throw new Error('TOTP not configured');
    }

    const isValid = this.verifyTOTPCode(config.secret, code);
    if (isValid) {
      config.status = MFAStatus.ENABLED;
      config.lastUsedAt = new Date();
      return true;
    }

    return false;
  }

  /**
   * Setup SMS MFA
   */
  async setupSMS(userId: string, phoneNumber: string): Promise<void> {
    const config: MFAConfig = {
      userId,
      method: MFAMethod.SMS,
      status: MFAStatus.PENDING_SETUP,
      phoneNumber,
      createdAt: new Date(),
      metadata: {},
    };

    this.addConfig(userId, config);

    // Send verification code
    await this.sendSMSChallenge(userId);
  }

  /**
   * Setup Email MFA
   */
  async setupEmail(userId: string, email: string): Promise<void> {
    const config: MFAConfig = {
      userId,
      method: MFAMethod.EMAIL,
      status: MFAStatus.PENDING_SETUP,
      email,
      createdAt: new Date(),
      metadata: {},
    };

    this.addConfig(userId, config);

    // Send verification code
    await this.sendEmailChallenge(userId);
  }

  /**
   * Send MFA challenge
   */
  async sendChallenge(userId: string, method: MFAMethod): Promise<string> {
    const config = this.getConfig(userId, method);
    if (!config || config.status !== MFAStatus.ENABLED) {
      throw new Error('MFA method not enabled');
    }

    const challenge: MFAChallenge = {
      id: nanoid(),
      userId,
      method,
      code: this.generateCode(),
      expiresAt: new Date(Date.now() + this.CODE_VALIDITY),
      attempts: 0,
      maxAttempts: this.MAX_ATTEMPTS,
      verified: false,
    };

    this.challenges.set(challenge.id, challenge);

    // Send challenge based on method
    switch (method) {
      case MFAMethod.SMS:
        await this.sendSMSCode(config.phoneNumber!, challenge.code!);
        break;
      case MFAMethod.EMAIL:
        await this.sendEmailCode(config.email!, challenge.code!);
        break;
      case MFAMethod.PUSH_NOTIFICATION:
        await this.sendPushNotification(userId, challenge.id);
        break;
      default:
        // TOTP doesn't need to send anything
        break;
    }

    return challenge.id;
  }

  /**
   * Verify MFA challenge
   */
  async verifyChallenge(challengeId: string, code: string): Promise<boolean> {
    const challenge = this.challenges.get(challengeId);
    if (!challenge) {
      throw new Error('Invalid challenge');
    }

    // Check expiration
    if (new Date() > challenge.expiresAt) {
      this.challenges.delete(challengeId);
      throw new Error('Challenge expired');
    }

    // Check max attempts
    challenge.attempts++;
    if (challenge.attempts > challenge.maxAttempts) {
      this.challenges.delete(challengeId);
      throw new Error('Maximum attempts exceeded');
    }

    const config = this.getConfig(challenge.userId, challenge.method);
    if (!config) {
      throw new Error('MFA configuration not found');
    }

    let isValid = false;

    switch (challenge.method) {
      case MFAMethod.TOTP:
        isValid = this.verifyTOTPCode(config.secret!, code);
        break;
      case MFAMethod.BACKUP_CODES:
        isValid = this.verifyBackupCode(config, code);
        break;
      default:
        isValid = challenge.code === code;
        break;
    }

    if (isValid) {
      challenge.verified = true;
      config.lastUsedAt = new Date();
      this.challenges.delete(challengeId);
    }

    return isValid;
  }

  /**
   * Disable MFA method
   */
  async disableMFA(userId: string, method: MFAMethod): Promise<void> {
    const configs = this.configs.get(userId);
    if (!configs) {
      return;
    }

    const config = configs.find((c) => c.method === method);
    if (config) {
      config.status = MFAStatus.DISABLED;
    }
  }

  /**
   * Get user's MFA methods
   */
  getMFAMethods(userId: string): MFAConfig[] {
    return this.configs.get(userId) || [];
  }

  /**
   * Check if user has MFA enabled
   */
  hasMFAEnabled(userId: string): boolean {
    const configs = this.configs.get(userId) || [];
    return configs.some((c) => c.status === MFAStatus.ENABLED);
  }

  /**
   * Regenerate backup codes
   */
  async regenerateBackupCodes(userId: string): Promise<string[]> {
    const config = this.getConfig(userId, MFAMethod.TOTP);
    if (!config) {
      throw new Error('TOTP not configured');
    }

    const backupCodes = this.generateBackupCodes();
    config.backupCodes = backupCodes;

    return backupCodes;
  }

  // ============================================================================
  // Private Helper Methods
  // ============================================================================

  private generateSecret(): string {
    // Generate 160-bit secret (32 base32 characters)
    const buffer = crypto.randomBytes(20);
    return this.base32Encode(buffer);
  }

  private base32Encode(buffer: Buffer): string {
    const alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567';
    let bits = '';
    let result = '';

    for (let i = 0; i < buffer.length; i++) {
      bits += buffer[i]!.toString(2).padStart(8, '0');
    }

    for (let i = 0; i < bits.length; i += 5) {
      const chunk = bits.slice(i, i + 5).padEnd(5, '0');
      result += alphabet[parseInt(chunk, 2)]!;
    }

    return result;
  }

  private generateTOTPQRCode(userId: string, secret: string): string {
    const issuer = 'HarborGrid';
    const label = `${issuer}:${userId}`;
    const otpauth = `otpauth://totp/${encodeURIComponent(label)}?secret=${secret}&issuer=${encodeURIComponent(issuer)}`;
    return otpauth;
  }

  private verifyTOTPCode(secret: string, code: string): boolean {
    // Get current time step (30 second window)
    const timeStep = Math.floor(Date.now() / 1000 / 30);

    // Check current and adjacent time steps (for clock drift)
    for (let i = -1; i <= 1; i++) {
      const stepCode = this.generateTOTPCode(secret, timeStep + i);
      if (stepCode === code) {
        return true;
      }
    }

    return false;
  }

  private generateTOTPCode(secret: string, timeStep: number): string {
    const buffer = Buffer.alloc(8);
    buffer.writeBigInt64BE(BigInt(timeStep));

    const secretBuffer = Buffer.from(this.base32Decode(secret));
    const hmac = crypto.createHmac('sha1', secretBuffer);
    hmac.update(buffer);
    const hash = hmac.digest();

    const offset = hash[hash.length - 1]! & 0x0f;
    const binary =
      ((hash[offset]! & 0x7f) << 24) |
      ((hash[offset + 1]! & 0xff) << 16) |
      ((hash[offset + 2]! & 0xff) << 8) |
      (hash[offset + 3]! & 0xff);

    const otp = binary % 1000000;
    return otp.toString().padStart(6, '0');
  }

  private base32Decode(encoded: string): string {
    const alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567';
    let bits = '';

    for (const char of encoded.toUpperCase()) {
      const val = alphabet.indexOf(char);
      if (val === -1) continue;
      bits += val.toString(2).padStart(5, '0');
    }

    const bytes: number[] = [];
    for (let i = 0; i + 8 <= bits.length; i += 8) {
      bytes.push(parseInt(bits.slice(i, i + 8), 2));
    }

    return Buffer.from(bytes).toString('binary');
  }

  private generateCode(): string {
    const code = crypto.randomInt(0, 10 ** this.CODE_LENGTH);
    return code.toString().padStart(this.CODE_LENGTH, '0');
  }

  private generateBackupCodes(count: number = 10): string[] {
    const codes: string[] = [];
    for (let i = 0; i < count; i++) {
      const code = crypto.randomBytes(4).toString('hex').toUpperCase();
      codes.push(`${code.slice(0, 4)}-${code.slice(4)}`);
    }
    return codes;
  }

  private verifyBackupCode(config: MFAConfig, code: string): boolean {
    if (!config.backupCodes) {
      return false;
    }

    const index = config.backupCodes.indexOf(code);
    if (index !== -1) {
      // Remove used backup code
      config.backupCodes.splice(index, 1);
      return true;
    }

    return false;
  }

  private async sendSMSCode(phoneNumber: string, code: string): Promise<void> {
    // In production, integrate with SMS provider (Twilio, AWS SNS, etc.)
    console.log(`SMS to ${phoneNumber}: Your verification code is ${code}`);
  }

  private async sendEmailCode(email: string, code: string): Promise<void> {
    // In production, integrate with email provider
    console.log(`Email to ${email}: Your verification code is ${code}`);
  }

  private async sendPushNotification(userId: string, challengeId: string): Promise<void> {
    // In production, integrate with push notification service
    console.log(`Push notification to user ${userId} for challenge ${challengeId}`);
  }

  private async sendSMSChallenge(userId: string): Promise<void> {
    const config = this.getConfig(userId, MFAMethod.SMS);
    if (!config || !config.phoneNumber) {
      throw new Error('SMS not configured');
    }

    const code = this.generateCode();
    await this.sendSMSCode(config.phoneNumber, code);
  }

  private async sendEmailChallenge(userId: string): Promise<void> {
    const config = this.getConfig(userId, MFAMethod.EMAIL);
    if (!config || !config.email) {
      throw new Error('Email not configured');
    }

    const code = this.generateCode();
    await this.sendEmailCode(config.email, code);
  }

  private addConfig(userId: string, config: MFAConfig): void {
    const configs = this.configs.get(userId) || [];
    const existing = configs.findIndex((c) => c.method === config.method);

    if (existing !== -1) {
      configs[existing] = config;
    } else {
      configs.push(config);
    }

    this.configs.set(userId, configs);
  }

  private getConfig(userId: string, method: MFAMethod): MFAConfig | undefined {
    const configs = this.configs.get(userId) || [];
    return configs.find((c) => c.method === method);
  }
}

// Export singleton instance
export const mfaManager = new MFAManager();
