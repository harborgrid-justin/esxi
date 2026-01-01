/**
 * @harborgrid/enterprise-security
 * Enterprise Security & Compliance System
 *
 * Complete security and compliance solution featuring:
 * - Authentication: SSO (SAML/OIDC), MFA, Sessions, JWT Tokens, Biometrics
 * - Authorization: RBAC, ABAC, Policy Engine, Permission Resolution, Scopes
 * - Encryption: Key Management, AES-256-GCM, Argon2id Hashing, Secure Storage
 * - Compliance: SOC2, HIPAA, GDPR, PCI DSS, Immutable Audit Trail
 * - Security Services: Threat Detection, Vulnerability Scanning, Incident Response
 * - React Components: Dashboards, Audit Logs, Policy Management, Role Management
 *
 * @version 0.4.0
 * @license MIT
 */

// ============================================================================
// Core Types
// ============================================================================
export * from './types';

// ============================================================================
// Authentication Module
// ============================================================================
export {
  // SSO Provider
  SSOProvider,
  ssoProvider,
  SSOProtocol,
  SSOBinding,
  type SSOConfig,
  type SSORequest,
  type SSOResponse,
  type SAMLAssertion,
  type OIDCTokens,

  // MFA Manager
  MFAManager,
  mfaManager,
  MFAMethod,
  MFAStatus,
  type MFAConfig,
  type MFAChallenge,
  type TOTPConfig,

  // Session Manager
  SessionManager,
  sessionManager,
  type Session,
  type SessionMetadata,
  type SessionConfig,
  type DeviceInfo,

  // Token Service
  TokenService,
  tokenService,
  TokenType,
  type TokenPayload,
  type TokenPair,
  type TokenConfig,
  type TokenValidationResult,

  // Biometric Auth
  BiometricAuth,
  biometricAuth,
  BiometricType,
  AuthenticatorType,
  type BiometricCredential,
  type RegistrationOptions,
  type AuthenticationOptions,
  type RegistrationResponse,
  type AuthenticationResponse,
} from './auth';

// ============================================================================
// Authorization Module
// ============================================================================
export {
  // RBAC Engine
  RBACEngine,
  rbacEngine,
  type RoleAssignment,
  type AssignmentCondition,
  type RoleHierarchy,

  // ABAC Engine
  ABACEngine,
  abacEngine,
  type AttributePolicy,
  type AttributeRule,
  type AttributeCondition,
  type AttributeContext,
  type AccessDecision,
  ComparisonOperator,

  // Policy Engine
  PolicyEngine,
  policyEngine,
  PolicyMode,
  type PolicyEvaluationContext,
  type PolicyEvaluationResult,
  type PolicyEngineConfig,

  // Permission Resolver
  PermissionResolver,
  permissionResolver,
  type PermissionDefinition,
  type ResolvedPermissions,
  type PermissionCheck,
  type PermissionCheckResult,

  // Scope Manager
  ScopeManager,
  scopeManager,
  ScopeType,
  type Scope,
  type ScopePermission,
  type ScopeCondition,
  type ResourceAccess,
} from './authz';

// ============================================================================
// Encryption Module
// ============================================================================
export {
  // Key Management
  KeyManagement,
  keyManagement,
  type KeyRotationPolicy,
  type KeyMetrics,

  // Data Encryption
  DataEncryption,
  dataEncryption,

  // Hashing Service
  HashingService,
  hashingService,
  type HashOptions,

  // Secure Storage
  SecureStorage,
  secureStorage,

  // Certificate Manager
  CertificateManager,
  certificateManager,
  type Certificate,
} from './encryption';

// ============================================================================
// Compliance Module
// ============================================================================
export {
  // SOC2 Compliance
  SOC2Compliance,
  soc2Compliance,
  type SOC2Control,

  // HIPAA Compliance
  HIPAACompliance,
  hipaaCompliance,

  // GDPR Compliance
  GDPRCompliance,
  gdprCompliance,
  type DataSubjectRequest,

  // PCI DSS Compliance
  PCIDSSCompliance,
  pciDSSCompliance,

  // Audit Trail
  AuditTrail,
  auditTrail,
} from './compliance';

// ============================================================================
// Security Services
// ============================================================================
export {
  // Threat Detection
  ThreatDetection,
  threatDetection,
  type DetectionRule,

  // Vulnerability Scanner
  VulnerabilityScanner,
  vulnerabilityScanner,
  type ScanResult,

  // Incident Response
  IncidentResponse,
  incidentResponse,

  // Data Retention
  DataRetention,
  dataRetention,
  type RetentionPolicy,
  type DataRecord,
} from './services';

// ============================================================================
// React Components
// ============================================================================
export {
  SecurityDashboard,
  type SecurityDashboardProps,
  AuditLogViewer,
  type AuditLogViewerProps,
  PolicyEditor,
  type PolicyEditorProps,
  RoleManager,
  IncidentReport,
  ComplianceReport,
} from './components';

// ============================================================================
// Convenience Exports
// ============================================================================

/**
 * Initialize security system with default configuration
 */
export function initializeSecurity(config?: {
  tokenSecret?: string;
  sessionDuration?: number;
  enableMFA?: boolean;
  complianceFrameworks?: string[];
}) {
  // Configure token service
  if (config?.tokenSecret) {
    const ts = new TokenService({ secret: config.tokenSecret });
  }

  // Configure session manager
  if (config?.sessionDuration) {
    sessionManager.configure({
      sessionDuration: config.sessionDuration,
      refreshTokenDuration: config.sessionDuration * 7,
      maxSessionsPerUser: 5,
      requireMFAForSensitive: config.enableMFA ?? true,
      idleTimeout: 30 * 60 * 1000,
    });
  }

  return {
    auth: {
      sso: ssoProvider,
      mfa: mfaManager,
      sessions: sessionManager,
      tokens: tokenService,
      biometric: biometricAuth,
    },
    authz: {
      rbac: rbacEngine,
      abac: abacEngine,
      policy: policyEngine,
      permissions: permissionResolver,
      scopes: scopeManager,
    },
    encryption: {
      keys: keyManagement,
      data: dataEncryption,
      hashing: hashingService,
      storage: secureStorage,
      certificates: certificateManager,
    },
    compliance: {
      soc2: soc2Compliance,
      hipaa: hipaaCompliance,
      gdpr: gdprCompliance,
      pciDSS: pciDSSCompliance,
      audit: auditTrail,
    },
    services: {
      threats: threatDetection,
      vulnerabilities: vulnerabilityScanner,
      incidents: incidentResponse,
      retention: dataRetention,
    },
  };
}

/**
 * Security system version
 */
export const VERSION = '0.4.0';

/**
 * Package metadata
 */
export const PACKAGE_INFO = {
  name: '@harborgrid/enterprise-security',
  version: VERSION,
  description: 'Enterprise Security & Compliance System',
  author: 'HarborGrid',
  license: 'MIT',
} as const;
