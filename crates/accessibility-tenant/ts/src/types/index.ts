/**
 * Enterprise Multi-Tenant Type Definitions
 * Comprehensive type system for SaaS multi-tenancy
 */

export enum SubscriptionTier {
  FREE = 'FREE',
  STARTER = 'STARTER',
  PROFESSIONAL = 'PROFESSIONAL',
  ENTERPRISE = 'ENTERPRISE',
  CUSTOM = 'CUSTOM'
}

export enum SubscriptionStatus {
  ACTIVE = 'ACTIVE',
  TRIAL = 'TRIAL',
  SUSPENDED = 'SUSPENDED',
  CANCELED = 'CANCELED',
  EXPIRED = 'EXPIRED'
}

export enum UserRole {
  SUPER_ADMIN = 'SUPER_ADMIN',
  TENANT_ADMIN = 'TENANT_ADMIN',
  ORG_ADMIN = 'ORG_ADMIN',
  ORG_MANAGER = 'ORG_MANAGER',
  ORG_USER = 'ORG_USER',
  VIEWER = 'VIEWER'
}

export enum UserStatus {
  ACTIVE = 'ACTIVE',
  INVITED = 'INVITED',
  SUSPENDED = 'SUSPENDED',
  INACTIVE = 'INACTIVE'
}

export enum PermissionAction {
  CREATE = 'CREATE',
  READ = 'READ',
  UPDATE = 'UPDATE',
  DELETE = 'DELETE',
  EXECUTE = 'EXECUTE',
  MANAGE = 'MANAGE'
}

export enum ResourceType {
  TENANT = 'TENANT',
  ORGANIZATION = 'ORGANIZATION',
  USER = 'USER',
  ROLE = 'ROLE',
  PERMISSION = 'PERMISSION',
  BILLING = 'BILLING',
  SSO = 'SSO',
  AUDIT = 'AUDIT',
  SETTINGS = 'SETTINGS'
}

export enum SSOProvider {
  SAML = 'SAML',
  OIDC = 'OIDC',
  OAUTH2 = 'OAUTH2',
  LDAP = 'LDAP',
  ACTIVE_DIRECTORY = 'ACTIVE_DIRECTORY'
}

export enum AuditEventType {
  USER_LOGIN = 'USER_LOGIN',
  USER_LOGOUT = 'USER_LOGOUT',
  USER_CREATED = 'USER_CREATED',
  USER_UPDATED = 'USER_UPDATED',
  USER_DELETED = 'USER_DELETED',
  ROLE_CREATED = 'ROLE_CREATED',
  ROLE_UPDATED = 'ROLE_UPDATED',
  ROLE_DELETED = 'ROLE_DELETED',
  PERMISSION_GRANTED = 'PERMISSION_GRANTED',
  PERMISSION_REVOKED = 'PERMISSION_REVOKED',
  ORG_CREATED = 'ORG_CREATED',
  ORG_UPDATED = 'ORG_UPDATED',
  ORG_DELETED = 'ORG_DELETED',
  SSO_CONFIGURED = 'SSO_CONFIGURED',
  BILLING_UPDATED = 'BILLING_UPDATED',
  SETTINGS_CHANGED = 'SETTINGS_CHANGED'
}

export interface Tenant {
  id: string;
  name: string;
  slug: string;
  domain?: string;
  customDomain?: string;
  subscriptionTier: SubscriptionTier;
  subscriptionStatus: SubscriptionStatus;
  maxOrganizations: number;
  maxUsers: number;
  features: string[];
  settings: TenantSettings;
  branding: TenantBranding;
  createdAt: Date;
  updatedAt: Date;
  trialEndsAt?: Date;
  subscriptionEndsAt?: Date;
}

export interface TenantSettings {
  allowCustomDomains: boolean;
  allowSSOConfiguration: boolean;
  enforceSSO: boolean;
  allowUserRegistration: boolean;
  requireEmailVerification: boolean;
  sessionTimeout: number;
  maxSessionsPerUser: number;
  passwordPolicy: PasswordPolicy;
  auditLogRetention: number;
  dataResidency?: string;
}

export interface TenantBranding {
  logoUrl?: string;
  faviconUrl?: string;
  primaryColor?: string;
  secondaryColor?: string;
  customCSS?: string;
  emailTemplates?: Record<string, string>;
}

export interface PasswordPolicy {
  minLength: number;
  requireUppercase: boolean;
  requireLowercase: boolean;
  requireNumbers: boolean;
  requireSpecialChars: boolean;
  expirationDays?: number;
  preventReuse: number;
}

export interface Organization {
  id: string;
  tenantId: string;
  name: string;
  slug: string;
  description?: string;
  domain?: string;
  customDomain?: string;
  settings: OrganizationSettings;
  branding: OrganizationBranding;
  metadata: Record<string, any>;
  createdAt: Date;
  updatedAt: Date;
  createdBy: string;
  isActive: boolean;
}

export interface OrganizationSettings {
  allowSubOrganizations: boolean;
  maxUsers: number;
  ssoEnabled: boolean;
  ssoProvider?: SSOProvider;
  ssoConfiguration?: SSOConfiguration;
  customBrandingEnabled: boolean;
  features: string[];
}

export interface OrganizationBranding {
  logoUrl?: string;
  primaryColor?: string;
  secondaryColor?: string;
  customCSS?: string;
}

