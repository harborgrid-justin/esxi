/**
 * Biometric Authentication
 * WebAuthn/FIDO2 biometric authentication support
 */

import { nanoid } from 'nanoid';
import * as crypto from 'crypto';

// ============================================================================
// Types
// ============================================================================

export enum BiometricType {
  FINGERPRINT = 'FINGERPRINT',
  FACE_RECOGNITION = 'FACE_RECOGNITION',
  IRIS_SCAN = 'IRIS_SCAN',
  VOICE_RECOGNITION = 'VOICE_RECOGNITION',
}

export enum AuthenticatorType {
  PLATFORM = 'PLATFORM', // Built-in (Touch ID, Face ID, Windows Hello)
  CROSS_PLATFORM = 'CROSS_PLATFORM', // External (USB key, NFC)
}

export interface BiometricCredential {
  id: string;
  userId: string;
  credentialId: string;
  publicKey: string;
  counter: number;
  type: BiometricType;
  authenticatorType: AuthenticatorType;
  transports?: AuthenticatorTransport[];
  createdAt: Date;
  lastUsedAt?: Date;
  deviceName?: string;
  metadata: Record<string, unknown>;
}

export type AuthenticatorTransport = 'usb' | 'nfc' | 'ble' | 'internal';

export interface RegistrationOptions {
  challenge: string;
  rp: {
    name: string;
    id: string;
  };
  user: {
    id: string;
    name: string;
    displayName: string;
  };
  pubKeyCredParams: PublicKeyCredentialParameters[];
  timeout?: number;
  attestation?: AttestationConveyancePreference;
  authenticatorSelection?: AuthenticatorSelectionCriteria;
}

export interface PublicKeyCredentialParameters {
  type: 'public-key';
  alg: number; // COSE algorithm identifier
}

export type AttestationConveyancePreference = 'none' | 'indirect' | 'direct' | 'enterprise';

export interface AuthenticatorSelectionCriteria {
  authenticatorAttachment?: 'platform' | 'cross-platform';
  requireResidentKey?: boolean;
  residentKey?: 'discouraged' | 'preferred' | 'required';
  userVerification?: 'required' | 'preferred' | 'discouraged';
}

export interface AuthenticationOptions {
  challenge: string;
  timeout?: number;
  rpId?: string;
  allowCredentials?: PublicKeyCredentialDescriptor[];
  userVerification?: 'required' | 'preferred' | 'discouraged';
}

export interface PublicKeyCredentialDescriptor {
  type: 'public-key';
  id: string;
  transports?: AuthenticatorTransport[];
}

export interface RegistrationResponse {
  credentialId: string;
  publicKey: string;
  attestationObject: string;
  clientDataJSON: string;
}

export interface AuthenticationResponse {
  credentialId: string;
  authenticatorData: string;
  signature: string;
  clientDataJSON: string;
  userHandle?: string;
}

// ============================================================================
// Biometric Auth Implementation
// ============================================================================

export class BiometricAuth {
  private credentials: Map<string, BiometricCredential> = new Map();
  private userCredentials: Map<string, Set<string>> = new Map();
  private challenges: Map<string, { challenge: string; expiresAt: Date }> = new Map();
  private readonly rpName = 'HarborGrid Enterprise';
  private readonly rpId = 'harborgrid.com';

  /**
   * Generate registration options for WebAuthn
   */
  async generateRegistrationOptions(
    userId: string,
    username: string,
    displayName: string,
    authenticatorType?: AuthenticatorType
  ): Promise<RegistrationOptions> {
    const challenge = this.generateChallenge();

    // Store challenge
    this.challenges.set(challenge, {
      challenge,
      expiresAt: new Date(Date.now() + 5 * 60 * 1000), // 5 minutes
    });

    const options: RegistrationOptions = {
      challenge,
      rp: {
        name: this.rpName,
        id: this.rpId,
      },
      user: {
        id: userId,
        name: username,
        displayName,
      },
      pubKeyCredParams: [
        { type: 'public-key', alg: -7 }, // ES256
        { type: 'public-key', alg: -257 }, // RS256
      ],
      timeout: 60000,
      attestation: 'none',
      authenticatorSelection: {
        authenticatorAttachment:
          authenticatorType === AuthenticatorType.PLATFORM ? 'platform' : 'cross-platform',
        requireResidentKey: false,
        userVerification: 'preferred',
      },
    };

    return options;
  }

  /**
   * Register biometric credential
   */
  async registerCredential(
    userId: string,
    response: RegistrationResponse,
    challenge: string,
    deviceName?: string
  ): Promise<BiometricCredential> {
    // Verify challenge
    if (!this.verifyChallenge(challenge)) {
      throw new Error('Invalid or expired challenge');
    }

    // Parse and verify attestation
    const attestation = this.parseAttestationObject(response.attestationObject);

    // Create credential
    const credential: BiometricCredential = {
      id: nanoid(),
      userId,
      credentialId: response.credentialId,
      publicKey: response.publicKey,
      counter: 0,
      type: BiometricType.FINGERPRINT, // Default, could be detected from attestation
      authenticatorType: AuthenticatorType.PLATFORM,
      createdAt: new Date(),
      deviceName,
      metadata: {
        attestation,
      },
    };

    // Store credential
    this.credentials.set(credential.credentialId, credential);

    // Track user credentials
    const userCreds = this.userCredentials.get(userId) || new Set();
    userCreds.add(credential.credentialId);
    this.userCredentials.set(userId, userCreds);

    // Clean up challenge
    this.challenges.delete(challenge);

    return credential;
  }

