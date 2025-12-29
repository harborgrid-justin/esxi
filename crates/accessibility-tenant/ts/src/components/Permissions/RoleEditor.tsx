/**
 * Role Editor Component
 * Create and edit custom roles with permissions
 */

import React, { useState } from 'react';
import { useForm } from 'react-hook-form';
import { Role, Permission, ResourceType, PermissionAction } from '../../types';
import { PermissionMatrix } from './PermissionMatrix';
import { createPermission } from '../../utils/permissions';

interface RoleEditorProps {
  role?: Role;
  onSave: (roleData: Partial<Role>) => Promise<void>;
  onCancel?: () => void;
  className?: string;
}

interface RoleFormData {
  name: string;
  description: string;
  isDefault: boolean;
}

export const RoleEditor: React.FC<RoleEditorProps> = ({
  role,
  onSave,
  onCancel,
  className,
}) => {
  const [permissions, setPermissions] = useState<Permission[]>(role?.permissions || []);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string>('');
  const [success, setSuccess] = useState(false);

  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<RoleFormData>({
    defaultValues: {
      name: role?.name || '',
      description: role?.description || '',
      isDefault: role?.isDefault || false,
    },
  });

  const handleTogglePermission = (resource: ResourceType, action: PermissionAction) => {
    setPermissions((current) => {
      const exists = current.some((p) => p.resource === resource && p.action === action);

      if (exists) {
        return current.filter((p) => !(p.resource === resource && p.action === action));
      } else {
        return [...current, createPermission(resource, action)];
      }
    });
  };

  const onSubmit = async (data: RoleFormData) => {
    setIsSubmitting(true);
    setError('');
    setSuccess(false);

    try {
      const roleData: Partial<Role> = {
        name: data.name,
        description: data.description || undefined,
        permissions,
        isDefault: data.isDefault,
      };

      await onSave(roleData);
      setSuccess(true);
      setTimeout(() => {
        setSuccess(false);
      }, 2000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save role');
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className={className}>
      <header>
        <h2>{role ? 'Edit Role' : 'Create Role'}</h2>
        <p className="subtitle">Define role details and permissions</p>
      </header>

      <form onSubmit={handleSubmit(onSubmit)} className="role-editor-form">
        <section className="basic-info">
          <h3>Basic Information</h3>

          <div className="form-group">
            <label htmlFor="role-name">
              Role Name
              <span className="required" aria-label="required">*</span>
            </label>
            <input
              id="role-name"
              type="text"
              {...register('name', {
                required: 'Role name is required',
                minLength: { value: 2, message: 'Name must be at least 2 characters' },
                maxLength: { value: 50, message: 'Name must be less than 50 characters' },
              })}
              className="form-input"
              aria-invalid={errors.name ? 'true' : 'false'}
            />
            {errors.name && (
              <p className="error-message" role="alert">
                {errors.name.message}
              </p>
            )}
          </div>

          <div className="form-group">
            <label htmlFor="role-description">Description</label>
            <textarea
              id="role-description"
              {...register('description', {
                maxLength: {
                  value: 200,
                  message: 'Description must be less than 200 characters',
                },
              })}
              className="form-textarea"
              rows={3}
              aria-invalid={errors.description ? 'true' : 'false'}
            />
            {errors.description && (
              <p className="error-message" role="alert">
                {errors.description.message}
              </p>
            )}
          </div>

          <div className="form-group checkbox-group">
            <label>
              <input type="checkbox" {...register('isDefault')} />
              <span>Set as Default Role</span>
            </label>
            <p className="help-text">
              Default roles are automatically assigned to new users
            </p>
          </div>
        </section>

        <section className="permissions-section">
          <PermissionMatrix
            permissions={permissions}
            onTogglePermission={handleTogglePermission}
            readOnly={false}
          />

          <div className="permission-count">
            <p>
              <strong>{permissions.length}</strong> permission{permissions.length !== 1 ? 's' : ''} granted
            </p>
          </div>
        </section>

        <div className="form-actions">
          <button
            type="submit"
            disabled={isSubmitting}
            className="btn btn-primary"
          >
            {isSubmitting ? 'Saving...' : 'Save Role'}
          </button>
          {onCancel && (
            <button
              type="button"
              onClick={onCancel}
              disabled={isSubmitting}
              className="btn btn-secondary"
            >
              Cancel
            </button>
          )}
        </div>

        {success && (
          <div className="success-message" role="status" aria-live="polite">
            Role saved successfully!
          </div>
        )}
        {error && (
          <div className="error-message" role="alert" aria-live="assertive">
            {error}
          </div>
        )}
      </form>
    </div>
  );
};

export default RoleEditor;
