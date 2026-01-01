/**
 * NotificationList - Simple notification list component
 */

import React from 'react';
import { Notification, NotificationStatus } from '../types';

export interface NotificationListProps {
  notifications: Notification[];
  onNotificationClick?: (notification: Notification) => void;
  onMarkAsRead?: (notificationId: string) => void;
  onDelete?: (notificationId: string) => void;
  renderItem?: (notification: Notification) => React.ReactNode;
  emptyMessage?: string;
  className?: string;
}

export const NotificationList: React.FC<NotificationListProps> = ({
  notifications,
  onNotificationClick,
  onMarkAsRead,
  onDelete,
  renderItem,
  emptyMessage = 'No notifications',
  className = '',
}) => {
  if (notifications.length === 0) {
    return (
      <div className={`notification-list empty ${className}`}>
        <p className="empty-message">{emptyMessage}</p>
      </div>
    );
  }

  return (
    <div className={`notification-list ${className}`}>
      {notifications.map(notification =>
        renderItem ? (
          <div key={notification.id}>{renderItem(notification)}</div>
        ) : (
          <DefaultNotificationItem
            key={notification.id}
            notification={notification}
            onClick={onNotificationClick}
            onMarkAsRead={onMarkAsRead}
            onDelete={onDelete}
          />
        )
      )}
    </div>
  );
};

interface DefaultNotificationItemProps {
  notification: Notification;
  onClick?: (notification: Notification) => void;
  onMarkAsRead?: (notificationId: string) => void;
  onDelete?: (notificationId: string) => void;
}

const DefaultNotificationItem: React.FC<DefaultNotificationItemProps> = ({
  notification,
  onClick,
  onMarkAsRead,
  onDelete,
}) => {
  const isUnread = notification.status !== NotificationStatus.READ;

  return (
    <div
      className={`notification-item ${isUnread ? 'unread' : ''}`}
      onClick={() => onClick?.(notification)}
    >
      <div className="item-content">
        <h4>{notification.title}</h4>
        <p>{notification.message}</p>
        <span className="timestamp">{new Date(notification.createdAt).toLocaleString()}</span>
      </div>

      <div className="item-actions">
        {isUnread && onMarkAsRead && (
          <button onClick={(e) => { e.stopPropagation(); onMarkAsRead(notification.id); }}>
            Mark Read
          </button>
        )}
        {onDelete && (
          <button onClick={(e) => { e.stopPropagation(); onDelete(notification.id); }}>
            Delete
          </button>
        )}
      </div>
    </div>
  );
};

export default NotificationList;
