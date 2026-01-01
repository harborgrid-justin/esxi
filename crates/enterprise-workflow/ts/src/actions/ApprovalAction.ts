/**
 * Approval Action - Human approval workflows
 */

import { EventEmitter } from 'eventemitter3';
import { v4 as uuidv4 } from 'uuid';
import { ApprovalActionConfig, Context } from '../types';

export interface ApprovalRequest {
  id: string;
  approvers: string[];
  approvalType: 'any' | 'all' | 'majority';
  message: string;
  deadline?: Date;
  createdAt: Date;
  status: 'pending' | 'approved' | 'rejected' | 'expired';
  responses: ApprovalResponse[];
  metadata?: Record<string, any>;
}

export interface ApprovalResponse {
  approverId: string;
  approved: boolean;
  comment?: string;
  timestamp: Date;
}

export class ApprovalAction extends EventEmitter {
  private approvalRequests: Map<string, ApprovalRequest>;
  private expirationTimers: Map<string, NodeJS.Timeout>;

  constructor() {
    super();
    this.approvalRequests = new Map();
    this.expirationTimers = new Map();
  }

  /**
   * Execute approval action
   */
  async execute(config: ApprovalActionConfig, context: Context): Promise<any> {
    const requestId = uuidv4();

    // Create approval request
    const request: ApprovalRequest = {
      id: requestId,
      approvers: config.approvers,
      approvalType: config.approvalType,
      message: this.interpolateString(config.message, context),
      deadline: config.deadline,
      createdAt: new Date(),
      status: 'pending',
      responses: [],
      metadata: config.metadata
    };

    this.approvalRequests.set(requestId, request);

    // Set up deadline timer if configured
    if (config.deadline) {
      const timeUntilDeadline = config.deadline.getTime() - Date.now();
      if (timeUntilDeadline > 0) {
        const timer = setTimeout(() => {
          this.expireApproval(requestId);
        }, timeUntilDeadline);

        this.expirationTimers.set(requestId, timer);
      }
    }

    // Emit approval request event
    this.emit('approval:requested', {
      requestId,
      request,
      context
    });

    // In a real implementation, this would:
    // 1. Send notifications to approvers
    // 2. Create UI for approval
    // 3. Wait for responses
    // For now, return the request

    return {
      requestId,
      status: 'pending',
      approvers: request.approvers,
      deadline: request.deadline,
      message: request.message
    };
  }

  /**
   * Submit approval response
   */
  async respond(
    requestId: string,
    approverId: string,
    approved: boolean,
    comment?: string
  ): Promise<{ success: boolean; finalDecision?: boolean; error?: string }> {
    const request = this.approvalRequests.get(requestId);

    if (!request) {
      return {
        success: false,
        error: 'Approval request not found'
      };
    }

    if (request.status !== 'pending') {
      return {
        success: false,
        error: `Approval request is ${request.status}`
      };
    }

    // Check if approver is authorized
    if (!request.approvers.includes(approverId)) {
      return {
        success: false,
        error: 'User is not an authorized approver'
      };
    }

    // Check if approver already responded
    const existingResponse = request.responses.find(r => r.approverId === approverId);
    if (existingResponse) {
      return {
        success: false,
        error: 'Approver has already responded'
      };
    }

    // Add response
    const response: ApprovalResponse = {
      approverId,
      approved,
      comment,
      timestamp: new Date()
    };

    request.responses.push(response);

    this.emit('approval:response', {
      requestId,
      response,
      request
    });

    // Check if approval is complete
    const decision = this.evaluateApproval(request);

    if (decision !== null) {
      this.finalizeApproval(requestId, decision);

      return {
        success: true,
        finalDecision: decision
      };
    }

    return {
      success: true
    };
  }

