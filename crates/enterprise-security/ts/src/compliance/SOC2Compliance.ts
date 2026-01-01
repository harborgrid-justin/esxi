/**
 * SOC 2 Compliance - Trust Service Criteria
 * Security, Availability, Processing Integrity, Confidentiality, Privacy
 */

import { ComplianceStatus, ComplianceControl } from '../types';
import { nanoid } from 'nanoid';

export interface SOC2Control extends ComplianceControl {
  trustServiceCriteria: 'CC' | 'A' | 'PI' | 'C' | 'P';
  controlType: 'PREVENTIVE' | 'DETECTIVE' | 'CORRECTIVE';
  frequency: 'CONTINUOUS' | 'DAILY' | 'WEEKLY' | 'MONTHLY' | 'QUARTERLY' | 'ANNUAL';
}

export class SOC2Compliance {
  private controls: Map<string, SOC2Control> = new Map();

  constructor() {
    this.initializeControls();
  }

  /**
   * Initialize SOC 2 controls
   */
  private initializeControls(): void {
    // Common Criteria (CC) - Security
    this.addControl({
      code: 'CC6.1',
      title: 'Logical and Physical Access Controls',
      trustServiceCriteria: 'CC',
      controlType: 'PREVENTIVE',
      frequency: 'CONTINUOUS',
    });

    this.addControl({
      code: 'CC6.6',
      title: 'Encryption of Data at Rest and in Transit',
      trustServiceCriteria: 'CC',
      controlType: 'PREVENTIVE',
      frequency: 'CONTINUOUS',
    });

    this.addControl({
      code: 'CC7.2',
      title: 'System Monitoring',
      trustServiceCriteria: 'CC',
      controlType: 'DETECTIVE',
      frequency: 'CONTINUOUS',
    });

    // Availability (A)
    this.addControl({
      code: 'A1.2',
      title: 'System Availability and Performance',
      trustServiceCriteria: 'A',
      controlType: 'PREVENTIVE',
      frequency: 'CONTINUOUS',
    });

    // Processing Integrity (PI)
    this.addControl({
      code: 'PI1.4',
      title: 'Data Processing Accuracy',
      trustServiceCriteria: 'PI',
      controlType: 'DETECTIVE',
      frequency: 'DAILY',
    });

    // Confidentiality (C)
    this.addControl({
      code: 'C1.1',
      title: 'Confidential Information Protection',
      trustServiceCriteria: 'C',
      controlType: 'PREVENTIVE',
      frequency: 'CONTINUOUS',
    });

    // Privacy (P)
    this.addControl({
      code: 'P4.2',
      title: 'Data Retention and Disposal',
      trustServiceCriteria: 'P',
      controlType: 'CORRECTIVE',
      frequency: 'MONTHLY',
    });
  }

  private addControl(partial: Omit<SOC2Control, 'id' | 'description' | 'category' | 'status' | 'evidence' | 'lastAssessed' | 'assessedBy' | 'findings'>): void {
    const control: SOC2Control = {
      id: nanoid(),
      code: partial.code,
      title: partial.title,
      description: `SOC 2 Control: ${partial.title}`,
      category: `SOC2-${partial.trustServiceCriteria}`,
      status: ComplianceStatus.UNDER_REVIEW,
      evidence: [],
      lastAssessed: new Date(),
      assessedBy: 'system',
      findings: [],
      trustServiceCriteria: partial.trustServiceCriteria,
      controlType: partial.controlType,
      frequency: partial.frequency,
    };

    this.controls.set(control.code, control);
  }

  /**
   * Get all controls
   */
  getAllControls(): SOC2Control[] {
    return Array.from(this.controls.values());
  }

  /**
   * Get control by code
   */
  getControl(code: string): SOC2Control | undefined {
    return this.controls.get(code);
  }

  /**
   * Assess control
   */
  async assessControl(code: string, status: ComplianceStatus, findings: string[]): Promise<void> {
    const control = this.controls.get(code);
    if (control) {
      control.status = status;
      control.findings = findings;
      control.lastAssessed = new Date();
    }
  }

  /**
   * Get compliance summary
   */
  getSummary(): {
    total: number;
    compliant: number;
    nonCompliant: number;
    underReview: number;
    complianceRate: number;
  } {
    const controls = this.getAllControls();
    return {
      total: controls.length,
      compliant: controls.filter(c => c.status === ComplianceStatus.COMPLIANT).length,
      nonCompliant: controls.filter(c => c.status === ComplianceStatus.NON_COMPLIANT).length,
      underReview: controls.filter(c => c.status === ComplianceStatus.UNDER_REVIEW).length,
      complianceRate: controls.filter(c => c.status === ComplianceStatus.COMPLIANT).length / controls.length,
    };
  }
}

export const soc2Compliance = new SOC2Compliance();
