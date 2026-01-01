/**
 * Core Security & Compliance Types
 * Enterprise-grade type definitions for security, compliance, and governance
 */

import { z } from 'zod';

// ============================================================================
// Security Policy Types
// ============================================================================

export enum SecurityPolicyType {
  PASSWORD = 'PASSWORD',
  SESSION = 'SESSION',
  ACCESS_CONTROL = 'ACCESS_CONTROL',
  DATA_PROTECTION = 'DATA_PROTECTION',
  NETWORK = 'NETWORK',
  API = 'API',
  ENCRYPTION = 'ENCRYPTION',
}

export enum PolicyEnforcement {
  ENFORCE = 'ENFORCE',
  AUDIT = 'AUDIT',
  DISABLED = 'DISABLED',
}

export interface SecurityPolicy {
  id: string;
  name: string;
  type: SecurityPolicyType;
  enforcement: PolicyEnforcement;
  rules: PolicyRule[];
  metadata: Record<string, unknown>;
  createdAt: Date;
  updatedAt: Date;
  createdBy: string;
  version: number;
}

export interface PolicyRule {
  id: string;
  condition: string;
  action: PolicyAction;
  priority: number;
  enabled: boolean;
}

export enum PolicyAction {
  ALLOW = 'ALLOW',
  DENY = 'DENY',
  REQUIRE_MFA = 'REQUIRE_MFA',
  LOG = 'LOG',
  ALERT = 'ALERT',
  BLOCK = 'BLOCK',
}

// ============================================================================
// Compliance Standards
// ============================================================================

export enum ComplianceFramework {
  SOC2 = 'SOC2',
  HIPAA = 'HIPAA',
  GDPR = 'GDPR',
  PCI_DSS = 'PCI_DSS',
  ISO27001 = 'ISO27001',
  NIST = 'NIST',
  CCPA = 'CCPA',
}

export enum ComplianceStatus {
  COMPLIANT = 'COMPLIANT',
  NON_COMPLIANT = 'NON_COMPLIANT',
  PARTIALLY_COMPLIANT = 'PARTIALLY_COMPLIANT',
  NOT_APPLICABLE = 'NOT_APPLICABLE',
  UNDER_REVIEW = 'UNDER_REVIEW',
}

export interface ComplianceStandard {
  id: string;
  framework: ComplianceFramework;
  version: string;
  controls: ComplianceControl[];
  status: ComplianceStatus;
  assessmentDate: Date;
  nextReviewDate: Date;
  auditor?: string;
  certificationDate?: Date;
  expirationDate?: Date;
}

export interface ComplianceControl {
  id: string;
  code: string;
  title: string;
  description: string;
  category: string;
  status: ComplianceStatus;
  evidence: Evidence[];
  lastAssessed: Date;
  assessedBy: string;
  findings: string[];
  remediationPlan?: string;
}

export interface Evidence {
  id: string;
  type: string;
  description: string;
  url?: string;
  collectedAt: Date;
  collectedBy: string;
  metadata: Record<string, unknown>;
}

// ============================================================================
// Audit Logging
// ============================================================================

export enum AuditEventType {
  // Authentication
  LOGIN_SUCCESS = 'LOGIN_SUCCESS',
  LOGIN_FAILURE = 'LOGIN_FAILURE',
  LOGOUT = 'LOGOUT',
  MFA_ENABLED = 'MFA_ENABLED',
  MFA_DISABLED = 'MFA_DISABLED',
  PASSWORD_CHANGED = 'PASSWORD_CHANGED',

  // Authorization
  ACCESS_GRANTED = 'ACCESS_GRANTED',
  ACCESS_DENIED = 'ACCESS_DENIED',
  PERMISSION_CHANGED = 'PERMISSION_CHANGED',
  ROLE_ASSIGNED = 'ROLE_ASSIGNED',
  ROLE_REVOKED = 'ROLE_REVOKED',

