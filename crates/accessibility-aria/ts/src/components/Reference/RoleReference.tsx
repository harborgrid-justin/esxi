/**
 * Role Reference Component
 * Reference documentation for ARIA roles
 */

import React, { useState } from 'react';
import { ARIARole } from '../../types';
import { ARIA_ROLES } from '../../rules/ARIARoles';

export const RoleReference: React.FC = () => {
  const [selectedRole, setSelectedRole] = useState<ARIARole | null>(null);
  const [filter, setFilter] = useState<string>('');

  const roles = Object.values(ARIA_ROLES).filter(
    role =>
      !role.abstract &&
      (filter === '' || role.name.toLowerCase().includes(filter.toLowerCase()))
  );

  const selectedDefinition = selectedRole ? ARIA_ROLES[selectedRole] : null;

  return (
    <div className="role-reference">
      <h3>ARIA Role Reference</h3>

      <input
        type="text"
        placeholder="Search roles..."
        value={filter}
        onChange={e => setFilter(e.target.value)}
        className="search-input"
      />

      <div className="reference-container">
        <div className="role-list">
          {roles.map(role => (
            <button
              key={role.name}
              className={`role-list-item ${selectedRole === role.name ? 'selected' : ''}`}
              onClick={() => setSelectedRole(role.name)}
            >
              {role.name}
              <span className="role-type">{role.type}</span>
            </button>
          ))}
        </div>

        <div className="role-details">
          {selectedDefinition ? (
            <>
              <h4>{selectedDefinition.name}</h4>

              <div className="detail-section">
                <strong>Type:</strong> {selectedDefinition.type}
              </div>

              <div className="detail-section">
                <strong>Superclass Roles:</strong>
                <div className="role-badges">
                  {selectedDefinition.superclassRoles.map(role => (
                    <span key={role} className="role-badge">
                      {role}
                    </span>
                  ))}
                </div>
              </div>

              {selectedDefinition.requiredAttributes.length > 0 && (
                <div className="detail-section">
                  <strong>Required Attributes:</strong>
                  <ul>
                    {selectedDefinition.requiredAttributes.map(attr => (
                      <li key={attr}>{attr}</li>
                    ))}
                  </ul>
                </div>
              )}

              <div className="detail-section">
                <strong>Supported Attributes:</strong>
                <div className="attribute-list">
                  {selectedDefinition.supportedAttributes.slice(0, 10).map(attr => (
                    <span key={attr} className="attribute-badge">
                      {attr}
                    </span>
                  ))}
                  {selectedDefinition.supportedAttributes.length > 10 && (
                    <span className="more-badge">
                      +{selectedDefinition.supportedAttributes.length - 10} more
                    </span>
                  )}
                </div>
              </div>

              {selectedDefinition.requiredContextRole.length > 0 && (
                <div className="detail-section">
                  <strong>Required Context:</strong>
                  <div className="role-badges">
                    {selectedDefinition.requiredContextRole.map(role => (
                      <span key={role} className="role-badge">
                        {role}
                      </span>
                    ))}
                  </div>
                </div>
              )}
            </>
          ) : (
            <div className="empty-state">Select a role to view details</div>
          )}
        </div>
      </div>
    </div>
  );
};
