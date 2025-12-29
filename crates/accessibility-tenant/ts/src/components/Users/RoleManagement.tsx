/**
 * Role Management Component
 * Manage custom roles and permissions
 */

import React, { useState } from 'react';
import { Role, Permission } from '../../types';
import { usePermissions } from '../../hooks/usePermissions';

interface RoleManagementProps {
  roles: Role[];
  onCreateRole?: () => void;
  onEditRole?: (role: Role) => void;
  onDeleteRole?: (roleId: string) => void;
  className?: string;
}

export const RoleManagement: React.FC<RoleManagementProps> = ({
  roles,
  onCreateRole,
  onEditRole,
  onDeleteRole,
  className,
}) => {
  const [search, setSearch] = useState('');
  const { can } = usePermissions();

  const canManage = can.manageOrganization();

  const filteredRoles = roles.filter(
    (role) =>
      !search ||
      role.name.toLowerCase().includes(search.toLowerCase()) ||
      role.description?.toLowerCase().includes(search.toLowerCase())
  );

  return (
    <div className={className}>
      <header className="management-header">
        <div>
          <h1>Role Management</h1>
          <p className="subtitle">{filteredRoles.length} role{filteredRoles.length !== 1 ? 's' : ''}</p>
        </div>
        {canManage && onCreateRole && (
          <button type="button" onClick={onCreateRole} className="btn btn-primary">
            Create Role
          </button>
        )}
      </header>

      <div className="filters">
        <input
          type="search"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder="Search roles..."
          className="filter-input"
          aria-label="Search roles"
        />
      </div>

      <div className="role-grid">
        {filteredRoles.map((role) => (
          <article key={role.id} className="role-card">
            <header className="card-header">
              <h3>{role.name}</h3>
              {role.isSystemRole && <span className="system-badge">System</span>}
              {role.isDefault && <span className="default-badge">Default</span>}
            </header>
            {role.description && <p className="role-description">{role.description}</p>}
            <div className="role-permissions">
              <strong>Permissions:</strong> {role.permissions.length}
            </div>
            {canManage && !role.isSystemRole && (
              <div className="card-actions">
                {onEditRole && (
                  <button
                    type="button"
                    onClick={() => onEditRole(role)}
                    className="btn btn-secondary"
                  >
                    Edit
                  </button>
                )}
                {onDeleteRole && (
                  <button
                    type="button"
                    onClick={() => onDeleteRole(role.id)}
                    className="btn btn-danger"
                  >
                    Delete
                  </button>
                )}
              </div>
            )}
          </article>
        ))}
      </div>

      {filteredRoles.length === 0 && (
        <div className="empty-state">
          <p>No roles found</p>
        </div>
      )}
    </div>
  );
};

export default RoleManagement;
