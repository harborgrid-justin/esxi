import { useState, useCallback, useEffect } from 'react';
import { ReportConfig, ReportTemplate, ReportSection, BrandingConfig } from '../types';

interface UseReportBuilderReturn {
  config: ReportConfig | null;
  template: ReportTemplate | null;
  sections: ReportSection[];
  branding: BrandingConfig;
  isValid: boolean;
  errors: string[];
  setTemplate: (template: ReportTemplate) => void;
  updateSections: (sections: ReportSection[]) => void;
  updateBranding: (branding: BrandingConfig) => void;
  updateMetadata: (metadata: Partial<ReportConfig>) => void;
  buildConfig: () => ReportConfig | null;
  reset: () => void;
  saveDraft: () => void;
  loadDraft: () => boolean;
}

const DEFAULT_BRANDING: BrandingConfig = {
  companyName: '',
  primaryColor: '#0066cc',
  secondaryColor: '#666666',
  accentColor: '#ff6b00',
  fontFamily: 'Arial, sans-serif',
  includePageNumbers: true,
  includeDateGenerated: true,
};

const DRAFT_STORAGE_KEY = 'accessibility-report-draft';

/**
 * Custom hook for building accessibility reports
 * Manages state and validation for the report builder
 */
export const useReportBuilder = (): UseReportBuilderReturn => {
  const [config, setConfig] = useState<ReportConfig | null>(null);
  const [template, setTemplateState] = useState<ReportTemplate | null>(null);
  const [sections, setSections] = useState<ReportSection[]>([]);
  const [branding, setBranding] = useState<BrandingConfig>(DEFAULT_BRANDING);
  const [metadata, setMetadata] = useState<Partial<ReportConfig>>({});
  const [errors, setErrors] = useState<string[]>([]);

  // Validate configuration
  const validate = useCallback((): boolean => {
    const newErrors: string[] = [];

    if (!template) {
      newErrors.push('No template selected');
    }

    if (sections.filter((s) => s.enabled).length === 0) {
      newErrors.push('At least one section must be enabled');
    }

    if (!branding.companyName || branding.companyName.trim() === '') {
      newErrors.push('Company name is required');
    }

    if (!metadata.title || metadata.title.trim() === '') {
      newErrors.push('Report title is required');
    }

    setErrors(newErrors);
    return newErrors.length === 0;
  }, [template, sections, branding, metadata]);

  // Set template and initialize sections
  const setTemplate = useCallback((newTemplate: ReportTemplate) => {
    setTemplateState(newTemplate);
    setSections(newTemplate.sections);

    if (newTemplate.defaultBranding) {
      setBranding((prev) => ({
        ...prev,
        ...newTemplate.defaultBranding,
      }));
    }
  }, []);

  // Update sections
  const updateSections = useCallback((newSections: ReportSection[]) => {
    setSections(newSections);
  }, []);

  // Update branding
  const updateBranding = useCallback((newBranding: BrandingConfig) => {
    setBranding(newBranding);
  }, []);

  // Update metadata
  const updateMetadata = useCallback((newMetadata: Partial<ReportConfig>) => {
    setMetadata((prev) => ({
      ...prev,
      ...newMetadata,
    }));
  }, []);

  // Build final configuration
  const buildConfig = useCallback((): ReportConfig | null => {
    if (!validate()) {
      return null;
    }

    if (!template) {
      return null;
    }

    const now = new Date();
    const newConfig: ReportConfig = {
      id: `report-${now.getTime()}`,
      title: metadata.title || 'Accessibility Report',
      subtitle: metadata.subtitle,
      description: metadata.description,
      template,
      sections,
      branding,
      dateRange: metadata.dateRange || {
        from: new Date(now.getTime() - 30 * 24 * 60 * 60 * 1000),
        to: now,
      },
      createdAt: now,
      createdBy: metadata.createdBy || 'current-user',
      version: '1.0',
      filters: metadata.filters,
    };

    setConfig(newConfig);
    return newConfig;
  }, [validate, template, metadata, sections, branding]);

  // Reset to initial state
  const reset = useCallback(() => {
    setConfig(null);
    setTemplateState(null);
    setSections([]);
    setBranding(DEFAULT_BRANDING);
    setMetadata({});
    setErrors([]);
  }, []);

  // Save draft to localStorage
  const saveDraft = useCallback(() => {
    const draft = {
      template,
      sections,
      branding,
      metadata,
      savedAt: new Date().toISOString(),
    };

    try {
      localStorage.setItem(DRAFT_STORAGE_KEY, JSON.stringify(draft));
    } catch (error) {
      console.error('Failed to save draft:', error);
    }
  }, [template, sections, branding, metadata]);

  // Load draft from localStorage
  const loadDraft = useCallback((): boolean => {
    try {
      const draftJson = localStorage.getItem(DRAFT_STORAGE_KEY);
      if (!draftJson) {
        return false;
      }

      const draft = JSON.parse(draftJson);

      if (draft.template) {
        setTemplateState(draft.template);
      }
      if (draft.sections) {
        setSections(draft.sections);
      }
      if (draft.branding) {
        setBranding(draft.branding);
      }
      if (draft.metadata) {
        setMetadata(draft.metadata);
      }

      return true;
    } catch (error) {
      console.error('Failed to load draft:', error);
      return false;
    }
  }, []);

  // Auto-save draft periodically
  useEffect(() => {
    if (!template) return;

    const autoSaveInterval = setInterval(() => {
      saveDraft();
    }, 30000); // Auto-save every 30 seconds

    return () => clearInterval(autoSaveInterval);
  }, [template, saveDraft]);

  // Validate on changes
  useEffect(() => {
    if (template) {
      validate();
    }
  }, [template, sections, branding, metadata, validate]);

  return {
    config,
    template,
    sections,
    branding,
    isValid: errors.length === 0,
    errors,
    setTemplate,
    updateSections,
    updateBranding,
    updateMetadata,
    buildConfig,
    reset,
    saveDraft,
    loadDraft,
  };
};
