/**
 * PowerPoint Analyzer Component
 * Display PowerPoint-specific analysis results
 */

import React from 'react';
import type { OfficeAnalysisResult } from '../../types/index.js';

export interface PowerPointAnalyzerProps {
  result: OfficeAnalysisResult;
  className?: string;
}

export function PowerPointAnalyzer({ result, className = '' }: PowerPointAnalyzerProps): JSX.Element {
  return (
    <div className={`powerpoint-analyzer ${className}`}>
      <h3>PowerPoint Presentation Analysis</h3>

      <section className="powerpoint-analyzer__section">
        <h4>Slide Accessibility</h4>
        <dl>
          <dt>Images</dt>
          <dd>{result.images.length} total</dd>
          <dt>Images with Alt Text</dt>
          <dd>
            {result.images.filter(i => i.hasAltText).length} of {result.images.length}
            {result.hasAltText ? ' ✓' : ' ✗'}
          </dd>
          <dt>Tables</dt>
          <dd>{result.tables.length} total</dd>
          <dt>Tables with Headers</dt>
          <dd>
            {result.tables.filter(t => t.hasHeaders).length} of {result.tables.length}
          </dd>
        </dl>
      </section>

      <section className="powerpoint-analyzer__section">
        <h4>Best Practices</h4>
        <ul>
          <li>✓ Ensure every slide has a unique title</li>
          <li>✓ Use built-in slide layouts</li>
          <li>✓ Add alt text to all images, charts, and SmartArt</li>
          <li>✓ Provide sufficient color contrast (4.5:1 for text)</li>
          <li>✓ Use Reading Order pane to set correct order</li>
          <li>✓ Avoid auto-playing animations</li>
          <li>✓ Mark decorative images as such</li>
        </ul>
      </section>

      <section className="powerpoint-analyzer__section">
        <h4>Reading Order</h4>
        <p>
          Use the Selection Pane (Home &gt; Arrange &gt; Selection Pane) to verify
          and adjust the reading order of objects on each slide.
        </p>
        <p>
          Objects are read from bottom to top in the Selection Pane list.
        </p>
      </section>

      {result.images.length > 0 && (
        <section className="powerpoint-analyzer__section">
          <h4>Images by Slide</h4>
          <table>
            <thead>
              <tr>
                <th>Slide</th>
                <th>Images</th>
                <th>With Alt Text</th>
              </tr>
            </thead>
            <tbody>
              {Array.from(new Set(result.images.map(i => i.page))).map(page => {
                const slideImages = result.images.filter(i => i.page === page);
                const withAlt = slideImages.filter(i => i.hasAltText).length;
                return (
                  <tr key={page}>
                    <td>Slide {page}</td>
                    <td>{slideImages.length}</td>
                    <td>
                      {withAlt} of {slideImages.length}
                      {withAlt === slideImages.length ? ' ✓' : ' ✗'}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </section>
      )}
    </div>
  );
}
