/**
 * Role Manager - RBAC Role Management UI
 * Manage roles, permissions, and user assignments
 */

import React, { useState, useEffect } from 'react';
import { rbacEngine } from '../authz/RBACEngine';
import { Role, Permission } from '../types';

export const RoleManager: React.FC = () => {
  const [roles, setRoles] = useState<Role[]>([]);
  const [selectedRole, setSelectedRole] = useState<Role | null>(null);

  useEffect(() => {
    setRoles(rbacEngine.getAllRoles());
  }, []);

  const handleCreateRole = async () => {
    const newRole = await rbacEngine.createRole(
      'New Role',
      'Description',
      [Permission.READ]
    );
    setRoles([...roles, newRole]);
  };

  return (
    <div className="role-manager">
      <div className="manager-header">
        <h2>Role Management</h2>
        <button onClick={handleCreateRole}>Create Role</button>
      </div>

      <div className="roles-grid">
        {roles.map((role) => (
          <div
            key={role.id}
            className={`role-card ${selectedRole?.id === role.id ? 'selected' : ''}`}
            onClick={() => setSelectedRole(role)}
          >
            <h3>{role.name}</h3>
            <p>{role.description}</p>
            <div className="permissions-count">
              {role.permissions.length} permissions
            </div>
          </div>
        ))}
      </div>

      {selectedRole && (
        <div className="role-details">
          <h3>{selectedRole.name}</h3>
          <div className="permissions-list">
            {selectedRole.permissions.map((perm) => (
              <span key={perm} className="permission-tag">{perm}</span>
            ))}
          </div>
        </div>
      )}

      <style>{`
        .role-manager {
          background: #fff;
          padding: 24px;
          border-radius: 8px;
        }
        .manager-header {
          display: flex;
          justify-content: space-between;
          margin-bottom: 24px;
        }
        .roles-grid {
          display: grid;
          grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
          gap: 16px;
          margin-bottom: 24px;
        }
        .role-card {
          padding: 16px;
          border: 2px solid #e0e0e0;
          border-radius: 8px;
          cursor: pointer;
          transition: all 0.2s;
        }
        .role-card:hover {
          border-color: #2196f3;
        }
        .role-card.selected {
          border-color: #2196f3;
          background: #e3f2fd;
        }
        .permission-tag {
          display: inline-block;
          padding: 4px 12px;
          margin: 4px;
          background: #e3f2fd;
          border-radius: 16px;
          font-size: 12px;
        }
      `}</style>
    </div>
  );
};