  // Data Operations
  DATA_ACCESSED = 'DATA_ACCESSED',
  DATA_CREATED = 'DATA_CREATED',
  DATA_UPDATED = 'DATA_UPDATED',
  DATA_DELETED = 'DATA_DELETED',
  DATA_EXPORTED = 'DATA_EXPORTED',

  // Security Events
  SECURITY_ALERT = 'SECURITY_ALERT',
  THREAT_DETECTED = 'THREAT_DETECTED',
  VULNERABILITY_FOUND = 'VULNERABILITY_FOUND',
  INCIDENT_CREATED = 'INCIDENT_CREATED',

  // Compliance
  COMPLIANCE_CHECK = 'COMPLIANCE_CHECK',
  POLICY_VIOLATION = 'POLICY_VIOLATION',
  AUDIT_STARTED = 'AUDIT_STARTED',
  AUDIT_COMPLETED = 'AUDIT_COMPLETED',
}

export enum AuditSeverity {
  CRITICAL = 'CRITICAL',
  HIGH = 'HIGH',
  MEDIUM = 'MEDIUM',
  LOW = 'LOW',
  INFO = 'INFO',
}

export interface AuditLog {
  id: string;
  eventType: AuditEventType;
  severity: AuditSeverity;
  timestamp: Date;
  userId?: string;
  username?: string;
  ipAddress?: string;
  userAgent?: string;
  resource: string;
  action: string;
  result: 'SUCCESS' | 'FAILURE' | 'PARTIAL';
  details: Record<string, unknown>;
  metadata: AuditMetadata;
  hash: string; // Immutability verification
}

export interface AuditMetadata {
  tenantId?: string;
  sessionId?: string;
  requestId?: string;
  correlationId?: string;
  geolocation?: {
    country?: string;
    region?: string;
    city?: string;
  };
  device?: {
    type: string;
    os?: string;
    browser?: string;
  };
}

// ============================================================================
// Encryption Types
// ============================================================================

export enum EncryptionAlgorithm {
  AES_256_GCM = 'AES_256_GCM',
  AES_256_CBC = 'AES_256_CBC',
  RSA_OAEP = 'RSA_OAEP',
  CHACHA20_POLY1305 = 'CHACHA20_POLY1305',
}

export enum KeyType {
  SYMMETRIC = 'SYMMETRIC',
  ASYMMETRIC = 'ASYMMETRIC',
  SIGNING = 'SIGNING',
}

export enum KeyStatus {
  ACTIVE = 'ACTIVE',
  ROTATING = 'ROTATING',
  DEPRECATED = 'DEPRECATED',
  REVOKED = 'REVOKED',
  DESTROYED = 'DESTROYED',
}

export interface Encryption {
  algorithm: EncryptionAlgorithm;
  keyId: string;
  iv: string;
  authTag?: string;
  encryptedData: string;
  metadata: {
    encryptedAt: Date;
    encryptedBy: string;
    version: number;
  };
}

export interface EncryptionKey {
  id: string;
  type: KeyType;
  algorithm: EncryptionAlgorithm;
  status: KeyStatus;
  purpose: string;
  createdAt: Date;
  expiresAt?: Date;
  rotatedAt?: Date;
  version: number;
  metadata: Record<string, unknown>;
}

// ============================================================================
// Access Control
// ============================================================================

export enum Permission {
  // Resource Permissions
  CREATE = 'CREATE',
  READ = 'READ',
  UPDATE = 'UPDATE',
  DELETE = 'DELETE',

  // Admin Permissions
  MANAGE_USERS = 'MANAGE_USERS',
  MANAGE_ROLES = 'MANAGE_ROLES',
  MANAGE_POLICIES = 'MANAGE_POLICIES',

  // Security Permissions
  VIEW_AUDIT_LOGS = 'VIEW_AUDIT_LOGS',
  MANAGE_SECURITY = 'MANAGE_SECURITY',
  MANAGE_COMPLIANCE = 'MANAGE_COMPLIANCE',

