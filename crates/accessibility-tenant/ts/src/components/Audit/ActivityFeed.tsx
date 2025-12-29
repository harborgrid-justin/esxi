/**
 * Activity Feed Component
 * Real-time activity feed for organization
 */

import React from 'react';
import { Activity } from '../../types';

interface ActivityFeedProps {
  activities: Activity[];
  onRefresh?: () => void;
  maxItems?: number;
  className?: string;
}

export const ActivityFeed: React.FC<ActivityFeedProps> = ({
  activities,
  onRefresh,
  maxItems = 50,
  className,
}) => {
  const displayActivities = activities.slice(0, maxItems);

  const getTimeAgo = (timestamp: Date): string => {
    const seconds = Math.floor((new Date().getTime() - new Date(timestamp).getTime()) / 1000);

    if (seconds < 60) return 'just now';
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
    if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
    if (seconds < 604800) return `${Math.floor(seconds / 86400)}d ago`;
    return new Date(timestamp).toLocaleDateString();
  };

  return (
    <div className={className}>
      <header className="feed-header">
        <div>
          <h2>Activity Feed</h2>
          <p className="subtitle">Recent activity in your organization</p>
        </div>
        {onRefresh && (
          <button
            type="button"
            onClick={onRefresh}
            className="btn btn-secondary"
            aria-label="Refresh activity feed"
          >
            Refresh
          </button>
        )}
      </header>

      <div className="activity-feed" role="feed" aria-label="Activity feed">
        {displayActivities.map((activity) => (
          <article
            key={activity.id}
            className="activity-item"
            role="article"
            aria-label={`${activity.userName} ${activity.description}`}
          >
            <div className="activity-avatar">
              {activity.userAvatar ? (
                <img src={activity.userAvatar} alt="" />
              ) : (
                <div className="avatar-placeholder">
                  {activity.userName.charAt(0).toUpperCase()}
                </div>
              )}
            </div>
            <div className="activity-content">
              <div className="activity-header">
                <strong className="user-name">{activity.userName}</strong>
                <span className="activity-action">{activity.action}</span>
                {activity.resourceName && (
                  <span className="resource-name">{activity.resourceName}</span>
                )}
              </div>
              <p className="activity-description">{activity.description}</p>
              <time className="activity-time" dateTime={new Date(activity.timestamp).toISOString()}>
                {getTimeAgo(activity.timestamp)}
              </time>
            </div>
          </article>
        ))}

        {displayActivities.length === 0 && (
          <div className="empty-state">
            <p>No recent activity</p>
          </div>
        )}
      </div>
    </div>
  );
};

export default ActivityFeed;
