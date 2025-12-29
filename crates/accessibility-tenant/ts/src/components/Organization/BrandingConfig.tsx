/**
 * Branding Configuration Component
 * Configure organization branding (logo, colors, CSS)
 */

import React, { useState, useEffect } from 'react';
import { useForm } from 'react-hook-form';
import { useOrganization } from '../../hooks/useOrganization';
import { usePermissions } from '../../hooks/usePermissions';
import { OrganizationBranding } from '../../types';
import OrganizationService from '../../services/OrganizationService';

interface BrandingConfigProps {
  organizationService: OrganizationService;
  organizationId: string;
  className?: string;
}

interface BrandingFormData {
  primaryColor: string;
  secondaryColor: string;
  customCSS: string;
}

export const BrandingConfig: React.FC<BrandingConfigProps> = ({
  organizationService,
  organizationId,
  className,
}) => {
  const { organization, updateBranding, uploadLogo, isLoading } = useOrganization(
    organizationService,
    organizationId
  );
  const { can } = usePermissions();
  const [saveStatus, setSaveStatus] = useState<'idle' | 'saving' | 'success' | 'error'>('idle');
  const [errorMessage, setErrorMessage] = useState<string>('');
  const [logoPreview, setLogoPreview] = useState<string>('');
  const [selectedLogo, setSelectedLogo] = useState<File | null>(null);

  const canManage = can.manageOrganization(organizationId);

  const {
    register,
    handleSubmit,
    reset,
    formState: { errors, isDirty },
  } = useForm<BrandingFormData>();

  useEffect(() => {
    if (organization?.branding) {
      reset({
        primaryColor: organization.branding.primaryColor || '#000000',
        secondaryColor: organization.branding.secondaryColor || '#ffffff',
        customCSS: organization.branding.customCSS || '',
      });
      if (organization.branding.logoUrl) {
        setLogoPreview(organization.branding.logoUrl);
      }
    }
  }, [organization, reset]);

  const handleLogoChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      if (!file.type.startsWith('image/')) {
        setErrorMessage('Please select an image file');
        return;
      }
      if (file.size > 5 * 1024 * 1024) {
        setErrorMessage('Image must be less than 5MB');
        return;
      }
      setSelectedLogo(file);
      const reader = new FileReader();
      reader.onloadend = () => {
        setLogoPreview(reader.result as string);
      };
      reader.readAsDataURL(file);
    }
  };

  const onSubmit = async (data: BrandingFormData) => {
    if (!canManage) {
      setErrorMessage('You do not have permission to update branding');
      setSaveStatus('error');
      return;
    }

    setSaveStatus('saving');
    setErrorMessage('');

    try {
      // Upload logo if selected
      if (selectedLogo) {
        await uploadLogo.mutateAsync(selectedLogo);
      }

      // Update branding
      const branding: Partial<OrganizationBranding> = {
        primaryColor: data.primaryColor,
        secondaryColor: data.secondaryColor,
        customCSS: data.customCSS || undefined,
      };

      await updateBranding.mutateAsync(branding);

      setSaveStatus('success');
      setSelectedLogo(null);
      setTimeout(() => setSaveStatus('idle'), 3000);
    } catch (error) {
      setErrorMessage(error instanceof Error ? error.message : 'Failed to save branding');
      setSaveStatus('error');
    }
  };

  if (isLoading) {
    return (
      <div className={className} role="status" aria-label="Loading branding configuration">
        <div className="loading-spinner">Loading...</div>
      </div>
    );
  }

  if (!organization || !organization.settings.customBrandingEnabled) {
    return (
      <div className={className} role="alert">
        <p>Custom branding is not enabled for this organization</p>
      </div>
    );
  }

  return (
    <div className={className}>
      <header className="config-header">
        <h1>Branding Configuration</h1>
        <p className="subtitle">Customize your organization's appearance</p>
      </header>

      <form onSubmit={handleSubmit(onSubmit)} className="branding-form">
        {/* Logo Upload */}
        <section className="form-section" aria-labelledby="logo-heading">
          <h2 id="logo-heading">Logo</h2>

          <div className="logo-upload">
            {logoPreview && (
              <div className="logo-preview">
                <img src={logoPreview} alt="Organization logo preview" />
              </div>
            )}
            <div className="upload-control">
              <label htmlFor="logo-upload" className="btn btn-secondary">
                {logoPreview ? 'Change Logo' : 'Upload Logo'}
              </label>
              <input
                id="logo-upload"
                type="file"
                accept="image/*"
                onChange={handleLogoChange}
                disabled={!canManage}
                className="file-input"
                aria-label="Upload organization logo"
              />
              <p className="help-text">PNG, JPG, or SVG. Max 5MB. Recommended: 200x200px</p>
            </div>
          </div>
        </section>

        {/* Color Configuration */}
        <section className="form-section" aria-labelledby="colors-heading">
          <h2 id="colors-heading">Colors</h2>

          <div className="color-inputs">
            <div className="form-group">
              <label htmlFor="primary-color">Primary Color</label>
              <div className="color-input-group">
                <input
                  id="primary-color"
                  type="color"
                  {...register('primaryColor')}
                  disabled={!canManage}
                  className="color-picker"
                  aria-label="Select primary color"
                />
                <input
                  type="text"
                  {...register('primaryColor', {
                    pattern: {
                      value: /^#[0-9A-Fa-f]{6}$/,
                      message: 'Invalid color format',
                    },
                  })}
                  disabled={!canManage}
                  className="color-text"
                  placeholder="#000000"
                  aria-label="Primary color hex code"
                />
              </div>
              {errors.primaryColor && (
                <p className="error-message" role="alert">
                  {errors.primaryColor.message}
                </p>
              )}
            </div>

            <div className="form-group">
              <label htmlFor="secondary-color">Secondary Color</label>
              <div className="color-input-group">
                <input
                  id="secondary-color"
                  type="color"
                  {...register('secondaryColor')}
                  disabled={!canManage}
                  className="color-picker"
                  aria-label="Select secondary color"
                />
                <input
                  type="text"
                  {...register('secondaryColor', {
                    pattern: {
                      value: /^#[0-9A-Fa-f]{6}$/,
                      message: 'Invalid color format',
                    },
                  })}
                  disabled={!canManage}
                  className="color-text"
                  placeholder="#ffffff"
                  aria-label="Secondary color hex code"
                />
              </div>
              {errors.secondaryColor && (
                <p className="error-message" role="alert">
                  {errors.secondaryColor.message}
                </p>
              )}
            </div>
          </div>
        </section>

        {/* Custom CSS */}
        <section className="form-section" aria-labelledby="custom-css-heading">
          <h2 id="custom-css-heading">Custom CSS</h2>

          <div className="form-group">
            <label htmlFor="custom-css">Custom Styles</label>
            <textarea
              id="custom-css"
              {...register('customCSS', {
                maxLength: {
                  value: 10000,
                  message: 'CSS must be less than 10,000 characters',
                },
              })}
              disabled={!canManage}
              className="form-textarea code-editor"
              rows={10}
              placeholder="/* Add your custom CSS here */"
              aria-invalid={errors.customCSS ? 'true' : 'false'}
              spellCheck={false}
            />
            {errors.customCSS && (
              <p className="error-message" role="alert">
                {errors.customCSS.message}
              </p>
            )}
            <p className="help-text">
              Advanced: Add custom CSS to override default styles
            </p>
          </div>
        </section>

        {/* Form Actions */}
        {canManage && (
          <div className="form-actions">
            <button
              type="submit"
              disabled={(! isDirty && !selectedLogo) || saveStatus === 'saving'}
              className="btn btn-primary"
              aria-label="Save branding"
            >
              {saveStatus === 'saving' ? 'Saving...' : 'Save Branding'}
            </button>
            <button
              type="button"
              onClick={() => {
                reset();
                setSelectedLogo(null);
                if (organization.branding.logoUrl) {
                  setLogoPreview(organization.branding.logoUrl);
                }
              }}
              disabled={(!isDirty && !selectedLogo) || saveStatus === 'saving'}
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
            Branding saved successfully!
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

export default BrandingConfig;
