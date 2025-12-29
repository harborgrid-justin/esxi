/**
 * Checker Results Component
 * Displays comprehensive accessibility check results
 */

import React, { useState } from 'react';
import type { CheckerResult } from '../../types/index.js';
import { IssueList } from './IssueList.js';
import { StructureViewer } from './StructureViewer.js';
import { formatFileSize, formatDuration } from '../../utils/documentUtils.js';

export interface CheckerResultsProps {
  result: CheckerResult;
  className?: string;
}

export function CheckerResults({ result, className = '' }: CheckerResultsProps): JSX.Element {
  const [activeTab, setActiveTab] = useState<'summary' | 'issues' | 'structure'>('summary');

  const { complianceScore, summary, metadata, issues } = result;

  return (
    <div className={`checker-results ${className}`}>
      <div className="checker-results__header">
        <h2>Accessibility Check Results</h2>
        <div className="checker-results__meta">
          <span>{result.fileName}</span>
          <span>{formatFileSize(result.fileSize)}</span>
          <span>Checked: {result.checkedAt.toLocaleString()}</span>
          <span>Duration: {formatDuration(result.checkDuration)}</span>
        </div>
      </div>

      <div className="checker-results__score">
        <ScoreCard
          score={complianceScore.overall}
          label="Overall Accessibility Score"
          size="large"
        />
        <div className="checker-results__subscores">
          {complianceScore.wcagA !== undefined && (
            <ScoreCard score={complianceScore.wcagA} label="WCAG A" size="small" />
          )}
          {complianceScore.wcagAA !== undefined && (
            <ScoreCard score={complianceScore.wcagAA} label="WCAG AA" size="small" />
          )}
          {complianceScore.pdfua !== undefined && (
            <ScoreCard score={complianceScore.pdfua} label="PDF/UA" size="small" />
          )}
        </div>
      </div>

      <div className="checker-results__tabs" role="tablist">
        <button
          role="tab"
          aria-selected={activeTab === 'summary'}
          aria-controls="tab-summary"
          onClick={() => setActiveTab('summary')}
          className={activeTab === 'summary' ? 'active' : ''}
        >
          Summary
        </button>
        <button
          role="tab"
          aria-selected={activeTab === 'issues'}
          aria-controls="tab-issues"
          onClick={() => setActiveTab('issues')}
          className={activeTab === 'issues' ? 'active' : ''}
        >
          Issues ({issues.length})
        </button>
        <button
          role="tab"
          aria-selected={activeTab === 'structure'}
          aria-controls="tab-structure"
          onClick={() => setActiveTab('structure')}
          className={activeTab === 'structure' ? 'active' : ''}
        >
          Structure
        </button>
      </div>

      <div className="checker-results__content">
        {activeTab === 'summary' && (
          <div id="tab-summary" role="tabpanel">
            <SummaryPanel result={result} />
          </div>
        )}

        {activeTab === 'issues' && (
          <div id="tab-issues" role="tabpanel">
            <IssueList issues={issues} />
          </div>
        )}

        {activeTab === 'structure' && (
          <div id="tab-structure" role="tabpanel">
            {result.structure ? (
              <StructureViewer structure={result.structure} />
            ) : (
              <p>No structure information available</p>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

interface ScoreCardProps {
  score: number;
  label: string;
  size: 'small' | 'large';
}

function ScoreCard({ score, label, size }: ScoreCardProps): JSX.Element {
  const getScoreColor = (score: number): string => {
    if (score >= 90) return 'success';
    if (score >= 70) return 'warning';
    return 'error';
  };

  return (
    <div className={`score-card score-card--${size} score-card--${getScoreColor(score)}`}>
      <div className="score-card__value" aria-label={`${label}: ${score} out of 100`}>
        {Math.round(score)}
      </div>
      <div className="score-card__label">{label}</div>
    </div>
  );
}

interface SummaryPanelProps {
  result: CheckerResult;
}

function SummaryPanel({ result }: SummaryPanelProps): JSX.Element {
  const { summary, complianceScore, metadata } = result;

  return (
    <div className="summary-panel">
      <section className="summary-panel__section">
        <h3>Compliance Status</h3>
        <div className="summary-panel__badges">
          <ComplianceBadge label="WCAG A" passed={summary.isWCAGA} />
          <ComplianceBadge label="WCAG AA" passed={summary.isWCAGAA} />
          <ComplianceBadge label="WCAG AAA" passed={summary.isWCAGAAA} />
          {summary.isPDFUA !== undefined && (
            <ComplianceBadge label="PDF/UA" passed={summary.isPDFUA} />
          )}
        </div>
      </section>

      <section className="summary-panel__section">
        <h3>Issue Summary</h3>
        <div className="summary-panel__stats">
          <StatItem label="Critical" value={summary.criticalIssues} severity="critical" />
          <StatItem label="Errors" value={summary.errorIssues} severity="error" />
          <StatItem label="Warnings" value={summary.warningIssues} severity="warning" />
          <StatItem label="Info" value={summary.infoIssues} severity="info" />
        </div>
      </section>

      <section className="summary-panel__section">
        <h3>Document Features</h3>
        <div className="summary-panel__features">
          <FeatureItem label="Tagged Structure" present={summary.hasTaggedStructure} />
          <FeatureItem label="Metadata" present={summary.hasMetadata} />
          <FeatureItem label="Language" present={summary.hasLanguage} />
          <FeatureItem label="Alternative Text" present={summary.hasAlternativeText} />
        </div>
      </section>

      {summary.recommendedActions.length > 0 && (
        <section className="summary-panel__section">
          <h3>Recommended Actions</h3>
          <ol className="summary-panel__actions">
            {summary.recommendedActions.map((action, index) => (
              <li key={index}>{action}</li>
            ))}
          </ol>
        </section>
      )}

      <section className="summary-panel__section">
        <h3>Document Information</h3>
        <dl className="summary-panel__metadata">
          {metadata.title && (
            <>
              <dt>Title</dt>
              <dd>{metadata.title}</dd>
            </>
          )}
          {metadata.author && (
            <>
              <dt>Author</dt>
              <dd>{metadata.author}</dd>
            </>
          )}
          {metadata.language && (
            <>
              <dt>Language</dt>
              <dd>{metadata.language}</dd>
            </>
          )}
          {metadata.pageCount && (
            <>
              <dt>Pages</dt>
              <dd>{metadata.pageCount}</dd>
            </>
          )}
        </dl>
      </section>
    </div>
  );
}

interface ComplianceBadgeProps {
  label: string;
  passed: boolean;
}

function ComplianceBadge({ label, passed }: ComplianceBadgeProps): JSX.Element {
  return (
    <div className={`compliance-badge compliance-badge--${passed ? 'pass' : 'fail'}`}>
      <span className="compliance-badge__icon" aria-hidden="true">
        {passed ? '✓' : '✗'}
      </span>
      <span className="compliance-badge__label">{label}</span>
      <span className="compliance-badge__status">{passed ? 'Pass' : 'Fail'}</span>
    </div>
  );
}

interface StatItemProps {
  label: string;
  value: number;
  severity: string;
}

function StatItem({ label, value, severity }: StatItemProps): JSX.Element {
  return (
    <div className={`stat-item stat-item--${severity}`}>
      <div className="stat-item__value">{value}</div>
      <div className="stat-item__label">{label}</div>
    </div>
  );
}

interface FeatureItemProps {
  label: string;
  present: boolean;
}

function FeatureItem({ label, present }: FeatureItemProps): JSX.Element {
  return (
    <div className={`feature-item feature-item--${present ? 'present' : 'missing'}`}>
      <span className="feature-item__icon" aria-hidden="true">
        {present ? '✓' : '✗'}
      </span>
      <span className="feature-item__label">{label}</span>
    </div>
  );
}
