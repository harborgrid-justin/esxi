/**
 * NotificationBell - Bell icon with badge showing unread count
 */

import React, { useState, useRef, useEffect } from 'react';
import { Notification } from '../types';

export interface NotificationBellProps {
  notifications: Notification[];
  unreadCount: number;
  onOpen?: () => void;
  onClose?: () => void;
  renderDropdown?: (notifications: Notification[]) => React.ReactNode;
  maxDropdownItems?: number;
  className?: string;
}

export const NotificationBell: React.FC<NotificationBellProps> = ({
  notifications,
  unreadCount,
  onOpen,
  onClose,
  renderDropdown,
  maxDropdownItems = 5,
  className = '',
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        handleClose();
      }
    };

    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside);
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [isOpen]);

  const handleOpen = () => {
    setIsOpen(true);
    onOpen?.();
  };

  const handleClose = () => {
    setIsOpen(false);
    onClose?.();
  };

  const handleToggle = () => {
    if (isOpen) {
      handleClose();
    } else {
      handleOpen();
    }
  };

  return (
    <div className={`notification-bell ${className}`} ref={dropdownRef}>
      <button
        className={`bell-button ${isOpen ? 'open' : ''} ${unreadCount > 0 ? 'has-unread' : ''}`}
        onClick={handleToggle}
        aria-label={`Notifications (${unreadCount} unread)`}
      >
        <BellIcon />
        {unreadCount > 0 && (
          <span className="notification-badge">
            {unreadCount > 99 ? '99+' : unreadCount}
          </span>
        )}
      </button>

      {isOpen && (
        <div className="notification-dropdown">
          <div className="dropdown-header">
            <h3>Notifications</h3>
            {unreadCount > 0 && <span className="unread-count">{unreadCount} unread</span>}
          </div>

          <div className="dropdown-content">
            {notifications.length === 0 ? (
              <div className="empty-notifications">
                <p>No new notifications</p>
              </div>
            ) : (
              <>
                {renderDropdown ? (
                  renderDropdown(notifications.slice(0, maxDropdownItems))
                ) : (
                  <DefaultDropdownContent notifications={notifications.slice(0, maxDropdownItems)} />
                )}
              </>
            )}
          </div>

          {notifications.length > maxDropdownItems && (
            <div className="dropdown-footer">
              <a href="/notifications" className="view-all-link">
                View all notifications
              </a>
            </div>
          )}
        </div>
      )}
    </div>
  );
};

const BellIcon: React.FC = () => (
  <svg
    width="24"
    height="24"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9" />
    <path d="M13.73 21a2 2 0 0 1-3.46 0" />
  </svg>
);

const DefaultDropdownContent: React.FC<{ notifications: Notification[] }> = ({ notifications }) => (
  <div className="default-dropdown-content">
    {notifications.map(notification => (
      <div key={notification.id} className="dropdown-notification-item">
        <div className="notification-dot" data-priority={notification.priority} />
        <div className="notification-info">
          <p className="notification-title">{notification.title}</p>
          <p className="notification-message">{notification.message}</p>
          <span className="notification-time">{formatTime(notification.createdAt)}</span>
        </div>
      </div>
    ))}
  </div>
);

function formatTime(date: Date): string {
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const minutes = Math.floor(diff / 60000);

  if (minutes < 1) return 'Just now';
  if (minutes < 60) return `${minutes}m ago`;
  if (minutes < 1440) return `${Math.floor(minutes / 60)}h ago`;
  return date.toLocaleDateString();
}

export default NotificationBell;
