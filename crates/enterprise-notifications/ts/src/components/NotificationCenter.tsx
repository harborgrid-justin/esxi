/**
 * NotificationCenter - Main notification UI component
 * Displays notifications with filtering, grouping, and actions
 */

import React, { useState, useEffect, useCallback } from 'react';
import { Notification, NotificationStatus, NotificationPriority } from '../types';

export interface NotificationCenterProps {
  userId: string;
  tenantId: string;
  notifications: Notification[];
  onMarkAsRead?: (notificationId: string) => void;
  onMarkAllAsRead?: () => void;
  onDelete?: (notificationId: string) => void;
  onAction?: (notification: Notification, action: string) => void;
  onLoadMore?: () => void;
  className?: string;
}

export const NotificationCenter: React.FC<NotificationCenterProps> = ({
  userId,
  tenantId,
  notifications,
  onMarkAsRead,
  onMarkAllAsRead,
  onDelete,
  onAction,
  onLoadMore,
  className = '',
}) => {
  const [filter, setFilter] = useState<'all' | 'unread'>('all');
  const [priorityFilter, setPriorityFilter] = useState<NotificationPriority | 'all'>('all');
  const [groupBy, setGroupBy] = useState<'none' | 'date' | 'category'>('date');

  const filteredNotifications = notifications.filter(n => {
    if (filter === 'unread' && n.status === NotificationStatus.READ) {
      return false;
    }
    if (priorityFilter !== 'all' && n.priority !== priorityFilter) {
      return false;
    }
    return true;
  });

  const groupedNotifications = groupNotifications(filteredNotifications, groupBy);

  const unreadCount = notifications.filter(n => n.status !== NotificationStatus.READ).length;

  return (
    <div className={`notification-center ${className}`}>
      {/* Header */}
      <div className="notification-center-header">
        <h2>Notifications</h2>
        {unreadCount > 0 && (
          <button onClick={onMarkAllAsRead} className="mark-all-read-btn">
            Mark all as read
          </button>
        )}
      </div>

      {/* Filters */}
      <div className="notification-filters">
        <div className="filter-group">
          <button
            className={filter === 'all' ? 'active' : ''}
            onClick={() => setFilter('all')}
          >
            All ({notifications.length})
          </button>
          <button
            className={filter === 'unread' ? 'active' : ''}
            onClick={() => setFilter('unread')}
          >
            Unread ({unreadCount})
          </button>
        </div>

        <select
          value={priorityFilter}
          onChange={e => setPriorityFilter(e.target.value as NotificationPriority | 'all')}
          className="priority-filter"
        >
          <option value="all">All Priorities</option>
          <option value="critical">Critical</option>
          <option value="urgent">Urgent</option>
          <option value="high">High</option>
          <option value="normal">Normal</option>
          <option value="low">Low</option>
        </select>

        <select
          value={groupBy}
          onChange={e => setGroupBy(e.target.value as 'none' | 'date' | 'category')}
          className="group-by-filter"
        >
          <option value="none">No Grouping</option>
          <option value="date">Group by Date</option>
          <option value="category">Group by Category</option>
        </select>
      </div>

      {/* Notification List */}
      <div className="notification-list">
        {filteredNotifications.length === 0 ? (
          <div className="empty-state">
            <p>No notifications to display</p>
          </div>
        ) : (
          Object.entries(groupedNotifications).map(([group, items]) => (
            <div key={group} className="notification-group">
              {groupBy !== 'none' && <h3 className="group-header">{group}</h3>}
              {items.map(notification => (
                <NotificationItem
                  key={notification.id}
                  notification={notification}
                  onMarkAsRead={onMarkAsRead}
                  onDelete={onDelete}
                  onAction={onAction}
                />
              ))}
            </div>
          ))
        )}
      </div>

      {/* Load More */}
      {onLoadMore && (
        <div className="notification-center-footer">
          <button onClick={onLoadMore} className="load-more-btn">
            Load More
          </button>
        </div>
      )}
    </div>
  );
};

