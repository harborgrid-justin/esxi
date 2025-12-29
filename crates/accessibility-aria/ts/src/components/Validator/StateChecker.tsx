/**
 * State Checker Component
 * Checks ARIA state attributes and their consistency
 */

import React from 'react';
import { ValidationResult, ARIAState } from '../../types';

export interface StateCheckerProps {
  target?: HTMLElement | Document;
  result?: ValidationResult | null;
}

export const StateChecker: React.FC<StateCheckerProps> = ({ target, result }) => {
  const stateAttributes: ARIAState[] = [
    'aria-busy',
    'aria-checked',
    'aria-disabled',
    'aria-expanded',
    'aria-grabbed',
    'aria-hidden',
    'aria-invalid',
    'aria-pressed',
    'aria-selected',
  ];

  const getStateElements = (): Array<{ element: HTMLElement; states: Attr[] }> => {
    if (!target) return [];

    const elements = target instanceof Document
      ? Array.from(target.querySelectorAll('[class*="aria-"]'))
      : [target];

    return elements
      .filter((el): el is HTMLElement => el instanceof HTMLElement)
      .map(element => ({
        element,
        states: Array.from(element.attributes).filter(attr =>
          stateAttributes.includes(attr.name as ARIAState)
        ),
      }))
      .filter(item => item.states.length > 0);
  };

  const stateElements = getStateElements();
  const stateErrors = result?.errors.filter(e => e.type === 'state') || [];
  const stateWarnings = result?.warnings.filter(w => w.type === 'state') || [];

  return (
    <div className="state-checker">
      <h3>State Validation</h3>

      {stateElements.length === 0 && (
        <div className="empty-state">
          No elements with ARIA state attributes found.
        </div>
      )}

      {stateElements.map(({ element, states }, index) => {
        const role = element.getAttribute('role');

        return (
          <div key={index} className="state-item">
            <div className="state-item__header">
              <span className="element-tag">&lt;{element.tagName.toLowerCase()}&gt;</span>
              {role && <span className="role-badge">{role}</span>}
            </div>

            <div className="state-list">
              {states.map((state, i) => {
                const stateErrors = stateErrors.filter(e => e.attribute === state.name);
                const stateWarnings = stateWarnings.filter(w => w.attribute === state.name);

                return (
                  <div key={i} className="state-row">
                    <div className="state-row__name">
                      <strong>{state.name}:</strong>{' '}
                      <span className={`state-value ${state.value}`}>{state.value}</span>
                    </div>

                    {stateErrors.length > 0 && (
                      <div className="state-row__errors">
                        {stateErrors.map((error, j) => (
                          <div key={j} className="error-message">
                            {error.message}
                          </div>
                        ))}
                      </div>
                    )}

                    {stateWarnings.length > 0 && (
                      <div className="state-row__warnings">
                        {stateWarnings.map((warning, j) => (
                          <div key={j} className="warning-message">
                            {warning.message}
                            {warning.suggestion && (
                              <div className="suggestion">Suggestion: {warning.suggestion}</div>
                            )}
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          </div>
        );
      })}
    </div>
  );
};
