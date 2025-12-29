import React, { useState } from 'react';
import { ReportConfig, ReportTemplate, BrandingConfig, ReportSection } from '../../types';
import { SectionSelector } from './SectionSelector';
import { TemplateLibrary } from './TemplateLibrary';
import { BrandingConfig as BrandingConfigComponent } from './BrandingConfig';

interface ReportBuilderProps {
  initialConfig?: Partial<ReportConfig>;
  onConfigChange?: (config: ReportConfig) => void;
  onSave?: (config: ReportConfig) => void;
  className?: string;
}

export const ReportBuilder: React.FC<ReportBuilderProps> = ({
  initialConfig,
  onConfigChange,
  onSave,
  className = '',
}) => {
  const [currentStep, setCurrentStep] = useState<'template' | 'sections' | 'branding' | 'review'>('template');
  const [selectedTemplate, setSelectedTemplate] = useState<ReportTemplate | null>(null);
  const [sections, setSections] = useState<ReportSection[]>([]);
  const [branding, setBranding] = useState<BrandingConfig>({
    companyName: '',
    primaryColor: '#0066cc',
    secondaryColor: '#666666',
    accentColor: '#ff6b00',
    fontFamily: 'Arial, sans-serif',
    includePageNumbers: true,
    includeDateGenerated: true,
  });
  const [reportTitle, setReportTitle] = useState('');
  const [reportSubtitle, setReportSubtitle] = useState('');

  const handleTemplateSelect = (template: ReportTemplate) => {
    setSelectedTemplate(template);
    setSections(template.sections);
    if (template.defaultBranding) {
      setBranding({ ...branding, ...template.defaultBranding });
    }
    setCurrentStep('sections');
  };

  const handleSectionsUpdate = (updatedSections: ReportSection[]) => {
    setSections(updatedSections);
  };

  const handleBrandingUpdate = (updatedBranding: BrandingConfig) => {
    setBranding(updatedBranding);
  };

  const handleSaveReport = () => {
    if (!selectedTemplate) return;

    const config: ReportConfig = {
      id: `report-${Date.now()}`,
      title: reportTitle,
      subtitle: reportSubtitle,
      template: selectedTemplate,
      sections,
      branding,
      dateRange: {
        from: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000),
        to: new Date(),
      },
      createdAt: new Date(),
      createdBy: 'current-user',
      version: '1.0',
    };

    if (onConfigChange) {
      onConfigChange(config);
    }
    if (onSave) {
      onSave(config);
    }
  };

  const steps = [
    { id: 'template', label: 'Choose Template', icon: '📋' },
    { id: 'sections', label: 'Select Sections', icon: '📑' },
    { id: 'branding', label: 'Configure Branding', icon: '🎨' },
    { id: 'review', label: 'Review & Save', icon: '✅' },
  ];

  return (
    <div className={`report-builder ${className}`} role="main" aria-label="Report Builder">
      {/* Progress Steps */}
      <nav aria-label="Report creation steps">
        <ol className="steps-progress" style={styles.stepsProgress}>
          {steps.map((step, index) => (
            <li
              key={step.id}
              className={currentStep === step.id ? 'active' : ''}
              style={{
                ...styles.step,
                ...(currentStep === step.id ? styles.stepActive : {}),
              }}
              aria-current={currentStep === step.id ? 'step' : undefined}
            >
              <span className="step-icon" aria-hidden="true">{step.icon}</span>
              <span className="step-label">{step.label}</span>
            </li>
          ))}
        </ol>
      </nav>

      {/* Step Content */}
      <div className="step-content" style={styles.stepContent}>
        {currentStep === 'template' && (
          <div role="region" aria-label="Template selection">
            <h2 style={styles.heading}>Choose a Report Template</h2>
            <TemplateLibrary onSelectTemplate={handleTemplateSelect} />
          </div>
        )}

        {currentStep === 'sections' && selectedTemplate && (
          <div role="region" aria-label="Section selection">
            <h2 style={styles.heading}>Customize Report Sections</h2>
            <SectionSelector
              sections={sections}
              onSectionsChange={handleSectionsUpdate}
            />
            <div style={styles.navigation}>
              <button
                onClick={() => setCurrentStep('template')}
                style={styles.button}
                aria-label="Go back to template selection"
              >
                Back
              </button>
              <button
                onClick={() => setCurrentStep('branding')}
                style={{ ...styles.button, ...styles.primaryButton }}
                aria-label="Continue to branding configuration"
              >
                Next
              </button>
            </div>
          </div>
        )}

        {currentStep === 'branding' && (
          <div role="region" aria-label="Branding configuration">
            <h2 style={styles.heading}>Configure Report Branding</h2>
            <BrandingConfigComponent
              config={branding}
              onChange={handleBrandingUpdate}
            />
            <div style={styles.navigation}>
              <button
                onClick={() => setCurrentStep('sections')}
                style={styles.button}
                aria-label="Go back to section selection"
              >
                Back
              </button>
              <button
                onClick={() => setCurrentStep('review')}
                style={{ ...styles.button, ...styles.primaryButton }}
                aria-label="Continue to review"
              >
                Next
              </button>
            </div>
          </div>
        )}

        {currentStep === 'review' && (
          <div role="region" aria-label="Review and save">
            <h2 style={styles.heading}>Review Report Configuration</h2>
            <div style={styles.reviewSection}>
              <div style={styles.formGroup}>
                <label htmlFor="report-title" style={styles.label}>
                  Report Title *
                </label>
                <input
                  id="report-title"
                  type="text"
                  value={reportTitle}
                  onChange={(e) => setReportTitle(e.target.value)}
                  style={styles.input}
                  required
                  aria-required="true"
                />
              </div>
              <div style={styles.formGroup}>
                <label htmlFor="report-subtitle" style={styles.label}>
                  Report Subtitle
                </label>
                <input
                  id="report-subtitle"
                  type="text"
                  value={reportSubtitle}
                  onChange={(e) => setReportSubtitle(e.target.value)}
                  style={styles.input}
                />
              </div>

              <div style={styles.summary}>
                <h3 style={styles.subheading}>Summary</h3>
                <dl style={styles.definitionList}>
                  <dt>Template:</dt>
                  <dd>{selectedTemplate?.name}</dd>
                  <dt>Sections:</dt>
                  <dd>{sections.filter(s => s.enabled).length} enabled</dd>
                  <dt>Company:</dt>
                  <dd>{branding.companyName || 'Not specified'}</dd>
                </dl>
              </div>
            </div>
            <div style={styles.navigation}>
              <button
                onClick={() => setCurrentStep('branding')}
                style={styles.button}
                aria-label="Go back to branding"
              >
                Back
              </button>
              <button
                onClick={handleSaveReport}
                disabled={!reportTitle}
                style={{
                  ...styles.button,
                  ...styles.saveButton,
                  ...(reportTitle ? {} : styles.buttonDisabled),
                }}
                aria-label="Save report configuration"
              >
                Save Report
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

const styles = {
  stepsProgress: {
    display: 'flex',
    justifyContent: 'space-between',
    listStyle: 'none',
    padding: '0',
    margin: '0 0 2rem 0',
    borderBottom: '2px solid #e0e0e0',
  },
  step: {
    flex: 1,
    padding: '1rem',
    textAlign: 'center' as const,
    borderBottom: '3px solid transparent',
    marginBottom: '-2px',
    cursor: 'pointer',
    transition: 'all 0.3s ease',
  },
  stepActive: {
    borderBottomColor: '#0066cc',
    fontWeight: 'bold' as const,
  },
  stepContent: {
    padding: '2rem',
    minHeight: '400px',
  },
  heading: {
    fontSize: '1.75rem',
    fontWeight: 'bold' as const,
    marginBottom: '1.5rem',
    color: '#333',
  },
  subheading: {
    fontSize: '1.25rem',
    fontWeight: 'bold' as const,
    marginBottom: '1rem',
    color: '#555',
  },
  navigation: {
    display: 'flex',
    justifyContent: 'space-between',
    marginTop: '2rem',
    paddingTop: '1.5rem',
    borderTop: '1px solid #e0e0e0',
  },
  button: {
    padding: '0.75rem 1.5rem',
    fontSize: '1rem',
    border: '1px solid #ccc',
    borderRadius: '4px',
    backgroundColor: '#fff',
    cursor: 'pointer',
    transition: 'all 0.2s ease',
  },
  primaryButton: {
    backgroundColor: '#0066cc',
    color: '#fff',
    borderColor: '#0066cc',
  },
  saveButton: {
    backgroundColor: '#28a745',
    color: '#fff',
    borderColor: '#28a745',
  },
  buttonDisabled: {
    opacity: 0.5,
    cursor: 'not-allowed',
  },
  reviewSection: {
    backgroundColor: '#f9f9f9',
    padding: '1.5rem',
    borderRadius: '8px',
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
  summary: {
    marginTop: '1.5rem',
    padding: '1rem',
    backgroundColor: '#fff',
    borderRadius: '4px',
    border: '1px solid #e0e0e0',
  },
  definitionList: {
    display: 'grid',
    gridTemplateColumns: '150px 1fr',
    gap: '0.75rem',
    margin: 0,
  },
};
