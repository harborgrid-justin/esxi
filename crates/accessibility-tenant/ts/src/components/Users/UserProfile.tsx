/**
 * User Profile Component
 * View and edit user profile information
 */

import React, { useState, useEffect } from 'react';
import { useForm } from 'react-hook-form';
import { User } from '../../types';
import UserService from '../../services/UserService';

interface UserProfileProps {
  userService: UserService;
  user: User;
  onUpdate?: (user: User) => void;
  className?: string;
}

interface ProfileFormData {
  firstName: string;
  lastName: string;
  displayName: string;
}

export const UserProfile: React.FC<UserProfileProps> = ({
  userService,
  user,
  onUpdate,
  className,
}) => {
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string>('');
  const [success, setSuccess] = useState(false);
  const [avatarPreview, setAvatarPreview] = useState<string>(user.avatarUrl || '');
  const [selectedAvatar, setSelectedAvatar] = useState<File | null>(null);

  const {
    register,
    handleSubmit,
    reset,
    formState: { errors, isDirty },
  } = useForm<ProfileFormData>();

  useEffect(() => {
    reset({
      firstName: user.firstName,
      lastName: user.lastName,
      displayName: user.displayName,
    });
  }, [user, reset]);

  const handleAvatarChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      if (!file.type.startsWith('image/')) {
        setError('Please select an image file');
        return;
      }
      if (file.size > 2 * 1024 * 1024) {
        setError('Image must be less than 2MB');
        return;
      }
      setSelectedAvatar(file);
      const reader = new FileReader();
      reader.onloadend = () => {
        setAvatarPreview(reader.result as string);
      };
      reader.readAsDataURL(file);
    }
  };

  const onSubmit = async (data: ProfileFormData) => {
    setIsSubmitting(true);
    setError('');
    setSuccess(false);

    try {
      // Upload avatar if selected
      if (selectedAvatar) {
        await userService.uploadAvatar(user.id, selectedAvatar);
      }

      // Update profile
      const response = await userService.updateProfile(user.id, data);

      if (response.success && response.data) {
        setSuccess(true);
        setSelectedAvatar(null);
        onUpdate?.(response.data);
        setTimeout(() => setSuccess(false), 3000);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update profile');
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className={className}>
      <header>
        <h2>User Profile</h2>
        <p className="subtitle">Manage your profile information</p>
      </header>

      <form onSubmit={handleSubmit(onSubmit)} className="profile-form">
        <div className="avatar-section">
          <div className="avatar-preview">
            {avatarPreview ? (
              <img src={avatarPreview} alt="User avatar" />
            ) : (
              <div className="avatar-placeholder">{user.firstName[0]}{user.lastName[0]}</div>
            )}
          </div>
          <div className="avatar-upload">
            <label htmlFor="avatar-upload" className="btn btn-secondary">
              Change Avatar
            </label>
            <input
              id="avatar-upload"
              type="file"
              accept="image/*"
              onChange={handleAvatarChange}
              className="file-input"
              aria-label="Upload avatar"
            />
            <p className="help-text">PNG or JPG. Max 2MB.</p>
          </div>
        </div>

        <div className="form-group">
          <label htmlFor="firstName">
            First Name
            <span className="required" aria-label="required">*</span>
          </label>
          <input
            id="firstName"
            type="text"
            {...register('firstName', {
              required: 'First name is required',
              minLength: { value: 1, message: 'First name is required' },
            })}
            className="form-input"
            aria-invalid={errors.firstName ? 'true' : 'false'}
          />
          {errors.firstName && (
            <p className="error-message" role="alert">
              {errors.firstName.message}
            </p>
          )}
        </div>

        <div className="form-group">
          <label htmlFor="lastName">
            Last Name
            <span className="required" aria-label="required">*</span>
          </label>
          <input
            id="lastName"
            type="text"
            {...register('lastName', {
              required: 'Last name is required',
              minLength: { value: 1, message: 'Last name is required' },
            })}
            className="form-input"
            aria-invalid={errors.lastName ? 'true' : 'false'}
          />
          {errors.lastName && (
            <p className="error-message" role="alert">
              {errors.lastName.message}
            </p>
          )}
        </div>

        <div className="form-group">
          <label htmlFor="displayName">
            Display Name
            <span className="required" aria-label="required">*</span>
          </label>
          <input
            id="displayName"
            type="text"
            {...register('displayName', {
              required: 'Display name is required',
              minLength: { value: 1, message: 'Display name is required' },
            })}
            className="form-input"
            aria-invalid={errors.displayName ? 'true' : 'false'}
          />
          {errors.displayName && (
            <p className="error-message" role="alert">
              {errors.displayName.message}
            </p>
          )}
        </div>

        <div className="info-section">
          <h3>Account Information</h3>
          <div className="info-grid">
            <div className="info-item">
              <span className="label">Email:</span>
              <span className="value">{user.email}</span>
            </div>
            <div className="info-item">
              <span className="label">Role:</span>
              <span className="value">{user.role.replace(/_/g, ' ')}</span>
            </div>
            <div className="info-item">
              <span className="label">Status:</span>
              <span className="value">{user.status}</span>
            </div>
            {user.lastLoginAt && (
              <div className="info-item">
                <span className="label">Last Login:</span>
                <span className="value">
                  {new Date(user.lastLoginAt).toLocaleString()}
                </span>
              </div>
            )}
          </div>
        </div>

        <div className="form-actions">
          <button
            type="submit"
            disabled={(!isDirty && !selectedAvatar) || isSubmitting}
            className="btn btn-primary"
          >
            {isSubmitting ? 'Saving...' : 'Save Changes'}
          </button>
          <button
            type="button"
            onClick={() => {
              reset();
              setSelectedAvatar(null);
              setAvatarPreview(user.avatarUrl || '');
            }}
            disabled={(!isDirty && !selectedAvatar) || isSubmitting}
            className="btn btn-secondary"
          >
            Reset
          </button>
        </div>

        {success && (
          <div className="success-message" role="status" aria-live="polite">
            Profile updated successfully!
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

export default UserProfile;
