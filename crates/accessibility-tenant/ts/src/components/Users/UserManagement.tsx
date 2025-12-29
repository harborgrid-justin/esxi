/**
 * User Management Component
 * List and manage users with CRUD operations
 */

import React, { useState } from 'react';
import { usePermissions } from '../../hooks/usePermissions';
import { User, UserRole, UserStatus } from '../../types';
import UserService from '../../services/UserService';

interface UserManagementProps {
  userService: UserService;
  users: User[];
  onRefresh: () => void;
  onEditUser?: (user: User) => void;
  onDeleteUser?: (userId: string) => void;
  onInviteUser?: () => void;
  className?: string;
}

export const UserManagement: React.FC<UserManagementProps> = ({
  users,
  onRefresh,
  onEditUser,
  onDeleteUser,
  onInviteUser,
  className,
}) => {
  const [search, setSearch] = useState('');
  const [roleFilter, setRoleFilter] = useState<string>('');
  const [statusFilter, setStatusFilter] = useState<string>('');
  const { can } = usePermissions();

  const canManageUsers = can.manageUsers();

  const filteredUsers = users.filter((user) => {
    const matchesSearch =
      !search ||
      user.email.toLowerCase().includes(search.toLowerCase()) ||
      user.displayName.toLowerCase().includes(search.toLowerCase());
    const matchesRole = !roleFilter || user.role === roleFilter;
    const matchesStatus = !statusFilter || user.status === statusFilter;
    return matchesSearch && matchesRole && matchesStatus;
  });

  const getRoleDisplayName = (role: UserRole): string => {
    return role.replace(/_/g, ' ');
  };

  const getStatusBadgeClass = (status: UserStatus): string => {
    switch (status) {
      case UserStatus.ACTIVE:
        return 'status-active';
      case UserStatus.INVITED:
        return 'status-invited';
      case UserStatus.SUSPENDED:
        return 'status-suspended';
      case UserStatus.INACTIVE:
        return 'status-inactive';
      default:
        return '';
    }
  };

  return (
    <div className={className}>
      <header className="management-header">
        <div>
          <h1>User Management</h1>
          <p className="subtitle">{filteredUsers.length} user{filteredUsers.length !== 1 ? 's' : ''}</p>
        </div>
        {canManageUsers && onInviteUser && (
          <button type="button" onClick={onInviteUser} className="btn btn-primary">
            Invite User
          </button>
        )}
      </header>

      <div className="filters">
        <input
          type="search"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder="Search users..."
          className="filter-input"
          aria-label="Search users"
        />
        <select
          value={roleFilter}
          onChange={(e) => setRoleFilter(e.target.value)}
          className="filter-select"
          aria-label="Filter by role"
        >
          <option value="">All Roles</option>
          {Object.values(UserRole).map((role) => (
            <option key={role} value={role}>
              {getRoleDisplayName(role)}
            </option>
          ))}
        </select>
        <select
          value={statusFilter}
          onChange={(e) => setStatusFilter(e.target.value)}
          className="filter-select"
          aria-label="Filter by status"
        >
          <option value="">All Statuses</option>
          {Object.values(UserStatus).map((status) => (
            <option key={status} value={status}>
              {status}
            </option>
          ))}
        </select>
        <button type="button" onClick={onRefresh} className="btn btn-secondary">
          Refresh
        </button>
      </div>

      <div className="user-table-container">
        <table className="user-table">
          <thead>
            <tr>
              <th>User</th>
              <th>Email</th>
              <th>Role</th>
              <th>Status</th>
              <th>Last Login</th>
              {canManageUsers && <th>Actions</th>}
            </tr>
          </thead>
          <tbody>
            {filteredUsers.map((user) => (
              <tr key={user.id}>
                <td>
                  <div className="user-cell">
                    {user.avatarUrl && (
                      <img src={user.avatarUrl} alt="" className="user-avatar" />
                    )}
                    <span>{user.displayName}</span>
                  </div>
                </td>
                <td>{user.email}</td>
                <td>
                  <span className="role-badge">{getRoleDisplayName(user.role)}</span>
                </td>
                <td>
                  <span className={`status-badge ${getStatusBadgeClass(user.status)}`}>
                    {user.status}
                  </span>
                </td>
                <td>
                  {user.lastLoginAt
                    ? new Date(user.lastLoginAt).toLocaleDateString()
                    : 'Never'}
                </td>
                {canManageUsers && (
                  <td>
                    <div className="action-buttons">
                      {onEditUser && (
                        <button
                          type="button"
                          onClick={() => onEditUser(user)}
                          className="btn btn-sm btn-secondary"
                          aria-label={`Edit ${user.displayName}`}
                        >
                          Edit
                        </button>
                      )}
                      {onDeleteUser && (
                        <button
                          type="button"
                          onClick={() => onDeleteUser(user.id)}
                          className="btn btn-sm btn-danger"
                          aria-label={`Delete ${user.displayName}`}
                        >
                          Delete
                        </button>
                      )}
                    </div>
                  </td>
                )}
              </tr>
            ))}
          </tbody>
        </table>
        {filteredUsers.length === 0 && (
          <div className="empty-state">
            <p>No users found</p>
          </div>
        )}
      </div>
    </div>
  );
};

export default UserManagement;
