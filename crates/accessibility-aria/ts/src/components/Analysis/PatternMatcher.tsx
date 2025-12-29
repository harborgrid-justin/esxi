/**
 * Pattern Matcher Component
 * Matches ARIA design patterns in the DOM
 */

import React from 'react';
import { ARIA_PATTERNS } from '../../rules/ARIAPatterns';
import { ARIARole } from '../../types';

export interface PatternMatcherProps {
  target?: HTMLElement | Document;
}

export const PatternMatcher: React.FC<PatternMatcherProps> = ({ target }) => {
  const matchPatterns = () => {
    if (!target) return [];

    const matches: Array<{ pattern: string; elements: HTMLElement[]; complete: boolean }> = [];

    ARIA_PATTERNS.forEach(pattern => {
      const elements: HTMLElement[] = [];
      pattern.roles.forEach(role => {
        const selector = `[role="${role}"]`;
        const found = target instanceof Document
          ? target.querySelectorAll(selector)
          : target.matches(selector)
            ? [target]
            : target.querySelectorAll(selector);

        found.forEach(el => {
          if (el instanceof HTMLElement) {
            elements.push(el);
          }
        });
      });

      if (elements.length > 0) {
        // Check if pattern is complete
        const hasAllRoles = pattern.roles.every(role =>
          elements.some(el => el.getAttribute('role') === role)
        );

        matches.push({
          pattern: pattern.name,
          elements,
          complete: hasAllRoles,
        });
      }
    });

    return matches;
  };

  const matches = matchPatterns();

  return (
    <div className="pattern-matcher">
      <h3>Design Pattern Detection</h3>

      {matches.length === 0 ? (
        <div className="empty-state">No ARIA design patterns detected.</div>
      ) : (
        <div className="pattern-list">
          {matches.map((match, index) => (
            <div key={index} className={`pattern-item ${match.complete ? 'complete' : 'incomplete'}`}>
              <div className="pattern-header">
                <h4>{match.pattern}</h4>
                <span className={`status-badge ${match.complete ? 'complete' : 'incomplete'}`}>
                  {match.complete ? 'Complete' : 'Incomplete'}
                </span>
              </div>

              <div className="pattern-elements">
                <strong>Found Elements:</strong> {match.elements.length}
                <ul>
                  {match.elements.slice(0, 5).map((el, i) => (
                    <li key={i}>
                      &lt;{el.tagName.toLowerCase()}&gt; with role="{el.getAttribute('role')}"
                    </li>
                  ))}
                  {match.elements.length > 5 && (
                    <li>...and {match.elements.length - 5} more</li>
                  )}
                </ul>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};