  /**
   * Evaluate if approval is complete
   */
  private evaluateApproval(request: ApprovalRequest): boolean | null {
    const { approvalType, approvers, responses } = request;

    const approvedCount = responses.filter(r => r.approved).length;
    const rejectedCount = responses.filter(r => !r.approved).length;
    const totalApprovers = approvers.length;

    switch (approvalType) {
      case 'any':
        // Any single approval is sufficient
        if (approvedCount > 0) return true;
        // If all have responded and none approved
        if (responses.length === totalApprovers && approvedCount === 0) return false;
        break;

      case 'all':
        // All must approve
        if (rejectedCount > 0) return false;
        if (approvedCount === totalApprovers) return true;
        break;

      case 'majority':
        // More than half must approve
        const majorityThreshold = Math.floor(totalApprovers / 2) + 1;
        if (approvedCount >= majorityThreshold) return true;
        // If remaining approvers can't reach majority
        const remainingApprovers = totalApprovers - responses.length;
        if (approvedCount + remainingApprovers < majorityThreshold) return false;
        break;
    }

    return null; // Not yet decided
  }

  /**
   * Finalize approval decision
   */
  private finalizeApproval(requestId: string, approved: boolean): void {
    const request = this.approvalRequests.get(requestId);
    if (!request) return;

    request.status = approved ? 'approved' : 'rejected';

    // Clear expiration timer
    const timer = this.expirationTimers.get(requestId);
    if (timer) {
      clearTimeout(timer);
      this.expirationTimers.delete(requestId);
    }

    this.emit(approved ? 'approval:granted' : 'approval:rejected', {
      requestId,
      request
    });
  }

  /**
   * Expire approval request
   */
  private expireApproval(requestId: string): void {
    const request = this.approvalRequests.get(requestId);
    if (!request) return;

    if (request.status === 'pending') {
      request.status = 'expired';

      this.emit('approval:expired', {
        requestId,
        request
      });
    }
  }

  /**
   * Get approval request
   */
  getRequest(requestId: string): ApprovalRequest | undefined {
    return this.approvalRequests.get(requestId);
  }

  /**
   * Get pending requests for approver
   */
  getPendingRequests(approverId: string): ApprovalRequest[] {
    return Array.from(this.approvalRequests.values()).filter(request =>
      request.status === 'pending' &&
      request.approvers.includes(approverId) &&
      !request.responses.some(r => r.approverId === approverId)
    );
  }

  /**
   * Get all requests
   */
  getAllRequests(): ApprovalRequest[] {
    return Array.from(this.approvalRequests.values());
  }

  /**
   * Cancel approval request
   */
  cancel(requestId: string): boolean {
    const request = this.approvalRequests.get(requestId);
    if (!request || request.status !== 'pending') {
      return false;
    }

    request.status = 'rejected';

    // Clear timer
    const timer = this.expirationTimers.get(requestId);
    if (timer) {
      clearTimeout(timer);
      this.expirationTimers.delete(requestId);
    }

    this.emit('approval:cancelled', {
      requestId,
      request
    });

    return true;
  }

  /**
   * Interpolate string with context variables
   */
  private interpolateString(str: string, context: Context): string {
    return str.replace(/\${([^}]+)}/g, (match, varName) => {
      const value = context.variables.get(varName.trim());
      return value !== undefined ? String(value) : match;
    });
  }

  /**
   * Validate approval action configuration
   */
  validate(config: ApprovalActionConfig): string[] {
    const errors: string[] = [];

    if (!config.approvers || config.approvers.length === 0) {
      errors.push('At least one approver is required');
    }

    if (!config.approvalType) {
      errors.push('Approval type is required');
    } else if (!['any', 'all', 'majority'].includes(config.approvalType)) {
      errors.push('Invalid approval type');
    }

    if (!config.message) {
      errors.push('Approval message is required');
    }

    if (config.deadline && config.deadline <= new Date()) {
      errors.push('Deadline must be in the future');
    }

    return errors;
  }

  /**
   * Clean up expired requests
   */
  cleanup(maxAge: number = 86400000): void { // Default 24 hours
    const now = Date.now();

    this.approvalRequests.forEach((request, requestId) => {
      if (request.status !== 'pending') {
        const age = now - request.createdAt.getTime();
        if (age > maxAge) {
          // Clear timer if exists
          const timer = this.expirationTimers.get(requestId);
          if (timer) {
            clearTimeout(timer);
            this.expirationTimers.delete(requestId);
          }

          this.approvalRequests.delete(requestId);
        }
      }
    });
  }
}
