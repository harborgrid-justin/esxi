/**
 * Accessibility Tenant - Enterprise Multi-Tenant Management System
 * Main export file for the tenant management library
 * @packageDocumentation
 */

// Types
export * from './types';

// Services
export { TenantService } from './services/TenantService';
export { OrganizationService } from './services/OrganizationService';
export { UserService } from './services/UserService';
export { BillingService } from './services/BillingService';

// Context
export { TenantProvider, useTenantContext } from './context/TenantContext';

// Hooks
export { useTenant } from './hooks/useTenant';
export { useOrganization, useOrganizationList } from './hooks/useOrganization';
export {
  usePermissions,
  usePermissionFilter,
  usePermissionGate,
  useRoleGate,
} from './hooks/usePermissions';

// Utilities
export * from './utils/permissions';

// Admin Components
export { TenantDashboard } from './components/Admin/TenantDashboard';
export { OrganizationList } from './components/Admin/OrganizationList';
export { TenantSettings } from './components/Admin/TenantSettings';
export { UsageMetrics } from './components/Admin/UsageMetrics';

// Organization Components
export { OrganizationSettings } from './components/Organization/OrganizationSettings';
export { BrandingConfig } from './components/Organization/BrandingConfig';
export { DomainConfig } from './components/Organization/DomainConfig';

// User Components
export { UserManagement } from './components/Users/UserManagement';
export { RoleManagement } from './components/Users/RoleManagement';
export { InviteUser } from './components/Users/InviteUser';
export { UserProfile } from './components/Users/UserProfile';

// Permission Components
export { PermissionMatrix } from './components/Permissions/PermissionMatrix';
export { RoleEditor } from './components/Permissions/RoleEditor';

// Billing Components
export { SubscriptionManager } from './components/Billing/SubscriptionManager';
export { UsageDisplay } from './components/Billing/UsageDisplay';
export { InvoiceHistory } from './components/Billing/InvoiceHistory';

// SSO Components
export { SSOConfig } from './components/SSO/SSOConfig';
export { ProviderSetup } from './components/SSO/ProviderSetup';

// Audit Components
export { AuditLog } from './components/Audit/AuditLog';
export { ActivityFeed } from './components/Audit/ActivityFeed';

// Default exports
export default {
  // Services
  TenantService,
  OrganizationService,
  UserService,
  BillingService,

  // Context
  TenantProvider,
  useTenantContext,

  // Hooks
  useTenant,
  useOrganization,
  useOrganizationList,
  usePermissions,
  usePermissionFilter,
  usePermissionGate,
  useRoleGate,

  // Admin Components
  TenantDashboard,
  OrganizationList,
  TenantSettings,
  UsageMetrics,

  // Organization Components
  OrganizationSettings,
  BrandingConfig,
  DomainConfig,

  // User Components
  UserManagement,
  RoleManagement,
  InviteUser,
  UserProfile,

  // Permission Components
  PermissionMatrix,
  RoleEditor,

  // Billing Components
  SubscriptionManager,
  UsageDisplay,
  InvoiceHistory,

  // SSO Components
  SSOConfig,
  ProviderSetup,

  // Audit Components
  AuditLog,
  ActivityFeed,
};