  // Data Permissions
  EXPORT_DATA = 'EXPORT_DATA',
  IMPORT_DATA = 'IMPORT_DATA',
  PURGE_DATA = 'PURGE_DATA',
}

export interface Role {
  id: string;
  name: string;
  description: string;
  permissions: Permission[];
  inherits?: string[]; // Role inheritance
  metadata: Record<string, unknown>;
  createdAt: Date;
  updatedAt: Date;
}

export interface AccessControl {
  userId: string;
  roles: string[];
  permissions: Permission[];
  attributes: Record<string, unknown>;
  restrictions: AccessRestriction[];
}

export interface AccessRestriction {
  type: 'IP' | 'TIME' | 'LOCATION' | 'DEVICE';
  condition: string;
  enabled: boolean;
}

// ============================================================================
// Threat & Vulnerability Types
// ============================================================================

export enum ThreatSeverity {
  CRITICAL = 'CRITICAL',
  HIGH = 'HIGH',
  MEDIUM = 'MEDIUM',
  LOW = 'LOW',
  INFO = 'INFO',
}

export enum ThreatType {
  MALWARE = 'MALWARE',
  PHISHING = 'PHISHING',
  BRUTE_FORCE = 'BRUTE_FORCE',
  SQL_INJECTION = 'SQL_INJECTION',
  XSS = 'XSS',
  CSRF = 'CSRF',
  DDoS = 'DDoS',
  DATA_EXFILTRATION = 'DATA_EXFILTRATION',
  PRIVILEGE_ESCALATION = 'PRIVILEGE_ESCALATION',
  ANOMALOUS_BEHAVIOR = 'ANOMALOUS_BEHAVIOR',
}

export interface Threat {
  id: string;
  type: ThreatType;
  severity: ThreatSeverity;
  detectedAt: Date;
  source: string;
  target: string;
  description: string;
  indicators: ThreatIndicator[];
  mitigated: boolean;
  mitigationAction?: string;
  mitigatedAt?: Date;
  metadata: Record<string, unknown>;
}

export interface ThreatIndicator {
  type: string;
  value: string;
  confidence: number; // 0-100
  firstSeen: Date;
  lastSeen: Date;
}

export enum VulnerabilitySeverity {
  CRITICAL = 'CRITICAL',
  HIGH = 'HIGH',
  MEDIUM = 'MEDIUM',
  LOW = 'LOW',
  INFO = 'INFO',
}

export interface Vulnerability {
  id: string;
  cveId?: string;
  title: string;
  description: string;
  severity: VulnerabilitySeverity;
  cvssScore?: number;
  affectedComponent: string;
  affectedVersions: string[];
  discoveredAt: Date;
  patchAvailable: boolean;
  patchVersion?: string;
  remediation: string;
  status: 'OPEN' | 'IN_PROGRESS' | 'RESOLVED' | 'ACCEPTED' | 'MITIGATED';
  metadata: Record<string, unknown>;
}

// ============================================================================
// Risk Assessment
// ============================================================================

export enum RiskLevel {
  CRITICAL = 'CRITICAL',
  HIGH = 'HIGH',
  MEDIUM = 'MEDIUM',
  LOW = 'LOW',
  NEGLIGIBLE = 'NEGLIGIBLE',
}

export interface RiskAssessment {
  id: string;
  asset: string;
  riskLevel: RiskLevel;
  likelihood: number; // 1-5
  impact: number; // 1-5
  inherentRisk: number;
  residualRisk: number;
  controls: string[];
  threats: string[];
  vulnerabilities: string[];
  assessmentDate: Date;
  assessor: string;
  nextReviewDate: Date;
  mitigationPlan?: MitigationPlan;
}

