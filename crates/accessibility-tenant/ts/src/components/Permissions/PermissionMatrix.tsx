/**
 * Permission Matrix Component
 * Display and manage permission matrix for roles
 */

import React from 'react';
import { Permission, ResourceType, PermissionAction } from '../../types';

interface PermissionMatrixProps {
  permissions: Permission[];
  onTogglePermission?: (resource: ResourceType, action: PermissionAction) => void;
  readOnly?: boolean;
  className?: string;
}

export const PermissionMatrix: React.FC<PermissionMatrixProps> = ({
  permissions,
  onTogglePermission,
  readOnly = false,
  className,
}) => {
  const resources = Object.values(ResourceType);
  const actions = Object.values(PermissionAction);

  const hasPermission = (resource: ResourceType, action: PermissionAction): boolean => {
    return permissions.some((p) => p.resource === resource && p.action === action);
  };

  const handleToggle = (resource: ResourceType, action: PermissionAction) => {
    if (!readOnly && onTogglePermission) {
      onTogglePermission(resource, action);
    }
  };

  return (
    <div className={className}>
      <header>
        <h2>Permission Matrix</h2>
        <p className="subtitle">Configure granular permissions for this role</p>
      </header>

      <div className="matrix-container">
        <table className="permission-matrix">
          <thead>
            <tr>
              <th>Resource</th>
              {actions.map((action) => (
                <th key={action}>{action}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {resources.map((resource) => (
              <tr key={resource}>
                <td className="resource-name">{resource.replace(/_/g, ' ')}</td>
                {actions.map((action) => {
                  const isChecked = hasPermission(resource, action);
                  const cellId = `perm-${resource}-${action}`;
                  return (
                    <td key={action} className="permission-cell">
                      {readOnly ? (
                        <span
                          className={`permission-indicator ${isChecked ? 'granted' : 'denied'}`}
                          aria-label={`${resource} ${action}: ${isChecked ? 'Granted' : 'Denied'}`}
                        >
                          {isChecked ? '✓' : '—'}
                        </span>
                      ) : (
                        <label htmlFor={cellId} className="checkbox-label">
                          <input
                            id={cellId}
                            type="checkbox"
                            checked={isChecked}
                            onChange={() => handleToggle(resource, action)}
                            aria-label={`${resource} ${action}`}
                          />
                          <span className="checkbox-custom" />
                        </label>
                      )}
                    </td>
                  );
                })}
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {!readOnly && (
        <div className="matrix-legend">
          <p>
            <strong>Legend:</strong> Check boxes to grant permissions. Unchecked permissions are denied.
          </p>
        </div>
      )}
    </div>
  );
};

export default PermissionMatrix;
