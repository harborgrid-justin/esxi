/**
 * Certificate Manager - TLS/SSL Certificate Management
 * Certificate lifecycle management and validation
 */

import * as crypto from 'crypto';
import { nanoid } from 'nanoid';

export interface Certificate {
  id: string;
  commonName: string;
  organization?: string;
  validFrom: Date;
  validTo: Date;
  serialNumber: string;
  fingerprint: string;
  publicKey: string;
  privateKey?: string;
  chain?: string[];
  status: 'ACTIVE' | 'EXPIRING' | 'EXPIRED' | 'REVOKED';
  createdAt: Date;
}

export class CertificateManager {
  private certificates: Map<string, Certificate> = new Map();

  /**
   * Generate self-signed certificate
   */
  async generateSelfSigned(commonName: string, validDays: number = 365): Promise<Certificate> {
    const { publicKey, privateKey } = crypto.generateKeyPairSync('rsa', {
      modulusLength: 2048,
    });

    const validFrom = new Date();
    const validTo = new Date(validFrom.getTime() + validDays * 24 * 60 * 60 * 1000);

    const cert: Certificate = {
      id: nanoid(),
      commonName,
      validFrom,
      validTo,
      serialNumber: crypto.randomBytes(16).toString('hex'),
      fingerprint: crypto.randomBytes(32).toString('hex'),
      publicKey: publicKey.export({ type: 'spki', format: 'pem' }).toString(),
      privateKey: privateKey.export({ type: 'pkcs8', format: 'pem' }).toString(),
      status: 'ACTIVE',
      createdAt: new Date(),
    };

    this.certificates.set(cert.id, cert);
    return cert;
  }

  /**
   * Get certificate
   */
  getCertificate(id: string): Certificate | undefined {
    return this.certificates.get(id);
  }

  /**
   * Validate certificate
   */
  validate(id: string): boolean {
    const cert = this.certificates.get(id);
    if (!cert) return false;

    const now = new Date();
    return cert.status === 'ACTIVE' && now >= cert.validFrom && now <= cert.validTo;
  }

  /**
   * Revoke certificate
   */
  revoke(id: string): void {
    const cert = this.certificates.get(id);
    if (cert) {
      cert.status = 'REVOKED';
    }
  }
}

export const certificateManager = new CertificateManager();
