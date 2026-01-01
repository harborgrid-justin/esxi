# @harborgrid/enterprise-security

Enterprise-grade Security & Compliance System with comprehensive authentication, authorization, encryption, and compliance frameworks.

## Features

### 🔐 Authentication
- **SSO Provider**: SAML 2.0 and OpenID Connect (OIDC) single sign-on
- **MFA Manager**: Multi-factor authentication (TOTP, SMS, Email, Biometric)
- **Session Manager**: Secure session management with multi-device support
- **Token Service**: JWT and OAuth2 token management
- **Biometric Auth**: WebAuthn/FIDO2 biometric authentication

### 🛡️ Authorization
- **RBAC Engine**: Role-based access control with inheritance
- **ABAC Engine**: Attribute-based access control for fine-grained permissions
- **Policy Engine**: Unified policy evaluation combining RBAC and ABAC
- **Permission Resolver**: Dynamic permission resolution with caching
- **Scope Manager**: Resource-level access control and ownership

### 🔒 Encryption
- **Key Management**: Automated key rotation and lifecycle management
- **Data Encryption**: AES-256-GCM encryption for data at rest and in transit
- **Hashing Service**: Argon2id password hashing (PBKDF2 fallback)
- **Secure Storage**: Transparent encryption/decryption storage layer
- **Certificate Manager**: TLS/SSL certificate management

### 📋 Compliance
- **SOC 2**: Trust Service Criteria (Security, Availability, Processing Integrity, Confidentiality, Privacy)
- **HIPAA**: Administrative, Physical, and Technical Safeguards
- **GDPR**: Data subject rights and privacy controls
- **PCI DSS**: Payment card industry data security standards
- **Audit Trail**: Immutable, cryptographically-verified audit logging

### 🔍 Security Services
- **Threat Detection**: Real-time anomaly detection and threat intelligence
- **Vulnerability Scanner**: Automated security scanning and assessment
- **Incident Response**: Security incident management and tracking
- **Data Retention**: Automated data lifecycle and retention policies

### ⚛️ React Components
- **SecurityDashboard**: Real-time security metrics overview
- **AuditLogViewer**: Searchable audit log interface
- **PolicyEditor**: Security policy management UI
- **RoleManager**: RBAC role and permission management
- **IncidentReport**: Security incident reporting form
- **ComplianceReport**: Compliance framework status dashboard

## Installation

```bash
npm install @harborgrid/enterprise-security
# or
yarn add @harborgrid/enterprise-security
# or
pnpm add @harborgrid/enterprise-security
```

## Quick Start

### Initialize Security System

```typescript
import { initializeSecurity } from '@harborgrid/enterprise-security';

const security = initializeSecurity({
  tokenSecret: process.env.JWT_SECRET,
  sessionDuration: 24 * 60 * 60 * 1000, // 24 hours
  enableMFA: true,
  complianceFrameworks: ['SOC2', 'GDPR'],
});
```

### Authentication Examples

#### SSO Authentication
```typescript
import { ssoProvider, SSOProtocol } from '@harborgrid/enterprise-security';

// Configure SSO provider
ssoProvider.registerProvider({
  protocol: SSOProtocol.SAML2,
  providerId: 'okta',
  providerName: 'Okta SSO',
  issuer: 'https://your-company.okta.com',
  entryPoint: 'https://your-company.okta.com/sso/saml',
  certificate: '-----BEGIN CERTIFICATE-----...',
  callbackUrl: 'https://your-app.com/auth/callback',
});

// Initiate SSO login
const request = await ssoProvider.initiateSSO('okta');
const samlRequest = await ssoProvider.generateSAMLRequest('okta');
```

#### Multi-Factor Authentication
```typescript
import { mfaManager, MFAMethod } from '@harborgrid/enterprise-security';

// Setup TOTP
const totpConfig = await mfaManager.setupTOTP('user-123');
console.log('Scan QR code:', totpConfig.qrCode);
console.log('Backup codes:', totpConfig.backupCodes);

// Verify setup
const verified = await mfaManager.verifyTOTPSetup('user-123', '123456');

// Send MFA challenge
const challengeId = await mfaManager.sendChallenge('user-123', MFAMethod.SMS);
const isValid = await mfaManager.verifyChallenge(challengeId, '123456');
```

### Authorization Examples

#### RBAC
```typescript
import { rbacEngine, Permission } from '@harborgrid/enterprise-security';

// Create role
const adminRole = await rbacEngine.createRole(
  'Admin',
  'Administrator role with full permissions',
  [Permission.CREATE, Permission.READ, Permission.UPDATE, Permission.DELETE, Permission.MANAGE_USERS]
);

// Assign role to user
await rbacEngine.assignRole('user-123', adminRole.id, 'admin-user');

// Check permissions
const hasPermission = rbacEngine.hasPermission('user-123', Permission.DELETE);
const permissions = rbacEngine.getUserPermissions('user-123');
```

