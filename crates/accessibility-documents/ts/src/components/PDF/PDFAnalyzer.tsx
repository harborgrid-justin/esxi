/**
 * PDF Analyzer Component
 * PDF-specific analysis display
 */

import React from 'react';
import type { PDFAnalysisResult } from '../../types/index.js';

export interface PDFAnalyzerProps {
  result: PDFAnalysisResult;
  className?: string;
}

export function PDFAnalyzer({ result, className = '' }: PDFAnalyzerProps): JSX.Element {
  return (
    <div className={`pdf-analyzer ${className}`}>
      <h3>PDF Analysis</h3>

      <section className="pdf-analyzer__section">
        <h4>PDF/UA Compliance</h4>
        <dl>
          <dt>Tagged PDF</dt>
          <dd>{result.isTagged ? '✓ Yes' : '✗ No'}</dd>
          <dt>PDF Version</dt>
          <dd>{result.version}</dd>
          <dt>Page Count</dt>
          <dd>{result.pageCount}</dd>
          <dt>Structure Tree</dt>
          <dd>{result.hasStructureTree ? '✓ Present' : '✗ Missing'}</dd>
          <dt>Mark Info</dt>
          <dd>{result.hasMarkInfo ? '✓ Present' : '✗ Missing'}</dd>
        </dl>
      </section>

      <section className="pdf-analyzer__section">
        <h4>Fonts ({result.fonts.length})</h4>
        {result.fonts.length > 0 ? (
          <table>
            <thead>
              <tr>
                <th>Font Name</th>
                <th>Embedded</th>
                <th>Unicode Mapping</th>
              </tr>
            </thead>
            <tbody>
              {result.fonts.map((font, index) => (
                <tr key={index}>
                  <td>{font.name}</td>
                  <td>{font.embedded ? '✓' : '✗'}</td>
                  <td>{font.unicodeMapping ? '✓' : '✗'}</td>
                </tr>
              ))}
            </tbody>
          </table>
        ) : (
          <p>No font information available</p>
        )}
      </section>

      <section className="pdf-analyzer__section">
        <h4>Images ({result.images.length})</h4>
        <p>{result.images.filter(i => i.hasAltText).length} of {result.images.length} images have alt text</p>
      </section>

      <section className="pdf-analyzer__section">
        <h4>Forms ({result.forms.length})</h4>
        <p>{result.forms.filter(f => f.hasLabel).length} of {result.forms.length} form fields have labels</p>
      </section>
    </div>
  );
}
