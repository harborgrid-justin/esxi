/**
 * Word Document Analyzer Component
 * Display Word-specific analysis results
 */

import React from 'react';
import type { OfficeAnalysisResult } from '../../types/index.js';

export interface WordAnalyzerProps {
  result: OfficeAnalysisResult;
  className?: string;
}

export function WordAnalyzer({ result, className = '' }: WordAnalyzerProps): JSX.Element {
  return (
    <div className={`word-analyzer ${className}`}>
      <h3>Word Document Analysis</h3>

      <section className="word-analyzer__section">
        <h4>Document Structure</h4>
        <dl>
          <dt>Styles Used</dt>
          <dd>{result.hasStyles ? '✓ Yes' : '✗ No'}</dd>
          <dt>Headings</dt>
          <dd>{result.hasHeadings ? `✓ ${result.headings.length} found` : '✗ None'}</dd>
          <dt>Table of Contents</dt>
          <dd>{result.hasTableOfContents ? '✓ Present' : '✗ Missing'}</dd>
        </dl>
      </section>

      <section className="word-analyzer__section">
        <h4>Content Analysis</h4>
        <dl>
          <dt>Images</dt>
          <dd>{result.images.length} total</dd>
          <dt>Images with Alt Text</dt>
          <dd>{result.images.filter(i => i.hasAltText).length} of {result.images.length}</dd>
          <dt>Tables</dt>
          <dd>{result.tables.length} total</dd>
          <dt>Tables with Headers</dt>
          <dd>{result.tables.filter(t => t.hasHeaders).length} of {result.tables.length}</dd>
          <dt>Lists</dt>
          <dd>{result.lists.length} total</dd>
        </dl>
      </section>

      {result.headings.length > 0 && (
        <section className="word-analyzer__section">
          <h4>Heading Outline</h4>
          <ol className="word-analyzer__headings">
            {result.headings.map((heading, index) => (
              <li
                key={index}
                className={`heading-level-${heading.level}`}
                style={{ marginLeft: `${(heading.level - 1) * 20}px` }}
              >
                H{heading.level}: {heading.text}
              </li>
            ))}
          </ol>
        </section>
      )}
    </div>
  );
}
