/**
 * Pattern Library Component
 * Reference library for ARIA design patterns
 */

import React, { useState } from 'react';
import { ARIA_PATTERNS } from '../../rules/ARIAPatterns';

export const PatternLibrary: React.FC = () => {
  const [selectedPattern, setSelectedPattern] = useState<string | null>(null);

  const pattern = selectedPattern
    ? ARIA_PATTERNS.find(p => p.name === selectedPattern)
    : null;

  return (
    <div className="pattern-library">
      <h3>ARIA Design Pattern Library</h3>

      <div className="library-container">
        <div className="pattern-list">
          {ARIA_PATTERNS.map(p => (
            <button
              key={p.name}
              className={`pattern-list-item ${selectedPattern === p.name ? 'selected' : ''}`}
              onClick={() => setSelectedPattern(p.name)}
            >
              {p.name}
            </button>
          ))}
        </div>

        <div className="pattern-details">
          {pattern ? (
            <>
              <h4>{pattern.name}</h4>
              <p className="pattern-description">{pattern.description}</p>

              <div className="detail-section">
                <strong>Roles Used:</strong>
                <div className="role-badges">
                  {pattern.roles.map(role => (
                    <span key={role} className="role-badge">
                      {role}
                    </span>
                  ))}
                </div>
              </div>

              <div className="detail-section">
                <strong>Keyboard Interaction:</strong>
                <table className="keyboard-table">
                  <thead>
                    <tr>
                      <th>Key</th>
                      <th>Action</th>
                    </tr>
                  </thead>
                  <tbody>
                    {pattern.keyboardInteraction.map((interaction, index) => (
                      <tr key={index}>
                        <td>
                          <code>{interaction.key}</code>
                        </td>
                        <td>{interaction.action}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>

              <div className="detail-section">
                <strong>Focus Management:</strong>
                <p>{pattern.focusManagement}</p>
              </div>

              <div className="detail-section">
                <strong>Example HTML:</strong>
                <pre className="code-block">
                  <code>{pattern.exampleHTML}</code>
                </pre>
              </div>
            </>
          ) : (
            <div className="empty-state">Select a pattern to view details</div>
          )}
        </div>
      </div>
    </div>
  );
};
