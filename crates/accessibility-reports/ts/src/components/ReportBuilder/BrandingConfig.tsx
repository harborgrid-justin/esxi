import React, { useState } from 'react';
import { BrandingConfig as BrandingConfigType } from '../../types';

interface BrandingConfigProps {
  config: BrandingConfigType;
  onChange: (config: BrandingConfigType) => void;
  className?: string;
}

export const BrandingConfig: React.FC<BrandingConfigProps> = ({
  config,
  onChange,
  className = '',
}) => {
  const [logoPreview, setLogoPreview] = useState<string | undefined>(config.logo);

  const handleChange = (field: keyof BrandingConfigType, value: any) => {
    onChange({
      ...config,
      [field]: value,
    });
  };

  const handleLogoUpload = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file) {
      const reader = new FileReader();
      reader.onload = (e) => {
        const result = e.target?.result as string;
        setLogoPreview(result);
        handleChange('logo', result);
      };
      reader.readAsDataURL(file);
    }
  };

  const fontOptions = [
    'Arial, sans-serif',
    'Helvetica, sans-serif',
    'Times New Roman, serif',
    'Georgia, serif',
    'Courier New, monospace',
    'Verdana, sans-serif',
    'Trebuchet MS, sans-serif',
    'Palatino, serif',
  ];

  return (
    <div className={`branding-config ${className}`} role="region" aria-label="Branding configuration">
      <div style={styles.container}>
        {/* Company Information */}
        <section style={styles.section}>
          <h3 style={styles.sectionTitle}>Company Information</h3>

          <div style={styles.formGroup}>
            <label htmlFor="company-name" style={styles.label}>
              Company Name *
            </label>
            <input
              id="company-name"
              type="text"
              value={config.companyName}
              onChange={(e) => handleChange('companyName', e.target.value)}
              style={styles.input}
              required
              aria-required="true"
            />
          </div>

          <div style={styles.formGroup}>
            <label htmlFor="logo-upload" style={styles.label}>
              Company Logo
            </label>
            <input
              id="logo-upload"
              type="file"
              accept="image/*"
              onChange={handleLogoUpload}
              style={styles.fileInput}
              aria-describedby="logo-help"
            />
            <p id="logo-help" style={styles.helpText}>
              Upload a PNG or JPG image (recommended: 200x60px)
            </p>
            {logoPreview && (
              <div style={styles.logoPreview}>
                <img
                  src={logoPreview}
                  alt="Company logo preview"
                  style={styles.logoImage}
                />
              </div>
            )}
          </div>
        </section>

        {/* Color Scheme */}
        <section style={styles.section}>
          <h3 style={styles.sectionTitle}>Color Scheme</h3>

          <div style={styles.colorGrid}>
            <div style={styles.formGroup}>
              <label htmlFor="primary-color" style={styles.label}>
                Primary Color
              </label>
              <div style={styles.colorInputGroup}>
                <input
                  id="primary-color"
                  type="color"
                  value={config.primaryColor}
                  onChange={(e) => handleChange('primaryColor', e.target.value)}
                  style={styles.colorInput}
                  aria-label="Primary color picker"
                />
                <input
                  type="text"
                  value={config.primaryColor}
                  onChange={(e) => handleChange('primaryColor', e.target.value)}
                  style={styles.colorTextInput}
                  pattern="^#[0-9A-Fa-f]{6}$"
                  aria-label="Primary color hex value"
                />
              </div>
            </div>

            <div style={styles.formGroup}>
              <label htmlFor="secondary-color" style={styles.label}>
                Secondary Color
              </label>
              <div style={styles.colorInputGroup}>
                <input
                  id="secondary-color"
                  type="color"
                  value={config.secondaryColor}
                  onChange={(e) => handleChange('secondaryColor', e.target.value)}
                  style={styles.colorInput}
                  aria-label="Secondary color picker"
                />
                <input
                  type="text"
                  value={config.secondaryColor}
                  onChange={(e) => handleChange('secondaryColor', e.target.value)}
                  style={styles.colorTextInput}
                  pattern="^#[0-9A-Fa-f]{6}$"
                  aria-label="Secondary color hex value"
                />
              </div>
            </div>

            <div style={styles.formGroup}>
              <label htmlFor="accent-color" style={styles.label}>
                Accent Color
              </label>
              <div style={styles.colorInputGroup}>
                <input
                  id="accent-color"
                  type="color"
                  value={config.accentColor}
                  onChange={(e) => handleChange('accentColor', e.target.value)}
                  style={styles.colorInput}
                  aria-label="Accent color picker"
                />
                <input
                  type="text"
                  value={config.accentColor}
                  onChange={(e) => handleChange('accentColor', e.target.value)}
                  style={styles.colorTextInput}
                  pattern="^#[0-9A-Fa-f]{6}$"
                  aria-label="Accent color hex value"
                />
              </div>
            </div>
          </div>

          {/* Color Preview */}
          <div style={styles.colorPreview}>
            <h4 style={styles.previewTitle}>Color Preview</h4>
            <div style={styles.colorSwatches}>
              <div style={styles.swatch}>
                <div
                  style={{ ...styles.swatchColor, backgroundColor: config.primaryColor }}
                  aria-label={`Primary color: ${config.primaryColor}`}
                />
                <span style={styles.swatchLabel}>Primary</span>
              </div>
              <div style={styles.swatch}>
                <div
                  style={{ ...styles.swatchColor, backgroundColor: config.secondaryColor }}
                  aria-label={`Secondary color: ${config.secondaryColor}`}
                />
                <span style={styles.swatchLabel}>Secondary</span>
              </div>
              <div style={styles.swatch}>
                <div
                  style={{ ...styles.swatchColor, backgroundColor: config.accentColor }}
                  aria-label={`Accent color: ${config.accentColor}`}
                />
                <span style={styles.swatchLabel}>Accent</span>
              </div>
            </div>
          </div>
        </section>

        {/* Typography */}
        <section style={styles.section}>
          <h3 style={styles.sectionTitle}>Typography</h3>

          <div style={styles.formGroup}>
            <label htmlFor="font-family" style={styles.label}>
              Font Family
            </label>
            <select
              id="font-family"
              value={config.fontFamily}
              onChange={(e) => handleChange('fontFamily', e.target.value)}
              style={styles.select}
            >
              {fontOptions.map((font) => (
                <option key={font} value={font}>
                  {font.split(',')[0]}
                </option>
              ))}
            </select>
            <p style={{ ...styles.sampleText, fontFamily: config.fontFamily }}>
              Sample text in selected font
            </p>
          </div>
        </section>

        {/* Header & Footer */}
        <section style={styles.section}>
          <h3 style={styles.sectionTitle}>Header & Footer</h3>

          <div style={styles.formGroup}>
            <label htmlFor="header-text" style={styles.label}>
              Header Text
            </label>
            <input
              id="header-text"
              type="text"
              value={config.headerText || ''}
              onChange={(e) => handleChange('headerText', e.target.value)}
              style={styles.input}
              placeholder="Optional header text"
            />
          </div>

          <div style={styles.formGroup}>
            <label htmlFor="footer-text" style={styles.label}>
              Footer Text
            </label>
            <input
              id="footer-text"
              type="text"
              value={config.footerText || ''}
              onChange={(e) => handleChange('footerText', e.target.value)}
              style={styles.input}
              placeholder="Optional footer text"
            />
          </div>

          <div style={styles.formGroup}>
            <label htmlFor="watermark" style={styles.label}>
              Watermark Text
            </label>
            <input
              id="watermark"
              type="text"
              value={config.watermark || ''}
              onChange={(e) => handleChange('watermark', e.target.value)}
              style={styles.input}
              placeholder="Optional watermark (e.g., CONFIDENTIAL)"
            />
          </div>
        </section>

        {/* Options */}
        <section style={styles.section}>
          <h3 style={styles.sectionTitle}>Document Options</h3>

          <div style={styles.checkboxGroup}>
            <label style={styles.checkboxLabel}>
              <input
                type="checkbox"
                checked={config.includePageNumbers}
                onChange={(e) => handleChange('includePageNumbers', e.target.checked)}
                style={styles.checkbox}
              />
              <span>Include page numbers</span>
            </label>

            <label style={styles.checkboxLabel}>
              <input
                type="checkbox"
                checked={config.includeDateGenerated}
                onChange={(e) => handleChange('includeDateGenerated', e.target.checked)}
                style={styles.checkbox}
              />
              <span>Include generation date</span>
            </label>
          </div>
        </section>
      </div>
    </div>
  );
};

