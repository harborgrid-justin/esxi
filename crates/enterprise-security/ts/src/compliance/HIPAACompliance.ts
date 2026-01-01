/**
 * HIPAA Compliance - Health Insurance Portability and Accountability Act
 * Administrative, Physical, and Technical Safeguards
 */

import { ComplianceStatus, ComplianceControl } from '../types';
import { nanoid } from 'nanoid';

export class HIPAACompliance {
  private controls: Map<string, ComplianceControl> = new Map();

  constructor() {
    this.initializeControls();
  }

  private initializeControls(): void {
    this.addControl('164.308(a)(1)', 'Security Management Process', 'Administrative');
    this.addControl('164.308(a)(3)', 'Workforce Security', 'Administrative');
    this.addControl('164.308(a)(4)', 'Information Access Management', 'Administrative');
    this.addControl('164.310(a)(1)', 'Facility Access Controls', 'Physical');
    this.addControl('164.310(d)(1)', 'Device and Media Controls', 'Physical');
    this.addControl('164.312(a)(1)', 'Access Control', 'Technical');
    this.addControl('164.312(b)', 'Audit Controls', 'Technical');
    this.addControl('164.312(c)(1)', 'Integrity', 'Technical');
    this.addControl('164.312(d)', 'Person or Entity Authentication', 'Technical');
    this.addControl('164.312(e)(1)', 'Transmission Security', 'Technical');
  }

  private addControl(code: string, title: string, category: string): void {
    const control: ComplianceControl = {
      id: nanoid(),
      code,
      title,
      description: `HIPAA ${category} Safeguard: ${title}`,
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

  getControl(code: string): ComplianceControl | undefined {
    return this.controls.get(code);
  }

  async assessControl(code: string, status: ComplianceStatus, findings: string[]): Promise<void> {
    const control = this.controls.get(code);
    if (control) {
      control.status = status;
      control.findings = findings;
      control.lastAssessed = new Date();
    }
  }
}

export const hipaaCompliance = new HIPAACompliance();
