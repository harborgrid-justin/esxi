/**
 * Role Analysis Component
 * Analyzes and displays ARIA role usage statistics
 */

import React from 'react';
import { ARIARole } from '../../types';
import { implicitRoleAnalyzer } from '../../analyzers/ImplicitRoleAnalyzer';

export interface RoleAnalysisProps {
  target?: HTMLElement | Document;
}

export const RoleAnalysis: React.FC<RoleAnalysisProps> = ({ target }) => {
  const analyzeRoles = () => {
    if (!target) return { explicit: new Map(), implicit: new Map(), conflicts: [] };

    const explicitRoles = new Map<ARIARole, number>();
    const implicitRoles = new Map<ARIARole, number>();
    const conflicts: Array<{ element: HTMLElement; explicit: ARIARole; implicit: ARIARole }> = [];

    const elements = target instanceof Document
      ? Array.from(target.querySelectorAll('*'))
      : [target, ...Array.from(target.querySelectorAll('*'))];

    elements.forEach(el => {
      if (!(el instanceof HTMLElement)) return;

      const explicit = el.getAttribute('role') as ARIARole | null;
      const implicit = implicitRoleAnalyzer.getImplicitRole(el);

      if (explicit) {
        explicitRoles.set(explicit, (explicitRoles.get(explicit) || 0) + 1);
        if (implicit && explicit !== implicit) {
          conflicts.push({ element: el, explicit, implicit });
        }
      }

      if (implicit) {
        implicitRoles.set(implicit, (implicitRoles.get(implicit) || 0) + 1);
      }
    });

    return { explicit: explicitRoles, implicit: implicitRoles, conflicts };
  };

  const { explicit, implicit, conflicts } = analyzeRoles();

  return (
    <div className="role-analysis">
      <h3>Role Usage Analysis</h3>

      <div className="analysis-section">
        <h4>Explicit Roles</h4>
        {explicit.size === 0 ? (
          <div className="empty-state">No explicit ARIA roles found.</div>
        ) : (
          <ul className="role-list">
            {Array.from(explicit.entries())
              .sort((a, b) => b[1] - a[1])
              .map(([role, count]) => (
                <li key={role} className="role-list-item">
                  <span className="role-name">{role}</span>
                  <span className="role-count">{count}</span>
                </li>
              ))}
          </ul>
        )}
      </div>

      <div className="analysis-section">
        <h4>Implicit Roles</h4>
        {implicit.size === 0 ? (
          <div className="empty-state">No implicit roles detected.</div>
        ) : (
          <ul className="role-list">
            {Array.from(implicit.entries())
              .sort((a, b) => b[1] - a[1])
              .map(([role, count]) => (
                <li key={role} className="role-list-item">
                  <span className="role-name">{role}</span>
                  <span className="role-count">{count}</span>
                </li>
              ))}
          </ul>
        )}
      </div>

      {conflicts.length > 0 && (
        <div className="analysis-section conflicts">
          <h4>Role Conflicts ({conflicts.length})</h4>
          <ul className="conflict-list">
            {conflicts.map((conflict, index) => (
              <li key={index} className="conflict-item">
                <span className="element-tag">&lt;{conflict.element.tagName.toLowerCase()}&gt;</span>
                <span className="conflict-text">
                  Explicit: <strong>{conflict.explicit}</strong> vs Implicit: <strong>{conflict.implicit}</strong>
                </span>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
};
