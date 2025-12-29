/**
 * Role Checker Component
 * Checks ARIA roles for validity and compliance
 */

import React from 'react';
import { ValidationResult, ARIARole } from '../../types';
import { useARIAValidation } from '../../hooks/useARIAValidation';

export interface RoleCheckerProps {
  target?: HTMLElement | Document;
  result?: ValidationResult | null;
}

export const RoleChecker: React.FC<RoleCheckerProps> = ({ target, result }) => {
  const { getImplicitRole, getRoleDefinition } = useARIAValidation();

  const getRoleElements = (): HTMLElement[] => {
    if (!target) return [];

    const elements = target instanceof Document
      ? Array.from(target.querySelectorAll('[role]'))
      : target.hasAttribute('role')
        ? [target]
        : Array.from(target.querySelectorAll('[role]'));

    return elements.filter((el): el is HTMLElement => el instanceof HTMLElement);
  };

  const roleElements = getRoleElements();
  const roleErrors = result?.errors.filter(e => e.type === 'role') || [];

  return (
    <div className="role-checker">
      <h3>Role Validation</h3>

      {roleElements.length === 0 && (
        <div className="empty-state">
          No elements with explicit ARIA roles found.
        </div>
      )}

      {roleElements.map((element, index) => {
        const role = element.getAttribute('role') as ARIARole;
        const implicitRole = getImplicitRole(element);
        const roleDefinition = getRoleDefinition(role);
        const elementErrors = roleErrors.filter(e => e.role === role);

        return (
          <div key={index} className="role-item">
            <div className="role-item__header">
              <span className="role-name">{role}</span>
              <span className="element-tag">&lt;{element.tagName.toLowerCase()}&gt;</span>
              {implicitRole && implicitRole !== role && (
                <span className="implicit-role">Implicit: {implicitRole}</span>
              )}
            </div>

            {roleDefinition && (
              <div className="role-item__details">
                <div className="detail-row">
                  <strong>Type:</strong> {roleDefinition.type}
                </div>
                <div className="detail-row">
                  <strong>Abstract:</strong> {roleDefinition.abstract ? 'Yes' : 'No'}
                </div>
                {roleDefinition.requiredAttributes.length > 0 && (
                  <div className="detail-row">
                    <strong>Required Attributes:</strong>{' '}
                    {roleDefinition.requiredAttributes.join(', ')}
                  </div>
                )}
                {roleDefinition.accessibleNameRequired && (
                  <div className="detail-row">
                    <strong>Accessible Name:</strong> Required
                  </div>
                )}
              </div>
            )}

            {elementErrors.length > 0 && (
              <div className="role-item__errors">
                {elementErrors.map((error, i) => (
                  <div key={i} className="error-message">
                    {error.message}
                  </div>
                ))}
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
};