const styles = {
  container: {
    maxWidth: '800px',
  },
  section: {
    marginBottom: '2rem',
    padding: '1.5rem',
    backgroundColor: '#f9f9f9',
    borderRadius: '8px',
    border: '1px solid #e0e0e0',
  },
  sectionTitle: {
    margin: '0 0 1.5rem 0',
    fontSize: '1.25rem',
    fontWeight: 'bold' as const,
    color: '#333',
    borderBottom: '2px solid #0066cc',
    paddingBottom: '0.5rem',
  },
  formGroup: {
    marginBottom: '1.5rem',
  },
  label: {
    display: 'block',
    marginBottom: '0.5rem',
    fontWeight: 'bold' as const,
    color: '#333',
  },
  input: {
    width: '100%',
    padding: '0.75rem',
    fontSize: '1rem',
    border: '1px solid #ccc',
    borderRadius: '4px',
    boxSizing: 'border-box' as const,
  },
  select: {
    width: '100%',
    padding: '0.75rem',
    fontSize: '1rem',
    border: '1px solid #ccc',
    borderRadius: '4px',
    boxSizing: 'border-box' as const,
  },
  fileInput: {
    width: '100%',
    padding: '0.5rem',
    fontSize: '1rem',
  },
  helpText: {
    margin: '0.5rem 0 0 0',
    fontSize: '0.875rem',
    color: '#666',
  },
  logoPreview: {
    marginTop: '1rem',
    padding: '1rem',
    backgroundColor: '#fff',
    border: '1px solid #e0e0e0',
    borderRadius: '4px',
    textAlign: 'center' as const,
  },
  logoImage: {
    maxWidth: '200px',
    maxHeight: '60px',
    objectFit: 'contain' as const,
  },
  colorGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
    gap: '1rem',
    marginBottom: '1.5rem',
  },
  colorInputGroup: {
    display: 'flex',
    gap: '0.5rem',
    alignItems: 'center',
  },
  colorInput: {
    width: '60px',
    height: '40px',
    border: '1px solid #ccc',
    borderRadius: '4px',
    cursor: 'pointer',
  },
  colorTextInput: {
    flex: 1,
    padding: '0.5rem',
    fontSize: '1rem',
    border: '1px solid #ccc',
    borderRadius: '4px',
    fontFamily: 'monospace',
  },
  colorPreview: {
    padding: '1rem',
    backgroundColor: '#fff',
    borderRadius: '4px',
    border: '1px solid #e0e0e0',
  },
  previewTitle: {
    margin: '0 0 1rem 0',
    fontSize: '1rem',
    fontWeight: 'bold' as const,
  },
  colorSwatches: {
    display: 'flex',
    gap: '1rem',
    justifyContent: 'space-around',
  },
  swatch: {
    textAlign: 'center' as const,
  },
  swatchColor: {
    width: '80px',
    height: '80px',
    borderRadius: '8px',
    border: '2px solid #e0e0e0',
    marginBottom: '0.5rem',
  },
  swatchLabel: {
    display: 'block',
    fontSize: '0.875rem',
    color: '#666',
  },
  sampleText: {
    marginTop: '0.5rem',
    padding: '0.75rem',
    backgroundColor: '#fff',
    border: '1px solid #e0e0e0',
    borderRadius: '4px',
    fontSize: '1.125rem',
  },
  checkboxGroup: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '1rem',
  },
  checkboxLabel: {
    display: 'flex',
    alignItems: 'center',
    gap: '0.75rem',
    cursor: 'pointer',
    fontSize: '1rem',
  },
  checkbox: {
    width: '20px',
    height: '20px',
    cursor: 'pointer',
  },
};
