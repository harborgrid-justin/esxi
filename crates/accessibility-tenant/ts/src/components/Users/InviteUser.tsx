/**
 * Invite User Component
 * Form to invite new users to the organization
 */

import React, { useState } from 'react';
import { useForm } from 'react-hook-form';
import { InviteUser as InviteUserData, UserRole } from '../../types';
import UserService from '../../services/UserService';

interface InviteUserProps {
  userService: UserService;
  organizationId: string;
  onSuccess?: () => void;
  onCancel?: () => void;
  className?: string;
}

interface InviteFormData {
  email: string;
  role: UserRole;
  customMessage: string;
  expiresInDays: number;
}

export const InviteUser: React.FC<InviteUserProps> = ({
  userService,
  organizationId,
  onSuccess,
  onCancel,
  className,
}) => {
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string>('');
  const [success, setSuccess] = useState(false);

  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<InviteFormData>({
    defaultValues: {
      role: UserRole.ORG_USER,
      expiresInDays: 7,
    },
  });

  const onSubmit = async (data: InviteFormData) => {
    setIsSubmitting(true);
    setError('');
    setSuccess(false);

    try {
      const inviteData: InviteUserData = {
        email: data.email,
        role: data.role,
        organizationId,
        customMessage: data.customMessage || undefined,
        expiresInDays: data.expiresInDays,
      };

      await userService.inviteUser(inviteData);
      setSuccess(true);
      reset();
      setTimeout(() => {
        onSuccess?.();
      }, 2000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to send invitation');
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className={className}>
      <header>
        <h2>Invite User</h2>
        <p className="subtitle">Send an invitation to join the organization</p>
      </header>

      <form onSubmit={handleSubmit(onSubmit)} className="invite-form">
        <div className="form-group">
          <label htmlFor="email">
            Email Address
            <span className="required" aria-label="required">*</span>
          </label>
          <input
            id="email"
            type="email"
            {...register('email', {
              required: 'Email is required',
              pattern: {
                value: /^[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}$/i,
                message: 'Invalid email address',
              },
            })}
            className="form-input"
            aria-invalid={errors.email ? 'true' : 'false'}
          />
          {errors.email && (
            <p className="error-message" role="alert">
              {errors.email.message}
            </p>
          )}
        </div>

        <div className="form-group">
          <label htmlFor="role">
            Role
            <span className="required" aria-label="required">*</span>
          </label>
          <select
            id="role"
            {...register('role', { required: 'Role is required' })}
            className="form-select"
            aria-invalid={errors.role ? 'true' : 'false'}
          >
            <option value={UserRole.ORG_USER}>Organization User</option>
            <option value={UserRole.ORG_MANAGER}>Organization Manager</option>
            <option value={UserRole.ORG_ADMIN}>Organization Admin</option>
            <option value={UserRole.VIEWER}>Viewer</option>
          </select>
          {errors.role && (
            <p className="error-message" role="alert">
              {errors.role.message}
            </p>
          )}
        </div>

        <div className="form-group">
          <label htmlFor="expires">Invitation Expires In (days)</label>
          <input
            id="expires"
            type="number"
            {...register('expiresInDays', {
              min: { value: 1, message: 'Minimum 1 day' },
              max: { value: 30, message: 'Maximum 30 days' },
            })}
            className="form-input"
            aria-invalid={errors.expiresInDays ? 'true' : 'false'}
          />
          {errors.expiresInDays && (
            <p className="error-message" role="alert">
              {errors.expiresInDays.message}
            </p>
          )}
        </div>

        <div className="form-group">
          <label htmlFor="message">Custom Message (Optional)</label>
          <textarea
            id="message"
            {...register('customMessage')}
            className="form-textarea"
            rows={4}
            placeholder="Add a personal message to the invitation..."
          />
        </div>

        <div className="form-actions">
          <button
            type="submit"
            disabled={isSubmitting}
            className="btn btn-primary"
          >
            {isSubmitting ? 'Sending...' : 'Send Invitation'}
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
            Invitation sent successfully!
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

export default InviteUser;
