/**
 * Attribute Checker Component
 * Checks ARIA attributes for validity and proper usage
 */

import React from 'react';
import { ValidationResult, ARIAAttribute } from '../../types';
import { useARIAValidation } from '../../hooks/useARIAValidation';

export interface AttributeCheckerProps {
  target?: HTMLElement | Document;
  result?: ValidationResult | null;
}

export const AttributeChecker: React.FC<AttributeCheckerProps> = ({ target, result }) => {
  const { getAttributeDefinition } = useARIAValidation();

  const getAriaElements = (): Array<{ element: HTMLElement; attributes: Attr[] }> => {
    if (!target) return [];

    const elements = target instanceof Document
      ? Array.from(target.querySelectorAll('[class*="aria-"]'))
      : [target];

    return elements
      .filter((el): el is HTMLElement => el instanceof HTMLElement)
      .map(element => ({
        element,
        attributes: Array.from(element.attributes).filter(attr => attr.name.startsWith('aria-')),
      }))
      .filter(item => item.attributes.length > 0);
  };

  const ariaElements = getAriaElements();
  const attributeErrors = result?.errors.filter(e => e.type === 'attribute') || [];
  const attributeWarnings = result?.warnings.filter(w => w.type === 'attribute') || [];

  return (
    <div className="attribute-checker">
      <h3>Attribute Validation</h3>

      {ariaElements.length === 0 && (
        <div className="empty-state">
          No elements with ARIA attributes found.
        </div>
      )}

      {ariaElements.map(({ element, attributes }, index) => {
        const role = element.getAttribute('role');

        return (
          <div key={index} className="attribute-item">
            <div className="attribute-item__header">
              <span className="element-tag">&lt;{element.tagName.toLowerCase()}&gt;</span>
              {role && <span className="role-badge">{role}</span>}
            </div>

            <div className="attribute-list">
              {attributes.map((attr, i) => {
                const definition = getAttributeDefinition(attr.name as ARIAAttribute);
                const attrErrors = attributeErrors.filter(e => e.attribute === attr.name);
                const attrWarnings = attributeWarnings.filter(w => w.attribute === attr.name);

                return (
                  <div key={i} className="attribute-row">
                    <div className="attribute-row__name">
                      <strong>{attr.name}:</strong> {attr.value}
                    </div>

                    {definition && (
                      <div className="attribute-row__details">
                        <div className="detail-text">
                          <strong>Type:</strong> {definition.type}
                        </div>
                        <div className="detail-text">
                          <strong>Value Type:</strong> {definition.valueType}
                        </div>
                        <div className="detail-text">{definition.description}</div>
                      </div>
                    )}

                    {attrErrors.length > 0 && (
                      <div className="attribute-row__errors">
                        {attrErrors.map((error, j) => (
                          <div key={j} className="error-message">
                            {error.message}
                          </div>
                        ))}
                      </div>
                    )}

                    {attrWarnings.length > 0 && (
                      <div className="attribute-row__warnings">
                        {attrWarnings.map((warning, j) => (
                          <div key={j} className="warning-message">
                            {warning.message}
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