export interface SSOConfiguration {
  provider: SSOProvider;
  entityId?: string;
  ssoUrl?: string;
  certificate?: string;
  clientId?: string;
  clientSecret?: string;
  issuer?: string;
  authorizationUrl?: string;
  tokenUrl?: string;
  userInfoUrl?: string;
  scopes?: string[];
  attributeMapping: AttributeMapping;
  autoProvisionUsers: boolean;
  defaultRole?: string;
}

export interface AttributeMapping {
  email: string;
  firstName?: string;
  lastName?: string;
  displayName?: string;
  groups?: string;
  roles?: string;
}

export interface User {
  id: string;
  tenantId: string;
  organizationId: string;
  email: string;
  firstName: string;
  lastName: string;
  displayName: string;
  avatarUrl?: string;
  role: UserRole;
  customRoles: string[];
  permissions: Permission[];
  status: UserStatus;
  lastLoginAt?: Date;
  createdAt: Date;
  updatedAt: Date;
  metadata: Record<string, any>;
  ssoEnabled: boolean;
  ssoProvider?: SSOProvider;
  ssoId?: string;
}

export interface Role {
  id: string;
  tenantId: string;
  organizationId?: string;
  name: string;
  description?: string;
  permissions: Permission[];
  isSystemRole: boolean;
  isDefault: boolean;
  createdAt: Date;
  updatedAt: Date;
}

export interface Permission {
  id: string;
  resource: ResourceType;
  action: PermissionAction;
  conditions?: PermissionCondition[];
  scope?: 'GLOBAL' | 'TENANT' | 'ORGANIZATION' | 'USER';
}

export interface PermissionCondition {
  field: string;
  operator: 'equals' | 'not_equals' | 'in' | 'not_in' | 'contains' | 'greater_than' | 'less_than';
  value: any;
}

export interface Subscription {
  id: string;
  tenantId: string;
  tier: SubscriptionTier;
  status: SubscriptionStatus;
  currentPeriodStart: Date;
  currentPeriodEnd: Date;
  trialEndsAt?: Date;
  canceledAt?: Date;
  pricing: SubscriptionPricing;
  usage: UsageMetrics;
  paymentMethod?: PaymentMethod;
}

export interface SubscriptionPricing {
  basePriceMonthly: number;
  basePriceYearly: number;
  pricePerUser?: number;
  pricePerOrganization?: number;
  includedUsers: number;
  includedOrganizations: number;
  currency: string;
}

export interface UsageMetrics {
  currentUsers: number;
  currentOrganizations: number;
  storageUsedGB: number;
  apiCallsThisMonth: number;
  bandwidthUsedGB: number;
  customMetrics?: Record<string, number>;
}

export interface PaymentMethod {
  type: 'CARD' | 'BANK_ACCOUNT' | 'INVOICE';
  last4?: string;
  brand?: string;
  expiryMonth?: number;
  expiryYear?: number;
}

export interface Invoice {
  id: string;
  tenantId: string;
  subscriptionId: string;
  amount: number;
  currency: string;
  status: 'DRAFT' | 'OPEN' | 'PAID' | 'VOID' | 'UNCOLLECTIBLE';
  dueDate: Date;
  paidAt?: Date;
  invoiceUrl?: string;
  lineItems: InvoiceLineItem[];
  createdAt: Date;
}

export interface InvoiceLineItem {
  description: string;
  quantity: number;
  unitPrice: number;
  amount: number;
  metadata?: Record<string, any>;
}

export interface AuditLog {
  id: string;
  tenantId: string;
  organizationId?: string;
  userId: string;
  userEmail: string;
  eventType: AuditEventType;
  resource: ResourceType;
  resourceId?: string;
  action: string;
  changes?: Record<string, any>;
  metadata?: Record<string, any>;
  ipAddress?: string;
  userAgent?: string;
  timestamp: Date;
  success: boolean;
  errorMessage?: string;
}

export interface Activity {
  id: string;
  userId: string;
  userName: string;
  userAvatar?: string;
  action: string;
  resource: string;
  resourceName?: string;
  description: string;
  timestamp: Date;
  metadata?: Record<string, any>;
}

export interface InviteUser {
  email: string;
  role: UserRole;
  organizationId: string;
  customMessage?: string;
  expiresInDays?: number;
}

export interface UserInvitation {
  id: string;
  tenantId: string;
  organizationId: string;
  email: string;
  role: UserRole;
  invitedBy: string;
  token: string;
  expiresAt: Date;
  acceptedAt?: Date;
  status: 'PENDING' | 'ACCEPTED' | 'EXPIRED' | 'REVOKED';
  createdAt: Date;
}

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: ApiError;
  metadata?: {
    page?: number;
    perPage?: number;
    total?: number;
    totalPages?: number;
  };
}

export interface ApiError {
  code: string;
  message: string;
  details?: Record<string, any>;
  validationErrors?: ValidationError[];
}

export interface ValidationError {
  field: string;
  message: string;
  code: string;
}

export interface PaginationParams {
  page?: number;
  perPage?: number;
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
}

export interface FilterParams {
  search?: string;
  status?: string;
  role?: string;
  organizationId?: string;
  dateFrom?: Date;
  dateTo?: Date;
}
