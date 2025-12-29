/**
 * Organization Settings Component
 * Configure organization-specific settings
 */

import React, { useState, useEffect } from 'react';
import { useForm } from 'react-hook-form';
import { useOrganization } from '../../hooks/useOrganization';
import { usePermissions } from '../../hooks/usePermissions';
import { OrganizationSettings as OrgSettingsType, SSOProvider } from '../../types';
import OrganizationService from '../../services/OrganizationService';

interface OrganizationSettingsProps {
  organizationService: OrganizationService;
  organizationId: string;
  className?: string;
}

interface SettingsFormData {
  name: string;
  slug: string;
  description: string;
  allowSubOrganizations: boolean;
  maxUsers: number;
  ssoEnabled: boolean;
  ssoProvider: SSOProvider | '';
  customBrandingEnabled: boolean;
}

export const OrganizationSettings: React.FC<OrganizationSettingsProps> = ({
  organizationService,
  organizationId,
  className,
}) => {
  const { organization, updateOrganization, updateSettings, isLoading } = useOrganization(
    organizationService,
    organizationId
  );
  const { can } = usePermissions();
  const [saveStatus, setSaveStatus] = useState<'idle' | 'saving' | 'success' | 'error'>('idle');
  const [errorMessage, setErrorMessage] = useState<string>('');

  const canManage = can.manageOrganization(organizationId);

  const {
    register,
    handleSubmit,
    reset,
    watch,
    formState: { errors, isDirty },
  } = useForm<SettingsFormData>();

  const ssoEnabled = watch('ssoEnabled');

  useEffect(() => {
    if (organization) {
      reset({
        name: organization.name,
        slug: organization.slug,
        description: organization.description || '',
        allowSubOrganizations: organization.settings.allowSubOrganizations,
        maxUsers: organization.settings.maxUsers,
        ssoEnabled: organization.settings.ssoEnabled,
        ssoProvider: organization.settings.ssoProvider || '',
        customBrandingEnabled: organization.settings.customBrandingEnabled,
      });
    }
  }, [organization, reset]);

  const onSubmit = async (data: SettingsFormData) => {
    if (!canManage) {
      setErrorMessage('You do not have permission to update settings');
      setSaveStatus('error');
      return;
    }

    setSaveStatus('saving');
    setErrorMessage('');

    try {
      // Update basic organization info
      await updateOrganization.mutateAsync({
        name: data.name,
        slug: data.slug,
        description: data.description || undefined,
      });

      // Update settings
      const settings: Partial<OrgSettingsType> = {
        allowSubOrganizations: data.allowSubOrganizations,
        maxUsers: data.maxUsers,
        ssoEnabled: data.ssoEnabled,
        ssoProvider: data.ssoProvider || undefined,
        customBrandingEnabled: data.customBrandingEnabled,
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
      <div className={className} role="status" aria-label="Loading organization settings">
        <div className="loading-spinner">Loading...</div>
      </div>
    );
  }

  if (!organization) {
    return (
      <div className={className} role="alert">
        <p>Unable to load organization settings</p>
      </div>
    );
  }

  return (
    <div className={className}>
      <header className="settings-header">
        <h1>Organization Settings</h1>
        <p className="subtitle">Configure settings for {organization.name}</p>
      </header>

      <form onSubmit={handleSubmit(onSubmit)} className="settings-form">
        {/* Basic Information */}
        <section className="settings-section" aria-labelledby="basic-info-heading">
          <h2 id="basic-info-heading">Basic Information</h2>

          <div className="form-group">
            <label htmlFor="org-name">
              Organization Name
              <span className="required" aria-label="required">*</span>
            </label>
            <input
              id="org-name"
              type="text"
              {...register('name', {
                required: 'Organization name is required',
                minLength: { value: 2, message: 'Name must be at least 2 characters' },
                maxLength: { value: 100, message: 'Name must be less than 100 characters' },
              })}
              disabled={!canManage}
              className="form-input"
              aria-invalid={errors.name ? 'true' : 'false'}
            />
            {errors.name && (
              <p className="error-message" role="alert">
                {errors.name.message}
              </p>
            )}
          </div>

          <div className="form-group">
            <label htmlFor="org-slug">
              Slug
              <span className="required" aria-label="required">*</span>
            </label>
            <input
              id="org-slug"
              type="text"
              {...register('slug', {
                required: 'Slug is required',
                pattern: {
                  value: /^[a-z0-9-]+$/,
                  message: 'Slug can only contain lowercase letters, numbers, and hyphens',
                },
                minLength: { value: 2, message: 'Slug must be at least 2 characters' },
                maxLength: { value: 50, message: 'Slug must be less than 50 characters' },
              })}
              disabled={!canManage}
              className="form-input"
              aria-invalid={errors.slug ? 'true' : 'false'}
            />
            {errors.slug && (
              <p className="error-message" role="alert">
                {errors.slug.message}
              </p>
            )}
            <p className="help-text">Used in URLs and must be unique</p>
          </div>

          <div className="form-group">
            <label htmlFor="org-description">Description</label>
            <textarea
              id="org-description"
              {...register('description', {
                maxLength: {
                  value: 500,
                  message: 'Description must be less than 500 characters',
                },
              })}
              disabled={!canManage}
              className="form-textarea"
              rows={4}
              aria-invalid={errors.description ? 'true' : 'false'}
            />
            {errors.description && (
              <p className="error-message" role="alert">
                {errors.description.message}
              </p>
            )}
          </div>
        </section>

        {/* Organization Features */}
        <section className="settings-section" aria-labelledby="features-heading">
          <h2 id="features-heading">Organization Features</h2>

          <div className="form-group checkbox-group">
            <label>
              <input
                type="checkbox"
                {...register('allowSubOrganizations')}
                disabled={!canManage}
              />
              <span>Allow Sub-Organizations</span>
            </label>
            <p className="help-text">
              Enable creation of nested organizations
            </p>
          </div>

          <div className="form-group checkbox-group">
            <label>
              <input
                type="checkbox"
                {...register('customBrandingEnabled')}
                disabled={!canManage}
              />
              <span>Enable Custom Branding</span>
            </label>
            <p className="help-text">
              Allow custom logos, colors, and styling
            </p>
          </div>

          <div className="form-group">
            <label htmlFor="max-users">
              Maximum Users
              <span className="required" aria-label="required">*</span>
            </label>
            <input
              id="max-users"
              type="number"
              {...register('maxUsers', {
                required: 'Maximum users is required',
                min: { value: 1, message: 'Must allow at least 1 user' },
                max: { value: 100000, message: 'Maximum 100,000 users' },
              })}
              disabled={!canManage}
              className="form-input"
              aria-invalid={errors.maxUsers ? 'true' : 'false'}
            />
            {errors.maxUsers && (
              <p className="error-message" role="alert">
                {errors.maxUsers.message}
              </p>
            )}
          </div>
        </section>

        {/* SSO Settings */}
        <section className="settings-section" aria-labelledby="sso-settings-heading">
          <h2 id="sso-settings-heading">SSO Settings</h2>

          <div className="form-group checkbox-group">
            <label>
              <input
                type="checkbox"
                {...register('ssoEnabled')}
                disabled={!canManage}
              />
              <span>Enable SSO</span>
            </label>
            <p className="help-text">
              Enable Single Sign-On for this organization
            </p>
          </div>

          {ssoEnabled && (
            <div className="form-group">
              <label htmlFor="sso-provider">SSO Provider</label>
              <select
                id="sso-provider"
                {...register('ssoProvider')}
                disabled={!canManage}
                className="form-select"
              >
                <option value="">Select Provider</option>
                <option value={SSOProvider.SAML}>SAML</option>
                <option value={SSOProvider.OIDC}>OpenID Connect (OIDC)</option>
                <option value={SSOProvider.OAUTH2}>OAuth 2.0</option>
                <option value={SSOProvider.LDAP}>LDAP</option>
                <option value={SSOProvider.ACTIVE_DIRECTORY}>Active Directory</option>
              </select>
              <p className="help-text">
                Select your identity provider type
              </p>
            </div>
          )}
        </section>

        {/* Status Information */}
        <section className="settings-section" aria-labelledby="status-heading">
          <h2 id="status-heading">Status Information</h2>

          <div className="info-grid">
            <div className="info-item">
              <span className="label">Status:</span>
              <span className={`value status-badge ${organization.isActive ? 'active' : 'inactive'}`}>
                {organization.isActive ? 'Active' : 'Inactive'}
              </span>
            </div>
            <div className="info-item">
              <span className="label">Created:</span>
              <span className="value">
                {new Date(organization.createdAt).toLocaleString()}
              </span>
            </div>
            <div className="info-item">
              <span className="label">Last Updated:</span>
              <span className="value">
                {new Date(organization.updatedAt).toLocaleString()}
              </span>
            </div>
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

export default OrganizationSettings;