#### ABAC
```typescript
import { abacEngine } from '@harborgrid/enterprise-security';

// Create attribute-based policy
const policy = await abacEngine.createPolicy(
  'Document Access Policy',
  'Allow users to access documents in their department',
  'ALLOW',
  [{
    id: 'rule-1',
    subject: [{ attribute: 'department', operator: 'EQUALS', value: 'engineering' }],
    resource: [{ attribute: 'department', operator: 'EQUALS', value: 'engineering' }],
    action: ['read', 'write'],
    operator: 'AND',
  }],
  100
);

// Evaluate access
const decision = await abacEngine.evaluate({
  subject: { userId: 'user-123', department: 'engineering' },
  resource: { type: 'document', department: 'engineering' },
  action: 'read',
  environment: { timestamp: Date.now() },
});

console.log('Access allowed:', decision.allowed);
```

### Encryption Examples

```typescript
import { dataEncryption, keyManagement, hashingService } from '@harborgrid/enterprise-security';

// Generate encryption key
await keyManagement.generateKey('SYMMETRIC', 'AES_256_GCM', 'user-data');

// Encrypt data
const encrypted = await dataEncryption.encrypt('sensitive data', 'user-data');

// Decrypt data
const decrypted = await dataEncryption.decrypt(encrypted);

// Hash password
const hashedPassword = await hashingService.hash('user-password');
const isValid = await hashingService.verify('user-password', hashedPassword);
```

### Compliance Examples

```typescript
import { auditTrail, soc2Compliance } from '@harborgrid/enterprise-security';
import { AuditEventType, AuditSeverity } from '@harborgrid/enterprise-security';

// Log audit event
await auditTrail.log(
  AuditEventType.DATA_ACCESSED,
  AuditSeverity.INFO,
  'user-records',
  'READ',
  'SUCCESS',
  { recordCount: 10 },
  {
    userId: 'user-123',
    username: 'john.doe',
    ipAddress: '192.168.1.1',
  }
);

// Query audit logs
const logs = auditTrail.query({
  userId: 'user-123',
  startDate: new Date('2024-01-01'),
  limit: 100,
});

// Check SOC 2 compliance
const summary = soc2Compliance.getSummary();
console.log('Compliance rate:', summary.complianceRate);
```

### React Component Examples

```tsx
import {
  SecurityDashboard,
  AuditLogViewer,
  RoleManager,
  ComplianceReport
} from '@harborgrid/enterprise-security';

function SecurityApp() {
  return (
    <div>
      <SecurityDashboard refreshInterval={30000} />
      <AuditLogViewer userId="user-123" limit={50} />
      <RoleManager />
      <ComplianceReport />
    </div>
  );
}
```

## API Documentation

### Authentication
- `SSOProvider` - SAML/OIDC SSO integration
- `MFAManager` - Multi-factor authentication
- `SessionManager` - Session lifecycle management
- `TokenService` - JWT token generation and validation
- `BiometricAuth` - WebAuthn biometric authentication

### Authorization
- `RBACEngine` - Role-based access control
- `ABACEngine` - Attribute-based access control
- `PolicyEngine` - Unified policy evaluation
- `PermissionResolver` - Permission resolution with caching
- `ScopeManager` - Resource scope management

### Encryption
- `KeyManagement` - Encryption key lifecycle
- `DataEncryption` - AES-256-GCM encryption
- `HashingService` - Password hashing
- `SecureStorage` - Encrypted storage
- `CertificateManager` - TLS certificate management

### Compliance
- `SOC2Compliance` - SOC 2 controls
- `HIPAACompliance` - HIPAA safeguards
- `GDPRCompliance` - GDPR data rights
- `PCIDSSCompliance` - PCI DSS requirements
- `AuditTrail` - Immutable audit logging

### Services
- `ThreatDetection` - Threat and anomaly detection
- `VulnerabilityScanner` - Security vulnerability scanning
- `IncidentResponse` - Incident management
- `DataRetention` - Data lifecycle policies

## TypeScript Support

Full TypeScript support with comprehensive type definitions:

```typescript
import type {
  SecurityPolicy,
  ComplianceStandard,
  AuditLog,
  Encryption,
  Threat,
  Vulnerability,
  Incident
} from '@harborgrid/enterprise-security';
```

## License

MIT © HarborGrid

## Support

For issues and questions, please file an issue on GitHub.