  /**
   * Generate authentication options
   */
  async generateAuthenticationOptions(userId?: string): Promise<AuthenticationOptions> {
    const challenge = this.generateChallenge();

    // Store challenge
    this.challenges.set(challenge, {
      challenge,
      expiresAt: new Date(Date.now() + 5 * 60 * 1000),
    });

    const options: AuthenticationOptions = {
      challenge,
      timeout: 60000,
      rpId: this.rpId,
      userVerification: 'preferred',
    };

    // If userId provided, include their credentials
    if (userId) {
      const userCreds = this.getUserCredentials(userId);
      options.allowCredentials = userCreds.map((cred) => ({
        type: 'public-key' as const,
        id: cred.credentialId,
        transports: cred.transports,
      }));
    }

    return options;
  }

  /**
   * Verify authentication
   */
  async verifyAuthentication(
    response: AuthenticationResponse,
    challenge: string
  ): Promise<BiometricCredential | null> {
    // Verify challenge
    if (!this.verifyChallenge(challenge)) {
      throw new Error('Invalid or expired challenge');
    }

    // Get credential
    const credential = this.credentials.get(response.credentialId);
    if (!credential) {
      return null;
    }

    // Verify signature
    const isValid = this.verifySignature(
      credential.publicKey,
      response.authenticatorData,
      response.signature,
      response.clientDataJSON
    );

    if (!isValid) {
      return null;
    }

    // Update credential
    credential.counter++;
    credential.lastUsedAt = new Date();

    // Clean up challenge
    this.challenges.delete(challenge);

    return credential;
  }

  /**
   * Get user credentials
   */
  getUserCredentials(userId: string): BiometricCredential[] {
    const credIds = this.userCredentials.get(userId);
    if (!credIds) {
      return [];
    }

    const credentials: BiometricCredential[] = [];
    for (const credId of credIds) {
      const cred = this.credentials.get(credId);
      if (cred) {
        credentials.push(cred);
      }
    }

    return credentials;
  }

  /**
   * Remove credential
   */
  async removeCredential(credentialId: string): Promise<void> {
    const credential = this.credentials.get(credentialId);
    if (!credential) {
      return;
    }

    // Remove from maps
    this.credentials.delete(credentialId);

    const userCreds = this.userCredentials.get(credential.userId);
    if (userCreds) {
      userCreds.delete(credentialId);
      if (userCreds.size === 0) {
        this.userCredentials.delete(credential.userId);
      }
    }
  }

  /**
   * Remove all user credentials
   */
  async removeUserCredentials(userId: string): Promise<void> {
    const credIds = this.userCredentials.get(userId);
    if (!credIds) {
      return;
    }

    for (const credId of Array.from(credIds)) {
      await this.removeCredential(credId);
    }
  }

  /**
   * Check if user has biometric enabled
   */
  hasBiometricEnabled(userId: string): boolean {
    const creds = this.getUserCredentials(userId);
    return creds.length > 0;
  }

  /**
   * Get credential by ID
   */
  getCredential(credentialId: string): BiometricCredential | undefined {
    return this.credentials.get(credentialId);
  }

  /**
   * Update credential metadata
   */
  async updateCredentialMetadata(
    credentialId: string,
    metadata: Record<string, unknown>
  ): Promise<void> {
    const credential = this.credentials.get(credentialId);
    if (!credential) {
      throw new Error('Credential not found');
    }

    credential.metadata = {
      ...credential.metadata,
      ...metadata,
    };
  }

  // ============================================================================
  // Private Helper Methods
  // ============================================================================

  private generateChallenge(): string {
    return crypto.randomBytes(32).toString('base64url');
  }

  private verifyChallenge(challenge: string): boolean {
    const stored = this.challenges.get(challenge);
    if (!stored) {
      return false;
    }

    // Check expiration
    if (new Date() > stored.expiresAt) {
      this.challenges.delete(challenge);
      return false;
    }

    return true;
  }

  private parseAttestationObject(attestationObject: string): Record<string, unknown> {
    // Simplified - production would use CBOR parser
    return {
      fmt: 'none',
      attStmt: {},
      authData: attestationObject,
    };
  }

  private verifySignature(
    publicKey: string,
    authenticatorData: string,
    signature: string,
    clientDataJSON: string
  ): boolean {
    // Simplified verification - production would:
    // 1. Hash clientDataJSON
    // 2. Concatenate authenticatorData + hash
    // 3. Verify signature using public key

    try {
      // Mock verification
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Clean up expired challenges
   */
  async cleanupExpiredChallenges(): Promise<number> {
    let cleaned = 0;
    const now = new Date();

    for (const [challenge, data] of this.challenges.entries()) {
      if (data.expiresAt < now) {
        this.challenges.delete(challenge);
        cleaned++;
      }
    }

    return cleaned;
  }

  /**
   * Get statistics
   */
  getStats(): {
    totalCredentials: number;
    usersWithBiometric: number;
    averageCredentialsPerUser: number;
  } {
    return {
      totalCredentials: this.credentials.size,
      usersWithBiometric: this.userCredentials.size,
      averageCredentialsPerUser:
        this.userCredentials.size > 0
          ? this.credentials.size / this.userCredentials.size
          : 0,
    };
  }
}

// Export singleton instance
export const biometricAuth = new BiometricAuth();