export interface MitigationPlan {
  id: string;
  controls: string[];
  actions: MitigationAction[];
  budget?: number;
  timeline: string;
  owner: string;
  status: 'PLANNED' | 'IN_PROGRESS' | 'COMPLETED' | 'ON_HOLD';
}

export interface MitigationAction {
  id: string;
  description: string;
  priority: number;
  dueDate: Date;
  assignedTo: string;
  status: 'PENDING' | 'IN_PROGRESS' | 'COMPLETED';
  completedAt?: Date;
}

// ============================================================================
// Incident Management
// ============================================================================

export enum IncidentSeverity {
  CRITICAL = 'CRITICAL',
  HIGH = 'HIGH',
  MEDIUM = 'MEDIUM',
  LOW = 'LOW',
}

export enum IncidentStatus {
  NEW = 'NEW',
  INVESTIGATING = 'INVESTIGATING',
  CONTAINED = 'CONTAINED',
  ERADICATED = 'ERADICATED',
  RECOVERED = 'RECOVERED',
  CLOSED = 'CLOSED',
}

export interface Incident {
  id: string;
  title: string;
  description: string;
  severity: IncidentSeverity;
  status: IncidentStatus;
  category: string;
  detectedAt: Date;
  reportedBy: string;
  assignedTo?: string;
  affectedSystems: string[];
  affectedUsers: string[];
  rootCause?: string;
  resolution?: string;
  timeline: IncidentTimelineEntry[];
  metadata: Record<string, unknown>;
}

export interface IncidentTimelineEntry {
  id: string;
  timestamp: Date;
  event: string;
  details: string;
  actor: string;
  automated: boolean;
}

// ============================================================================
// Zod Schemas for Validation
// ============================================================================

export const SecurityPolicySchema = z.object({
  id: z.string(),
  name: z.string().min(1),
  type: z.nativeEnum(SecurityPolicyType),
  enforcement: z.nativeEnum(PolicyEnforcement),
  rules: z.array(z.any()),
  metadata: z.record(z.unknown()),
  createdAt: z.date(),
  updatedAt: z.date(),
  createdBy: z.string(),
  version: z.number().int().positive(),
});

export const AuditLogSchema = z.object({
  id: z.string(),
  eventType: z.nativeEnum(AuditEventType),
  severity: z.nativeEnum(AuditSeverity),
  timestamp: z.date(),
  userId: z.string().optional(),
  username: z.string().optional(),
  ipAddress: z.string().optional(),
  resource: z.string(),
  action: z.string(),
  result: z.enum(['SUCCESS', 'FAILURE', 'PARTIAL']),
  details: z.record(z.unknown()),
  metadata: z.any(),
  hash: z.string(),
});

export const ThreatSchema = z.object({
  id: z.string(),
  type: z.nativeEnum(ThreatType),
  severity: z.nativeEnum(ThreatSeverity),
  detectedAt: z.date(),
  source: z.string(),
  target: z.string(),
  description: z.string(),
  indicators: z.array(z.any()),
  mitigated: z.boolean(),
  metadata: z.record(z.unknown()),
});

export const VulnerabilitySchema = z.object({
  id: z.string(),
  title: z.string().min(1),
  description: z.string(),
  severity: z.nativeEnum(VulnerabilitySeverity),
  affectedComponent: z.string(),
  discoveredAt: z.date(),
  patchAvailable: z.boolean(),
  status: z.enum(['OPEN', 'IN_PROGRESS', 'RESOLVED', 'ACCEPTED', 'MITIGATED']),
});

export const IncidentSchema = z.object({
  id: z.string(),
  title: z.string().min(1),
  description: z.string(),
  severity: z.nativeEnum(IncidentSeverity),
  status: z.nativeEnum(IncidentStatus),
  category: z.string(),
  detectedAt: z.date(),
  reportedBy: z.string(),
  affectedSystems: z.array(z.string()),
  affectedUsers: z.array(z.string()),
  timeline: z.array(z.any()),
  metadata: z.record(z.unknown()),
});