interface NotificationItemProps {
  notification: Notification;
  onMarkAsRead?: (notificationId: string) => void;
  onDelete?: (notificationId: string) => void;
  onAction?: (notification: Notification, action: string) => void;
}

const NotificationItem: React.FC<NotificationItemProps> = ({
  notification,
  onMarkAsRead,
  onDelete,
  onAction,
}) => {
  const isUnread = notification.status !== NotificationStatus.READ;

  const handleClick = () => {
    if (isUnread && onMarkAsRead) {
      onMarkAsRead(notification.id);
    }
  };

  const priorityColor = {
    critical: '#dc2626',
    urgent: '#ea580c',
    high: '#f59e0b',
    normal: '#3b82f6',
    low: '#6b7280',
  }[notification.priority];

  return (
    <div
      className={`notification-item ${isUnread ? 'unread' : 'read'} priority-${notification.priority}`}
      onClick={handleClick}
    >
      <div className="notification-indicator" style={{ backgroundColor: priorityColor }} />

      <div className="notification-content">
        <div className="notification-header">
          <h4 className="notification-title">{notification.title}</h4>
          <span className="notification-time">
            {formatRelativeTime(notification.createdAt)}
          </span>
        </div>

        <p className="notification-message">{notification.message}</p>

        {notification.data && Object.keys(notification.data).length > 0 && (
          <div className="notification-metadata">
            {Object.entries(notification.data).slice(0, 3).map(([key, value]) => (
              <span key={key} className="metadata-item">
                <strong>{key}:</strong> {String(value)}
              </span>
            ))}
          </div>
        )}

        {notification.actionUrl && (
          <div className="notification-actions">
            <button
              onClick={e => {
                e.stopPropagation();
                window.open(notification.actionUrl, '_blank');
              }}
              className="action-btn primary"
            >
              {notification.actionLabel ?? 'View'}
            </button>
          </div>
        )}

        {notification.links && notification.links.length > 0 && (
          <div className="notification-links">
            {notification.links.map((link, index) => (
              <button
                key={index}
                onClick={e => {
                  e.stopPropagation();
                  onAction?.(notification, link.action ?? link.url);
                }}
                className="action-btn secondary"
              >
                {link.label}
              </button>
            ))}
          </div>
        )}
      </div>

      <div className="notification-menu">
        {isUnread && onMarkAsRead && (
          <button
            onClick={e => {
              e.stopPropagation();
              onMarkAsRead(notification.id);
            }}
            className="menu-btn"
            title="Mark as read"
          >
            ✓
          </button>
        )}
        {onDelete && (
          <button
            onClick={e => {
              e.stopPropagation();
              onDelete(notification.id);
            }}
            className="menu-btn"
            title="Delete"
          >
            ×
          </button>
        )}
      </div>
    </div>
  );
};

function groupNotifications(
  notifications: Notification[],
  groupBy: 'none' | 'date' | 'category'
): Record<string, Notification[]> {
  if (groupBy === 'none') {
    return { all: notifications };
  }

  const groups: Record<string, Notification[]> = {};

  for (const notification of notifications) {
    let key: string;

    if (groupBy === 'date') {
      key = formatDate(notification.createdAt);
    } else {
      key = notification.category ?? 'Uncategorized';
    }

    if (!groups[key]) {
      groups[key] = [];
    }
    groups[key]!.push(notification);
  }

  return groups;
}

function formatDate(date: Date): string {
  const today = new Date();
  const yesterday = new Date(today);
  yesterday.setDate(yesterday.getDate() - 1);

  if (date.toDateString() === today.toDateString()) {
    return 'Today';
  } else if (date.toDateString() === yesterday.toDateString()) {
    return 'Yesterday';
  } else {
    return date.toLocaleDateString();
  }
}

function formatRelativeTime(date: Date): string {
  const now = new Date();
  const diff = now.getTime() - date.getTime();

  const seconds = Math.floor(diff / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);

  if (seconds < 60) {
    return 'Just now';
  } else if (minutes < 60) {
    return `${minutes}m ago`;
  } else if (hours < 24) {
    return `${hours}h ago`;
  } else {
    return `${days}d ago`;
  }
}

export default NotificationCenter;
