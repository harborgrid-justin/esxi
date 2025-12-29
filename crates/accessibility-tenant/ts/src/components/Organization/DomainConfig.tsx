/**
 * Domain Configuration Component
 * Configure custom domain for organization
 */

import React, { useState } from 'react';
import { useForm } from 'react-hook-form';
import { useOrganization } from '../../hooks/useOrganization';
import { usePermissions } from '../../hooks/usePermissions';
import OrganizationService from '../../services/OrganizationService';

interface DomainConfigProps {
  organizationService: OrganizationService;
  organizationId: string;
  className?: string;
}

interface DomainFormData {
  customDomain: string;
}

export const DomainConfig: React.FC<DomainConfigProps> = ({
  organizationService,
  organizationId,
  className,
}) => {
  const {
    organization,
    setCustomDomain,
    verifyCustomDomain,
    removeCustomDomain,
    isLoading,
  } = useOrganization(organizationService, organizationId);

  const { can } = usePermissions();
  const [actionStatus, setActionStatus] = useState<'idle' | 'processing' | 'success' | 'error'>(
    'idle'
  );
  const [errorMessage, setErrorMessage] = useState<string>('');
  const [verificationStatus, setVerificationStatus] = useState<{
    verified: boolean;
    records?: Record<string, string>;
  } | null>(null);

  const canManage = can.manageOrganization(organizationId);

  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<DomainFormData>();

  const onSubmit = async (data: DomainFormData) => {
    if (!canManage) {
      setErrorMessage('You do not have permission to configure domain');
      setActionStatus('error');
      return;
    }

    setActionStatus('processing');
    setErrorMessage('');

    try {
      await setCustomDomain.mutateAsync(data.customDomain);
      setActionStatus('success');
      setVerificationStatus(null);
      reset();
      setTimeout(() => setActionStatus('idle'), 3000);
    } catch (error) {
      setErrorMessage(error instanceof Error ? error.message : 'Failed to set domain');
      setActionStatus('error');
    }
  };

  const handleVerify = async () => {
    if (!organization?.customDomain) return;

    setActionStatus('processing');
    setErrorMessage('');

    try {
      const result = await verifyCustomDomain.mutateAsync(organization.customDomain);
      if (result.data) {
        setVerificationStatus(result.data);
        if (result.data.verified) {
          setActionStatus('success');
        } else {
          setActionStatus('idle');
        }
      }
    } catch (error) {
      setErrorMessage(error instanceof Error ? error.message : 'Failed to verify domain');
      setActionStatus('error');
    }
  };

  const handleRemove = async () => {
    if (!confirm('Are you sure you want to remove the custom domain?')) {
      return;
    }

    setActionStatus('processing');
    setErrorMessage('');

    try {
      await removeCustomDomain.mutateAsync();
      setActionStatus('success');
      setVerificationStatus(null);
      setTimeout(() => setActionStatus('idle'), 3000);
    } catch (error) {
      setErrorMessage(error instanceof Error ? error.message : 'Failed to remove domain');
      setActionStatus('error');
    }
  };

  if (isLoading) {
    return (
      <div className={className} role="status" aria-label="Loading domain configuration">
        <div className="loading-spinner">Loading...</div>
      </div>
    );
  }

  if (!organization) {
    return (
      <div className={className} role="alert">
        <p>Unable to load domain configuration</p>
      </div>
    );
  }

  return (
    <div className={className}>
      <header className="config-header">
        <h1>Domain Configuration</h1>
        <p className="subtitle">Configure custom domain for your organization</p>
      </header>

      {/* Current Domain */}
      {organization.customDomain && (
        <section className="current-domain" aria-labelledby="current-domain-heading">
          <h2 id="current-domain-heading">Current Custom Domain</h2>
          <div className="domain-info">
            <div className="domain-value">
              <span className="domain-name">{organization.customDomain}</span>
              {verificationStatus?.verified && (
                <span className="verified-badge" aria-label="Verified">
                  ✓ Verified
                </span>
              )}
            </div>
            {canManage && (
              <div className="domain-actions">
                <button
                  type="button"
                  onClick={handleVerify}
                  disabled={actionStatus === 'processing'}
                  className="btn btn-secondary"
                  aria-label="Verify domain"
                >
                  {actionStatus === 'processing' ? 'Verifying...' : 'Verify Domain'}
                </button>
                <button
                  type="button"
                  onClick={handleRemove}
                  disabled={actionStatus === 'processing'}
                  className="btn btn-danger"
                  aria-label="Remove domain"
                >
                  Remove Domain
                </button>
              </div>
            )}
          </div>
        </section>
      )}

      {/* DNS Records */}
      {verificationStatus && !verificationStatus.verified && verificationStatus.records && (
        <section className="dns-records" aria-labelledby="dns-records-heading">
          <h2 id="dns-records-heading">DNS Configuration</h2>
          <p>Add the following DNS records to verify your domain:</p>
          <div className="records-table">
            <table>
              <thead>
                <tr>
                  <th>Type</th>
                  <th>Name</th>
                  <th>Value</th>
                </tr>
              </thead>
              <tbody>
                {Object.entries(verificationStatus.records).map(([key, value]) => (
                  <tr key={key}>
                    <td>TXT</td>
                    <td>{key}</td>
                    <td>
                      <code>{value}</code>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <p className="help-text">
            DNS changes may take up to 48 hours to propagate. Click "Verify Domain" once you've
            added the records.
          </p>
        </section>
      )}

      {/* Add/Update Domain Form */}
      {canManage && (
        <section className="domain-form" aria-labelledby="domain-form-heading">
          <h2 id="domain-form-heading">
            {organization.customDomain ? 'Update Domain' : 'Add Custom Domain'}
          </h2>

          <form onSubmit={handleSubmit(onSubmit)}>
            <div className="form-group">
              <label htmlFor="custom-domain">
                Domain Name
                <span className="required" aria-label="required">*</span>
              </label>
              <input
                id="custom-domain"
                type="text"
                {...register('customDomain', {
                  required: 'Domain is required',
                  pattern: {
                    value: /^[a-zA-Z0-9][a-zA-Z0-9-]{0,61}[a-zA-Z0-9]?\.([a-zA-Z]{2,}\.?)+$/,
                    message: 'Invalid domain format',
                  },
                })}
                placeholder="example.com"
                className="form-input"
                aria-invalid={errors.customDomain ? 'true' : 'false'}
              />
              {errors.customDomain && (
                <p className="error-message" role="alert">
                  {errors.customDomain.message}
                </p>
              )}
              <p className="help-text">
                Enter your domain without "http://" or "www" (e.g., example.com)
              </p>
            </div>

            <div className="form-actions">
              <button
                type="submit"
                disabled={actionStatus === 'processing'}
                className="btn btn-primary"
                aria-label="Set custom domain"
              >
                {actionStatus === 'processing'
                  ? 'Processing...'
                  : organization.customDomain
                  ? 'Update Domain'
                  : 'Set Custom Domain'}
              </button>
            </div>
          </form>
        </section>
      )}

      {/* Default Domain */}
      <section className="default-domain" aria-labelledby="default-domain-heading">
        <h2 id="default-domain-heading">Default Domain</h2>
        <p>
          Your organization is always accessible at:{' '}
          <strong>{organization.domain || `${organization.slug}.yourdomain.com`}</strong>
        </p>
      </section>

      {/* Instructions */}
      <section className="instructions" aria-labelledby="instructions-heading">
        <h2 id="instructions-heading">Setup Instructions</h2>
        <ol className="setup-steps">
          <li>Enter your custom domain above and click "Set Custom Domain"</li>
          <li>Add the provided DNS records to your domain's DNS settings</li>
          <li>Wait for DNS propagation (usually 15 minutes, up to 48 hours)</li>
          <li>Click "Verify Domain" to complete the setup</li>
          <li>Once verified, your organization will be accessible at your custom domain</li>
        </ol>
      </section>

      {/* Status Messages */}
      {actionStatus === 'success' && (
        <div className="success-message" role="status" aria-live="polite">
          {organization.customDomain ? 'Domain updated successfully!' : 'Domain removed successfully!'}
        </div>
      )}
      {actionStatus === 'error' && errorMessage && (
        <div className="error-message" role="alert" aria-live="assertive">
          {errorMessage}
        </div>
      )}
    </div>
  );
};

export default DomainConfig;
