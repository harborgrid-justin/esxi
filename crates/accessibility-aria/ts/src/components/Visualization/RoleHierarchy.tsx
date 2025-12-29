/**
 * Role Hierarchy Component
 * Displays ARIA role inheritance hierarchy
 */

import React, { useState } from 'react';
import { ARIARole } from '../../types';
import { getRoleDefinition } from '../../rules/ARIARoles';

export interface RoleHierarchyProps {
  role: ARIARole;
}

export const RoleHierarchy: React.FC<RoleHierarchyProps> = ({ role }) => {
  const [expanded, setExpanded] = useState(true);

  const buildHierarchy = (currentRole: ARIARole): ARIARole[] => {
    const hierarchy: ARIARole[] = [currentRole];
    const definition = getRoleDefinition(currentRole);

    if (definition && definition.superclassRoles.length > 0) {
      definition.superclassRoles.forEach(superRole => {
        hierarchy.push(...buildHierarchy(superRole));
      });
    }

    return hierarchy;
  };

  const hierarchy = buildHierarchy(role);
  const definition = getRoleDefinition(role);

  return (
    <div className="role-hierarchy">
      <div className="hierarchy-header">
        <h4>Role: {role}</h4>
        <button onClick={() => setExpanded(!expanded)}>
          {expanded ? 'Collapse' : 'Expand'}
        </button>
      </div>

      {expanded && (
        <>
          <div className="hierarchy-chain">
            <strong>Inheritance Chain:</strong>
            <div className="chain">
              {hierarchy.reverse().map((r, index) => (
                <React.Fragment key={r}>
                  {index > 0 && <span className="arrow">→</span>}
                  <span className={`role-badge ${r === role ? 'current' : 'parent'}`}>{r}</span>
                </React.Fragment>
              ))}
            </div>
          </div>

          {definition && (
            <>
              {definition.subclassRoles.length > 0 && (
                <div className="subclasses">
                  <strong>Subclass Roles:</strong>
                  <div className="role-badges">
                    {definition.subclassRoles.map(subRole => (
                      <span key={subRole} className="role-badge subclass">
                        {subRole}
                      </span>
                    ))}
                  </div>
                </div>
              )}

              <div className="role-properties">
                <div className="property">
                  <strong>Type:</strong> {definition.type}
                </div>
                <div className="property">
                  <strong>Abstract:</strong> {definition.abstract ? 'Yes' : 'No'}
                </div>
                <div className="property">
                  <strong>Accessible Name Required:</strong>{' '}
                  {definition.accessibleNameRequired ? 'Yes' : 'No'}
                </div>
                <div className="property">
                  <strong>Children Presentational:</strong>{' '}
                  {definition.childrenPresentational ? 'Yes' : 'No'}
                </div>
              </div>
            </>
          )}
        </>
      )}
    </div>
  );
};
