/**
 * Excel Analyzer Component
 * Display Excel-specific analysis results
 */

import React from 'react';
import type { OfficeAnalysisResult } from '../../types/index.js';

export interface ExcelAnalyzerProps {
  result: OfficeAnalysisResult;
  className?: string;
}

export function ExcelAnalyzer({ result, className = '' }: ExcelAnalyzerProps): JSX.Element {
  return (
    <div className={`excel-analyzer ${className}`}>
      <h3>Excel Workbook Analysis</h3>

      <section className="excel-analyzer__section">
        <h4>Accessibility Features</h4>
        <dl>
          <dt>Alternative Text</dt>
          <dd>
            {result.hasAltText
              ? '✓ All images and charts have alt text'
              : '✗ Some images/charts missing alt text'}
          </dd>
          <dt>Tables</dt>
          <dd>{result.tables.length} data tables found</dd>
          <dt>Tables with Headers</dt>
          <dd>
            {result.tables.filter(t => t.hasHeaders).length} of {result.tables.length}
          </dd>
        </dl>
      </section>

      <section className="excel-analyzer__section">
        <h4>Content Summary</h4>
        <dl>
          <dt>Charts and Images</dt>
          <dd>{result.images.length}</dd>
          <dt>With Alt Text</dt>
          <dd>{result.images.filter(i => i.hasAltText).length}</dd>
        </dl>
      </section>

      <section className="excel-analyzer__section">
        <h4>Recommendations</h4>
        <ul>
          <li>Use Excel Tables (Insert &gt; Table) for structured data</li>
          <li>Provide descriptive sheet names</li>
          <li>Add alt text to all charts and images</li>
          <li>Avoid merged cells where possible</li>
          <li>Don't rely on color alone to convey information</li>
        </ul>
      </section>
    </div>
  );
}
