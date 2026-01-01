/**
 * Simple Workflow Example
 *
 * This example demonstrates creating and executing a simple approval workflow
 */

import {
  WorkflowService,
  ExecutionService,
  Workflow,
  WorkflowStatus,
  ActionType
} from '../src';

async function main() {
  console.log('Creating a simple approval workflow...\n');

  // Initialize services
  const workflowService = new WorkflowService();
  const executionService = new ExecutionService();

  // Create workflow
  const workflow: Omit<Workflow, 'id' | 'createdAt' | 'updatedAt'> = {
    name: 'Document Approval Workflow',
    description: 'Simple document approval with notifications',
    version: '1.0.0',
    status: WorkflowStatus.ACTIVE,
    category: 'Business Process',
    tags: ['approval', 'document'],

    // Define workflow variables
    variables: [
      {
        id: 'var-1',
        name: 'document_name',
        type: 'string',
        value: '',
        required: true,
        description: 'Name of the document to approve'
      },
      {
        id: 'var-2',
        name: 'approver_email',
        type: 'string',
        value: '',
        required: true,
        description: 'Email of the approver'
      },
      {
        id: 'var-3',
        name: 'requester_email',
        type: 'string',
        value: '',
        required: true,
        description: 'Email of the requester'
      }
    ],

    // Define workflow steps
    steps: [
      {
        id: 'send-approval-request',
        name: 'Send Approval Request',
        description: 'Send email to approver requesting approval',
        type: 'action',
        action: {
          id: 'email-1',
          name: 'Request Approval Email',
          type: ActionType.EMAIL,
          config: {
            to: '${approver_email}',
            subject: 'Approval Required: ${document_name}',
            body: 'Please review and approve the document: ${document_name}'
          }
        },
        position: { x: 100, y: 100 },
        transitions: [
          {
            id: 'trans-1',
            from: 'send-approval-request',
            to: 'wait-for-approval'
          }
        ]
      },
      {
        id: 'wait-for-approval',
        name: 'Wait for Approval',
        description: 'Wait for human approval',
        type: 'action',
        action: {
          id: 'approval-1',
          name: 'Document Approval',
          type: ActionType.APPROVAL,
          config: {
            approvers: ['${approver_email}'],
            approvalType: 'any',
            message: 'Please approve document: ${document_name}',
            deadline: new Date(Date.now() + 86400000) // 24 hours
          }
        },
        position: { x: 100, y: 200 },
        transitions: [
          {
            id: 'trans-2',
            from: 'wait-for-approval',
            to: 'check-approval'
          }
        ]
      },
      {
        id: 'check-approval',
        name: 'Check Approval Status',
        description: 'Check if approval was granted',
        type: 'condition',
        condition: {
          id: 'cond-1',
          type: 'simple',
          operator: 'equals',
          left: '${approval_status}',
          right: 'approved'
        },
        position: { x: 100, y: 300 },
        transitions: [
          {
            id: 'trans-3',
            from: 'check-approval',
            to: 'send-approved-notification',
            label: 'true'
          },
          {
            id: 'trans-4',
            from: 'check-approval',
            to: 'send-rejected-notification',
            label: 'false'
          }
        ]
      },
      {
        id: 'send-approved-notification',
        name: 'Send Approved Notification',
        description: 'Notify requester of approval',
        type: 'action',
        action: {
          id: 'email-2',
          name: 'Approval Notification',
          type: ActionType.EMAIL,
          config: {
            to: '${requester_email}',
            subject: 'Approved: ${document_name}',
            body: 'Your document has been approved!'
          }
        },
        position: { x: 50, y: 400 },
        transitions: []
      },
      {
        id: 'send-rejected-notification',
        name: 'Send Rejected Notification',
        description: 'Notify requester of rejection',
        type: 'action',
        action: {
          id: 'email-3',
          name: 'Rejection Notification',
          type: ActionType.EMAIL,
          config: {
            to: '${requester_email}',
            subject: 'Rejected: ${document_name}',
            body: 'Your document was not approved.'
          }
        },
        position: { x: 250, y: 400 },
        transitions: []
      }
    ],

    // Define workflow triggers
    triggers: [
      {
        id: 'manual-trigger',
        type: 'manual',
        enabled: true,
        config: {
          requiredRoles: ['user', 'admin'],
          confirmationRequired: false
        }
      }
    ],

    startStepId: 'send-approval-request',
    endStepIds: ['send-approved-notification', 'send-rejected-notification'],
    createdBy: 'system',

    settings: {
      timeout: 3600000,
      errorHandling: 'fail',
      logging: {
        level: 'info'
      },
      notifications: {
        onSuccess: true,
        onFailure: true,
        recipients: ['admin@example.com']
      }
    }
  };

  // Create the workflow
  const createResult = await workflowService.create(workflow);

  if (!createResult.success) {
    console.error('Failed to create workflow:', createResult.error);
    return;
  }

  console.log('Workflow created successfully!');
  console.log('Workflow ID:', createResult.data?.id);
  console.log('Workflow Name:', createResult.data?.name);
  console.log('');

  // Validate the workflow
  const validationResult = await workflowService.validate(createResult.data!);
  console.log('Validation result:', validationResult.data);
  console.log('');

  // Execute the workflow
  console.log('Executing workflow...');
  const executionResult = await executionService.execute(
    createResult.data!,
    {
      variables: new Map([
        ['document_name', 'Q4 Financial Report'],
        ['approver_email', 'manager@example.com'],
        ['requester_email', 'employee@example.com']
      ])
    },
    'manual'
  );

  if (!executionResult.success) {
    console.error('Failed to execute workflow:', executionResult.error);
    return;
  }

  console.log('Execution started!');
  console.log('Execution ID:', executionResult.data?.id);
  console.log('Status:', executionResult.data?.status);
  console.log('');

  // Get execution statistics
  const statsResult = await executionService.getStatistics(createResult.data?.id);
  console.log('Workflow statistics:', statsResult.data);
}

// Run the example
main().catch(console.error);
