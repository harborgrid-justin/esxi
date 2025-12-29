/**
 * Tenant Settings Component
 * Configure tenant-wide settings and policies
 */

import React, { useState, useEffect } from 'react';
import { useForm } from 'react-hook-form';
import { useTenant } from '../../hooks/useTenant';
import { usePermissions } from '../../hooks/usePermissions';
import { TenantSettings as TenantSettingsType, PasswordPolicy } from '../../types';
import TenantService from '../../services/TenantService';

interface TenantSettingsProps {
  tenantService: TenantService;
  tenantId?: string;
  className?: string;
}

interface SettingsFormData {
  allowCustomDomains: boolean;
  allowSSOConfiguration: boolean;
  enforceSSO: boolean;
  allowUserRegistration: boolean;
  requireEmailVerification: boolean;
  sessionTimeout: number;
  maxSessionsPerUser: number;
  auditLogRetention: number;
  dataResidency: string;
  passwordMinLength: number;
  passwordRequireUppercase: boolean;
  passwordRequireLowercase: boolean;
  passwordRequireNumbers: boolean;
  passwordRequireSpecialChars: boolean;
  passwordExpirationDays: number;
  passwordPreventReuse: number;
}

export const TenantSettings: React.FC<TenantSettingsProps> = ({
  tenantService,
  tenantId,
  className,
}) => {
  const { tenant, updateSettings, isLoading } = useTenant(tenantService, tenantId);
  const { can } = usePermissions();
  const [saveStatus, setSaveStatus] = useState<'idle' | 'saving' | 'success' | 'error'>('idle');
  const [errorMessage, setErrorMessage] = useState<string>('');

  const canManage = can.manageTenant();

  const {
    register,
    handleSubmit,
    reset,
    formState: { errors, isDirty },
  } = useForm<SettingsFormData>();

  useEffect(() => {
    if (tenant?.settings) {
      reset({
        allowCustomDomains: tenant.settings.allowCustomDomains,
        allowSSOConfiguration: tenant.settings.allowSSOConfiguration,
        enforceSSO: tenant.settings.enforceSSO,
        allowUserRegistration: tenant.settings.allowUserRegistration,
        requireEmailVerification: tenant.settings.requireEmailVerification,
        sessionTimeout: tenant.settings.sessionTimeout,
        maxSessionsPerUser: tenant.settings.maxSessionsPerUser,
        auditLogRetention: tenant.settings.auditLogRetention,
        dataResidency: tenant.settings.dataResidency || '',
        passwordMinLength: tenant.settings.passwordPolicy.minLength,
        passwordRequireUppercase: tenant.settings.passwordPolicy.requireUppercase,
        passwordRequireLowercase: tenant.settings.passwordPolicy.requireLowercase,
        passwordRequireNumbers: tenant.settings.passwordPolicy.requireNumbers,
        passwordRequireSpecialChars: tenant.settings.passwordPolicy.requireSpecialChars,
        passwordExpirationDays: tenant.settings.passwordPolicy.expirationDays || 0,
        passwordPreventReuse: tenant.settings.passwordPolicy.preventReuse,
      });
    }
  }, [tenant, reset]);

  const onSubmit = async (data: SettingsFormData) => {
    if (!canManage) {
      setErrorMessage('You do not have permission to update settings');
      setSaveStatus('error');
      return;
    }

    setSaveStatus('saving');
    setErrorMessage('');

    try {
      const passwordPolicy: PasswordPolicy = {
        minLength: data.passwordMinLength,
        requireUppercase: data.passwordRequireUppercase,
        requireLowercase: data.passwordRequireLowercase,
        requireNumbers: data.passwordRequireNumbers,
        requireSpecialChars: data.passwordRequireSpecialChars,
        expirationDays: data.passwordExpirationDays || undefined,
        preventReuse: data.passwordPreventReuse,
      };

      const settings: Partial<TenantSettingsType> = {
        allowCustomDomains: data.allowCustomDomains,
        allowSSOConfiguration: data.allowSSOConfiguration,
        enforceSSO: data.enforceSSO,
        allowUserRegistration: data.allowUserRegistration,
        requireEmailVerification: data.requireEmailVerification,
        sessionTimeout: data.sessionTimeout,
        maxSessionsPerUser: data.maxSessionsPerUser,
        auditLogRetention: data.auditLogRetention,
        dataResidency: data.dataResidency || undefined,
        passwordPolicy,
      };

      await updateSettings.mutateAsync(settings);
      setSaveStatus('success');
      setTimeout(() => setSaveStatus('idle'), 3000);
    } catch (error) {
      setErrorMessage(error instanceof Error ? error.message : 'Failed to save settings');
      setSaveStatus('error');
    }
  };

  if (isLoading) {
    return (
      <div className={className} role="status" aria-label="Loading settings">
        <div className="loading-spinner">Loading...</div>
      </div>
    );
  }

  if (!tenant) {
    return (
      <div className={className} role="alert">
        <p>Unable to load tenant settings</p>
      </div>
    );
  }

  return (
    <div className={className}>
      <header className="settings-header">
        <h1>Tenant Settings</h1>
        <p className="subtitle">Configure tenant-wide policies and preferences</p>
      </header>

      <form onSubmit={handleSubmit(onSubmit)} className="settings-form">
        {/* General Settings */}
        <section className="settings-section" aria-labelledby="general-settings-heading">
          <h2 id="general-settings-heading">General Settings</h2>

          <div className="form-group checkbox-group">
            <label>
              <input
                type="checkbox"
                {...register('allowCustomDomains')}
                disabled={!canManage}
              />
              <span>Allow Custom Domains</span>
            </label>
            <p className="help-text">
              Enable organizations to configure custom domains
            </p>
          </div>

          <div className="form-group checkbox-group">
            <label>
              <input
                type="checkbox"
                {...register('allowUserRegistration')}
                disabled={!canManage}
              />
              <span>Allow User Registration</span>
            </label>
            <p className="help-text">
              Allow users to self-register for accounts
            </p>
          </div>

          <div className="form-group checkbox-group">
            <label>
              <input
                type="checkbox"
                {...register('requireEmailVerification')}
                disabled={!canManage}
              />
              <span>Require Email Verification</span>
            </label>
            <p className="help-text">
              Require users to verify their email address
            </p>
          </div>

          <div className="form-group">
            <label htmlFor="data-residency">Data Residency</label>
            <input
              id="data-residency"
              type="text"
              {...register('dataResidency')}
              placeholder="e.g., US, EU, APAC"
              disabled={!canManage}
              className="form-input"
            />
            <p className="help-text">
              Specify data residency region (optional)
            </p>
          </div>
        </section>

        {/* SSO Settings */}
        <section className="settings-section" aria-labelledby="sso-settings-heading">
          <h2 id="sso-settings-heading">SSO Settings</h2>

          <div className="form-group checkbox-group">
            <label>
              <input
                type="checkbox"
                {...register('allowSSOConfiguration')}
                disabled={!canManage}
              />
              <span>Allow SSO Configuration</span>
            </label>
            <p className="help-text">
              Enable SSO configuration for organizations
            </p>
          </div>

          <div className="form-group checkbox-group">
            <label>
              <input
                type="checkbox"
                {...register('enforceSSO')}
                disabled={!canManage}
              />
              <span>Enforce SSO</span>
            </label>
            <p className="help-text">
              Require SSO authentication for all users
            </p>
          </div>
        </section>

        {/* Session Settings */}
        <section className="settings-section" aria-labelledby="session-settings-heading">
          <h2 id="session-settings-heading">Session Settings</h2>

          <div className="form-group">
            <label htmlFor="session-timeout">
              Session Timeout (minutes)
              <span className="required" aria-label="required">*</span>
            </label>
            <input
              id="session-timeout"
              type="number"
              {...register('sessionTimeout', {
                required: 'Session timeout is required',
                min: { value: 5, message: 'Minimum 5 minutes' },
                max: { value: 1440, message: 'Maximum 24 hours (1440 minutes)' },
              })}
              disabled={!canManage}
              className="form-input"
              aria-invalid={errors.sessionTimeout ? 'true' : 'false'}
            />
            {errors.sessionTimeout && (
              <p className="error-message" role="alert">
                {errors.sessionTimeout.message}
              </p>
            )}
          </div>

          <div className="form-group">
            <label htmlFor="max-sessions">
              Max Sessions Per User
              <span className="required" aria-label="required">*</span>
            </label>
            <input
              id="max-sessions"
              type="number"
              {...register('maxSessionsPerUser', {
                required: 'Max sessions is required',
                min: { value: 1, message: 'Minimum 1 session' },
                max: { value: 10, message: 'Maximum 10 sessions' },
              })}
              disabled={!canManage}
              className="form-input"
              aria-invalid={errors.maxSessionsPerUser ? 'true' : 'false'}
            />
            {errors.maxSessionsPerUser && (
              <p className="error-message" role="alert">
                {errors.maxSessionsPerUser.message}
              </p>
            )}
          </div>
        </section>

        {/* Password Policy */}
        <section className="settings-section" aria-labelledby="password-policy-heading">
          <h2 id="password-policy-heading">Password Policy</h2>

          <div className="form-group">
            <label htmlFor="password-min-length">
              Minimum Length
              <span className="required" aria-label="required">*</span>
            </label>
            <input
              id="password-min-length"
              type="number"
              {...register('passwordMinLength', {
                required: 'Minimum length is required',
                min: { value: 8, message: 'Minimum 8 characters' },
                max: { value: 128, message: 'Maximum 128 characters' },
              })}
              disabled={!canManage}
              className="form-input"
              aria-invalid={errors.passwordMinLength ? 'true' : 'false'}
            />
            {errors.passwordMinLength && (
              <p className="error-message" role="alert">
                {errors.passwordMinLength.message}
              </p>
            )}
          </div>

          <div className="form-group checkbox-group">
            <label>
              <input
                type="checkbox"
                {...register('passwordRequireUppercase')}
                disabled={!canManage}
              />
              <span>Require Uppercase Letters</span>
            </label>
          </div>

          <div className="form-group checkbox-group">
            <label>
              <input
                type="checkbox"
                {...register('passwordRequireLowercase')}
                disabled={!canManage}
              />
              <span>Require Lowercase Letters</span>
            </label>
          </div>

          <div className="form-group checkbox-group">
            <label>
              <input
                type="checkbox"
                {...register('passwordRequireNumbers')}
                disabled={!canManage}
              />
              <span>Require Numbers</span>
            </label>
          </div>

          <div className="form-group checkbox-group">
            <label>
              <input
                type="checkbox"
                {...register('passwordRequireSpecialChars')}
                disabled={!canManage}
              />
              <span>Require Special Characters</span>
            </label>
          </div>

          <div className="form-group">
            <label htmlFor="password-expiration">
              Password Expiration (days, 0 for never)
            </label>
            <input
              id="password-expiration"
              type="number"
              {...register('passwordExpirationDays', {
                min: { value: 0, message: 'Cannot be negative' },
                max: { value: 365, message: 'Maximum 365 days' },
              })}
              disabled={!canManage}
              className="form-input"
              aria-invalid={errors.passwordExpirationDays ? 'true' : 'false'}
            />
            {errors.passwordExpirationDays && (
              <p className="error-message" role="alert">
                {errors.passwordExpirationDays.message}
              </p>
            )}
          </div>

          <div className="form-group">
            <label htmlFor="password-prevent-reuse">
              Prevent Password Reuse (last N passwords)
            </label>
            <input
              id="password-prevent-reuse"
              type="number"
              {...register('passwordPreventReuse', {
                min: { value: 0, message: 'Cannot be negative' },
                max: { value: 24, message: 'Maximum 24 passwords' },
              })}
              disabled={!canManage}
              className="form-input"
              aria-invalid={errors.passwordPreventReuse ? 'true' : 'false'}
            />
            {errors.passwordPreventReuse && (
              <p className="error-message" role="alert">
                {errors.passwordPreventReuse.message}
              </p>
            )}
          </div>
        </section>

        {/* Audit Settings */}
        <section className="settings-section" aria-labelledby="audit-settings-heading">
          <h2 id="audit-settings-heading">Audit Settings</h2>

          <div className="form-group">
            <label htmlFor="audit-retention">
              Audit Log Retention (days)
              <span className="required" aria-label="required">*</span>
            </label>
            <input
              id="audit-retention"
              type="number"
              {...register('auditLogRetention', {
                required: 'Retention period is required',
                min: { value: 30, message: 'Minimum 30 days' },
                max: { value: 2555, message: 'Maximum 7 years (2555 days)' },
              })}
              disabled={!canManage}
              className="form-input"
              aria-invalid={errors.auditLogRetention ? 'true' : 'false'}
            />
            {errors.auditLogRetention && (
              <p className="error-message" role="alert">
                {errors.auditLogRetention.message}
              </p>
            )}
          </div>
        </section>

        {/* Form Actions */}
        {canManage && (
          <div className="form-actions">
            <button
              type="submit"
              disabled={!isDirty || saveStatus === 'saving'}
              className="btn btn-primary"
              aria-label="Save settings"
            >
              {saveStatus === 'saving' ? 'Saving...' : 'Save Settings'}
            </button>
            <button
              type="button"
              onClick={() => reset()}
              disabled={!isDirty || saveStatus === 'saving'}
              className="btn btn-secondary"
              aria-label="Reset form"
            >
              Reset
            </button>
          </div>
        )}

        {/* Status Messages */}
        {saveStatus === 'success' && (
          <div className="success-message" role="status" aria-live="polite">
            Settings saved successfully!
          </div>
        )}
        {saveStatus === 'error' && errorMessage && (
          <div className="error-message" role="alert" aria-live="assertive">
            {errorMessage}
          </div>
        )}
      </form>
    </div>
  );
};

export default TenantSettings;
