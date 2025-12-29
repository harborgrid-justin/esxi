/**
 * SSO Configuration Component
 * Configure Single Sign-On settings
 */

import React, { useState } from 'react';
import { useForm } from 'react-hook-form';
import { SSOConfiguration, SSOProvider, AttributeMapping } from '../../types';
import { usePermissions } from '../../hooks/usePermissions';

interface SSOConfigProps {
  currentConfig?: SSOConfiguration;
  onSave: (config: SSOConfiguration) => Promise<void>;
  onTest?: (config: SSOConfiguration) => Promise<boolean>;
  className?: string;
}

interface SSOFormData {
  provider: SSOProvider;
  entityId: string;
  ssoUrl: string;
  certificate: string;
  clientId: string;
  clientSecret: string;
  issuer: string;
  authorizationUrl: string;
  tokenUrl: string;
  userInfoUrl: string;
  scopes: string;
  emailMapping: string;
  firstNameMapping: string;
  lastNameMapping: string;
  displayNameMapping: string;
  groupsMapping: string;
  rolesMapping: string;
  autoProvisionUsers: boolean;
  defaultRole: string;
}

export const SSOConfig: React.FC<SSOConfigProps> = ({
  currentConfig,
  onSave,
  onTest,
  className,
}) => {
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isTesting, setIsTesting] = useState(false);
  const [error, setError] = useState<string>('');
  const [success, setSuccess] = useState(false);
  const [testResult, setTestResult] = useState<boolean | null>(null);
  const { can } = usePermissions();

  const canConfigureSSO = can.configureSSO();

  const {
    register,
    handleSubmit,
    watch,
    formState: { errors },
  } = useForm<SSOFormData>({
    defaultValues: {
      provider: currentConfig?.provider || SSOProvider.SAML,
      entityId: currentConfig?.entityId || '',
      ssoUrl: currentConfig?.ssoUrl || '',
      certificate: currentConfig?.certificate || '',
      clientId: currentConfig?.clientId || '',
      clientSecret: currentConfig?.clientSecret || '',
      issuer: currentConfig?.issuer || '',
      authorizationUrl: currentConfig?.authorizationUrl || '',
      tokenUrl: currentConfig?.tokenUrl || '',
      userInfoUrl: currentConfig?.userInfoUrl || '',
      scopes: currentConfig?.scopes?.join(', ') || '',
      emailMapping: currentConfig?.attributeMapping.email || 'email',
      firstNameMapping: currentConfig?.attributeMapping.firstName || 'firstName',
      lastNameMapping: currentConfig?.attributeMapping.lastName || 'lastName',
      displayNameMapping: currentConfig?.attributeMapping.displayName || 'displayName',
      groupsMapping: currentConfig?.attributeMapping.groups || '',
      rolesMapping: currentConfig?.attributeMapping.roles || '',
      autoProvisionUsers: currentConfig?.autoProvisionUsers ?? true,
      defaultRole: currentConfig?.defaultRole || '',
    },
  });

  const provider = watch('provider');

  const onSubmit = async (data: SSOFormData) => {
    if (!canConfigureSSO) {
      setError('You do not have permission to configure SSO');
      return;
    }

    setIsSubmitting(true);
    setError('');
    setSuccess(false);

    try {
      const attributeMapping: AttributeMapping = {
        email: data.emailMapping,
        firstName: data.firstNameMapping || undefined,
        lastName: data.lastNameMapping || undefined,
        displayName: data.displayNameMapping || undefined,
        groups: data.groupsMapping || undefined,
        roles: data.rolesMapping || undefined,
      };

      const config: SSOConfiguration = {
        provider: data.provider,
        attributeMapping,
        autoProvisionUsers: data.autoProvisionUsers,
        defaultRole: data.defaultRole || undefined,
      };

      // Add provider-specific fields
      if (provider === SSOProvider.SAML) {
        config.entityId = data.entityId;
        config.ssoUrl = data.ssoUrl;
        config.certificate = data.certificate;
      } else if (provider === SSOProvider.OIDC || provider === SSOProvider.OAUTH2) {
        config.clientId = data.clientId;
        config.clientSecret = data.clientSecret;
        config.issuer = data.issuer;
        config.authorizationUrl = data.authorizationUrl;
        config.tokenUrl = data.tokenUrl;
        config.userInfoUrl = data.userInfoUrl;
        config.scopes = data.scopes.split(',').map((s) => s.trim());
      }

      await onSave(config);
      setSuccess(true);
      setTimeout(() => setSuccess(false), 3000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save SSO configuration');
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleTest = async () => {
    if (!onTest) return;

    setIsTesting(true);
    setTestResult(null);

    try {
      const data = watch();
      const config: SSOConfiguration = {
        provider: data.provider,
        attributeMapping: {
          email: data.emailMapping,
          firstName: data.firstNameMapping,
          lastName: data.lastNameMapping,
        },
        autoProvisionUsers: data.autoProvisionUsers,
      };

      const result = await onTest(config);
      setTestResult(result);
    } catch (err) {
      setTestResult(false);
      setError('SSO test failed');
    } finally {
      setIsTesting(false);
    }
  };

  if (!canConfigureSSO) {
    return (
      <div className={className} role="alert">
        <p>You do not have permission to configure SSO</p>
      </div>
    );
  }

  return (
    <div className={className}>
      <header>
        <h2>SSO Configuration</h2>
        <p className="subtitle">Configure Single Sign-On for your organization</p>
      </header>

      <form onSubmit={handleSubmit(onSubmit)} className="sso-config-form">
        <section className="form-section">
          <h3>Provider Settings</h3>

          <div className="form-group">
            <label htmlFor="provider">SSO Provider</label>
            <select
              id="provider"
              {...register('provider', { required: 'Provider is required' })}
              className="form-select"
            >
              <option value={SSOProvider.SAML}>SAML 2.0</option>
              <option value={SSOProvider.OIDC}>OpenID Connect</option>
              <option value={SSOProvider.OAUTH2}>OAuth 2.0</option>
              <option value={SSOProvider.LDAP}>LDAP</option>
              <option value={SSOProvider.ACTIVE_DIRECTORY}>Active Directory</option>
            </select>
          </div>

          {provider === SSOProvider.SAML && (
            <>
              <div className="form-group">
                <label htmlFor="entityId">Entity ID</label>
                <input
                  id="entityId"
                  type="text"
                  {...register('entityId')}
                  className="form-input"
                />
              </div>

              <div className="form-group">
                <label htmlFor="ssoUrl">SSO URL</label>
                <input
                  id="ssoUrl"
                  type="url"
                  {...register('ssoUrl')}
                  className="form-input"
                />
              </div>

              <div className="form-group">
                <label htmlFor="certificate">X.509 Certificate</label>
                <textarea
                  id="certificate"
                  {...register('certificate')}
                  className="form-textarea code-editor"
                  rows={6}
                  placeholder="-----BEGIN CERTIFICATE-----&#10;...&#10;-----END CERTIFICATE-----"
                />
              </div>
            </>
          )}

          {(provider === SSOProvider.OIDC || provider === SSOProvider.OAUTH2) && (
            <>
              <div className="form-group">
                <label htmlFor="clientId">Client ID</label>
                <input
                  id="clientId"
                  type="text"
                  {...register('clientId')}
                  className="form-input"
                />
              </div>

              <div className="form-group">
                <label htmlFor="clientSecret">Client Secret</label>
                <input
                  id="clientSecret"
                  type="password"
                  {...register('clientSecret')}
                  className="form-input"
                />
              </div>

              <div className="form-group">
                <label htmlFor="issuer">Issuer</label>
                <input
                  id="issuer"
                  type="text"
                  {...register('issuer')}
                  className="form-input"
                />
              </div>

              <div className="form-group">
                <label htmlFor="authorizationUrl">Authorization URL</label>
                <input
                  id="authorizationUrl"
                  type="url"
                  {...register('authorizationUrl')}
                  className="form-input"
                />
              </div>

              <div className="form-group">
                <label htmlFor="tokenUrl">Token URL</label>
                <input
                  id="tokenUrl"
                  type="url"
                  {...register('tokenUrl')}
                  className="form-input"
                />
              </div>

              <div className="form-group">
                <label htmlFor="userInfoUrl">User Info URL</label>
                <input
                  id="userInfoUrl"
                  type="url"
                  {...register('userInfoUrl')}
                  className="form-input"
                />
              </div>

              <div className="form-group">
                <label htmlFor="scopes">Scopes (comma-separated)</label>
                <input
                  id="scopes"
                  type="text"
                  {...register('scopes')}
                  className="form-input"
                  placeholder="openid, profile, email"
                />
              </div>
            </>
          )}
        </section>

        <section className="form-section">
          <h3>Attribute Mapping</h3>

          <div className="form-group">
            <label htmlFor="emailMapping">Email Attribute</label>
            <input
              id="emailMapping"
              type="text"
              {...register('emailMapping', { required: 'Email mapping is required' })}
              className="form-input"
            />
          </div>

          <div className="form-group">
            <label htmlFor="firstNameMapping">First Name Attribute</label>
            <input
              id="firstNameMapping"
              type="text"
              {...register('firstNameMapping')}
              className="form-input"
            />
          </div>

          <div className="form-group">
            <label htmlFor="lastNameMapping">Last Name Attribute</label>
            <input
              id="lastNameMapping"
              type="text"
              {...register('lastNameMapping')}
              className="form-input"
            />
          </div>

          <div className="form-group">
            <label htmlFor="displayNameMapping">Display Name Attribute</label>
            <input
              id="displayNameMapping"
              type="text"
              {...register('displayNameMapping')}
              className="form-input"
            />
          </div>
        </section>

        <section className="form-section">
          <h3>User Provisioning</h3>

          <div className="form-group checkbox-group">
            <label>
              <input type="checkbox" {...register('autoProvisionUsers')} />
              <span>Auto-provision Users</span>
            </label>
            <p className="help-text">
              Automatically create user accounts on first SSO login
            </p>
          </div>

          <div className="form-group">
            <label htmlFor="defaultRole">Default Role</label>
            <input
              id="defaultRole"
              type="text"
              {...register('defaultRole')}
              className="form-input"
              placeholder="ORG_USER"
            />
          </div>
        </section>

        <div className="form-actions">
          <button
            type="submit"
            disabled={isSubmitting}
            className="btn btn-primary"
          >
            {isSubmitting ? 'Saving...' : 'Save Configuration'}
          </button>
          {onTest && (
            <button
              type="button"
              onClick={handleTest}
              disabled={isTesting}
              className="btn btn-secondary"
            >
              {isTesting ? 'Testing...' : 'Test Configuration'}
            </button>
          )}
        </div>

        {success && (
          <div className="success-message" role="status">
            SSO configuration saved successfully!
          </div>
        )}
        {error && (
          <div className="error-message" role="alert">
            {error}
          </div>
        )}
        {testResult !== null && (
          <div className={testResult ? 'success-message' : 'error-message'} role="status">
            {testResult ? 'SSO test successful!' : 'SSO test failed'}
          </div>
        )}
      </form>
    </div>
  );
};

export default SSOConfig;
