/**
 * Semantic Checker Component
 * Checks semantic HTML structure and usage
 */

import React from 'react';
import { semanticAnalyzer } from '../../analyzers/SemanticAnalyzer';

export interface SemanticCheckerProps {
  target?: HTMLElement | Document;
}

export const SemanticChecker: React.FC<SemanticCheckerProps> = ({ target }) => {
  const analyzeSemantics = () => {
    if (!target) return [];

    const elements = target instanceof Document
      ? Array.from(target.querySelectorAll('*'))
      : [target];

    return elements
      .filter((el): el is HTMLElement => el instanceof HTMLElement)
      .map(el => semanticAnalyzer.analyze(el))
      .filter(analysis => analysis.semanticIssues.length > 0 || analysis.roleConflict);
  };

  const analyses = analyzeSemantics();

  return (
    <div className="semantic-checker">
      <h3>Semantic Structure Analysis</h3>

      {analyses.length === 0 ? (
        <div className="empty-state success">No semantic issues detected.</div>
      ) : (
        <div className="semantic-issues">
          {analyses.map((analysis, index) => (
            <div key={index} className="semantic-item">
              <div className="semantic-header">
                <span className="element-tag">&lt;{analysis.element}&gt;</span>
                {analysis.explicitRole && (
                  <span className="role-badge">role="{analysis.explicitRole}"</span>
                )}
                {analysis.implicitRole && (
                  <span className="implicit-badge">implicit: {analysis.implicitRole}</span>
                )}
              </div>

              {analysis.roleConflict && (
                <div className="conflict-notice">
                  Role conflict detected between explicit and implicit roles
                </div>
              )}

              {analysis.semanticIssues.length > 0 && (
                <div className="issues-list">
                  <strong>Issues:</strong>
                  <ul>
                    {analysis.semanticIssues.map((issue, i) => (
                      <li key={i}>{issue}</li>
                    ))}
                  </ul>
                </div>
              )}

              {analysis.recommendations.length > 0 && (
                <div className="recommendations-list">
                  <strong>Recommendations:</strong>
                  <ul>
                    {analysis.recommendations.map((rec, i) => (
                      <li key={i}>{rec}</li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
};
