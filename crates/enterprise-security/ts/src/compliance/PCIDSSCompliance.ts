/**
 * PCI DSS Compliance - Payment Card Industry Data Security Standard
 * Cardholder data protection requirements
 */

import { ComplianceStatus, ComplianceControl } from '../types';
import { nanoid } from 'nanoid';

export class PCIDSSCompliance {
  private controls: Map<string, ComplianceControl> = new Map();

  constructor() {
    this.initializeControls();
  }

  private initializeControls(): void {
    // Build and Maintain a Secure Network
    this.addControl('1.1', 'Firewall Configuration Standards', 'Network Security');
    this.addControl('2.1', 'Vendor-Supplied Defaults', 'System Configuration');

    // Protect Cardholder Data
    this.addControl('3.4', 'Cardholder Data Encryption', 'Data Protection');
    this.addControl('4.1', 'Encryption for Transmission', 'Data Transmission');

    // Maintain Vulnerability Management
    this.addControl('5.1', 'Anti-Malware Software', 'Malware Protection');
    this.addControl('6.2', 'Security Patches', 'Patch Management');

    // Implement Strong Access Control
    this.addControl('7.1', 'Access Control Policies', 'Access Management');
    this.addControl('8.2', 'User Authentication', 'Authentication');
    this.addControl('8.3', 'Multi-Factor Authentication', 'MFA');

    // Monitor and Test Networks
    this.addControl('10.1', 'Audit Trails', 'Logging');
    this.addControl('11.2', 'Vulnerability Scans', 'Security Testing');

    // Maintain Information Security Policy
    this.addControl('12.1', 'Security Policy', 'Policy Management');
  }

  private addControl(code: string, title: string, category: string): void {
    const control: ComplianceControl = {
      id: nanoid(),
      code,
      title,
      description: `PCI DSS Requirement ${code}: ${title}`,
      category,
      status: ComplianceStatus.UNDER_REVIEW,
      evidence: [],
      lastAssessed: new Date(),
      assessedBy: 'system',
      findings: [],
    };
    this.controls.set(code, control);
  }

  getAllControls(): ComplianceControl[] {
    return Array.from(this.controls.values());
  }
}

export const pciDSSCompliance = new PCIDSSCompliance();
