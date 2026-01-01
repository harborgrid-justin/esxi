/**
 * GDPR Compliance - General Data Protection Regulation
 * Data subject rights and privacy controls
 */

import { ComplianceStatus, ComplianceControl } from '../types';
import { nanoid } from 'nanoid';

export interface DataSubjectRequest {
  id: string;
  type: 'ACCESS' | 'RECTIFICATION' | 'ERASURE' | 'PORTABILITY' | 'RESTRICTION' | 'OBJECTION';
  subjectId: string;
  status: 'PENDING' | 'IN_PROGRESS' | 'COMPLETED' | 'REJECTED';
  requestedAt: Date;
  completedAt?: Date;
  reason?: string;
}

export class GDPRCompliance {
  private controls: Map<string, ComplianceControl> = new Map();
  private requests: Map<string, DataSubjectRequest> = new Map();

  constructor() {
    this.initializeControls();
  }

  private initializeControls(): void {
    this.addControl('Art. 5', 'Principles of Data Processing', 'Data Protection');
    this.addControl('Art. 6', 'Lawfulness of Processing', 'Legal Basis');
    this.addControl('Art. 15', 'Right of Access', 'Data Subject Rights');
    this.addControl('Art. 16', 'Right to Rectification', 'Data Subject Rights');
    this.addControl('Art. 17', 'Right to Erasure', 'Data Subject Rights');
    this.addControl('Art. 20', 'Right to Data Portability', 'Data Subject Rights');
    this.addControl('Art. 25', 'Data Protection by Design', 'Privacy by Design');
    this.addControl('Art. 32', 'Security of Processing', 'Security');
    this.addControl('Art. 33', 'Breach Notification', 'Incident Response');
    this.addControl('Art. 35', 'Data Protection Impact Assessment', 'Risk Management');
  }

  private addControl(code: string, title: string, category: string): void {
    const control: ComplianceControl = {
      id: nanoid(),
      code,
      title,
      description: `GDPR ${code}: ${title}`,
      category,
      status: ComplianceStatus.UNDER_REVIEW,
      evidence: [],
      lastAssessed: new Date(),
      assessedBy: 'system',
      findings: [],
    };
    this.controls.set(code, control);
  }

  /**
   * Submit data subject request
   */
  async submitRequest(type: DataSubjectRequest['type'], subjectId: string): Promise<DataSubjectRequest> {
    const request: DataSubjectRequest = {
      id: nanoid(),
      type,
      subjectId,
      status: 'PENDING',
      requestedAt: new Date(),
    };
    this.requests.set(request.id, request);
    return request;
  }

  /**
   * Process data subject request
   */
  async processRequest(requestId: string, completed: boolean, reason?: string): Promise<void> {
    const request = this.requests.get(requestId);
    if (request) {
      request.status = completed ? 'COMPLETED' : 'REJECTED';
      request.completedAt = new Date();
      request.reason = reason;
    }
  }

  getAllControls(): ComplianceControl[] {
    return Array.from(this.controls.values());
  }

  getAllRequests(): DataSubjectRequest[] {
    return Array.from(this.requests.values());
  }
}

export const gdprCompliance = new GDPRCompliance();
