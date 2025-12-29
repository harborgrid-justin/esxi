/**
 * Analysis Progress Component
 * Displays progress during document analysis
 */

import React from 'react';
import type { CheckerProgress } from '../../types/index.js';

export interface AnalysisProgressProps {
  progress: CheckerProgress;
  fileName?: string;
  className?: string;
}

export function AnalysisProgress({
  progress,
  fileName,
  className = ''
}: AnalysisProgressProps): JSX.Element {
  return (
    <div className={`analysis-progress ${className}`} role="status" aria-live="polite">
      <div className="analysis-progress__header">
        <h2>Analyzing Document</h2>
        {fileName && <p className="analysis-progress__filename">{fileName}</p>}
      </div>

      <div className="analysis-progress__bar-container">
        <div
          className="analysis-progress__bar"
          role="progressbar"
          aria-valuenow={progress.progress}
          aria-valuemin={0}
          aria-valuemax={100}
          aria-label={`Analysis progress: ${progress.progress}%`}
        >
          <div
            className="analysis-progress__bar-fill"
            style={{ width: `${progress.progress}%` }}
          />
        </div>
        <div className="analysis-progress__percentage" aria-hidden="true">
          {progress.progress}%
        </div>
      </div>

      <div className="analysis-progress__status">
        <div className="analysis-progress__stage">{getStageLabel(progress.stage)}</div>
        <div className="analysis-progress__message">{progress.message}</div>
      </div>

      <div className="analysis-progress__steps">
        <StepIndicator stage={progress.stage} step="uploading" label="Uploading" />
        <StepIndicator stage={progress.stage} step="parsing" label="Parsing" />
        <StepIndicator stage={progress.stage} step="analyzing_structure" label="Analyzing" />
        <StepIndicator stage={progress.stage} step="validating_tags" label="Validating" />
        <StepIndicator stage={progress.stage} step="checking_metadata" label="Metadata" />
        <StepIndicator stage={progress.stage} step="checking_images" label="Images" />
        <StepIndicator stage={progress.stage} step="generating_report" label="Report" />
      </div>
    </div>
  );
}

interface StepIndicatorProps {
  stage: string;
  step: string;
  label: string;
}

function StepIndicator({ stage, step, label }: StepIndicatorProps): JSX.Element {
  const stageOrder = [
    'uploading',
    'parsing',
    'analyzing_structure',
    'validating_tags',
    'checking_metadata',
    'checking_images',
    'checking_forms',
    'checking_reading_order',
    'generating_report',
    'complete'
  ];

  const currentIndex = stageOrder.indexOf(stage);
  const stepIndex = stageOrder.indexOf(step);

  const status = stepIndex < currentIndex ? 'complete' : stepIndex === currentIndex ? 'active' : 'pending';

  return (
    <div className={`step-indicator step-indicator--${status}`}>
      <div className="step-indicator__icon" aria-hidden="true">
        {status === 'complete' && '✓'}
        {status === 'active' && '●'}
        {status === 'pending' && '○'}
      </div>
      <div className="step-indicator__label">{label}</div>
    </div>
  );
}

function getStageLabel(stage: string): string {
  const labels: Record<string, string> = {
    uploading: 'Uploading',
    parsing: 'Parsing Document',
    analyzing_structure: 'Analyzing Structure',
    validating_tags: 'Validating Tags',
    checking_metadata: 'Checking Metadata',
    checking_images: 'Checking Images',
    checking_forms: 'Checking Forms',
    checking_reading_order: 'Checking Reading Order',
    generating_report: 'Generating Report',
    complete: 'Complete',
    error: 'Error'
  };

  return labels[stage] || stage;
}
