/**
 * Presence Indicator Component
 * Displays active users in the collaboration session
 */

import React from 'react';
import { Participant, ParticipantStatus } from '../types';

export interface PresenceIndicatorProps {
  participants: Map<string, Participant>;
  maxVisible?: number;
  size?: 'small' | 'medium' | 'large';
  showStatus?: boolean;
  className?: string;
}

export const PresenceIndicator: React.FC<PresenceIndicatorProps> = ({
  participants,
  maxVisible = 5,
  size = 'medium',
  showStatus = true,
  className = '',
}) => {
  const participantArray = Array.from(participants.values());
  const visibleParticipants = participantArray.slice(0, maxVisible);
  const remainingCount = Math.max(0, participantArray.length - maxVisible);

  const sizeClasses = {
    small: 'w-6 h-6 text-xs',
    medium: 'w-8 h-8 text-sm',
    large: 'w-10 h-10 text-base',
  };

  const getStatusColor = (status: ParticipantStatus): string => {
    switch (status) {
      case ParticipantStatus.ACTIVE:
        return 'bg-green-500';
      case ParticipantStatus.IDLE:
        return 'bg-yellow-500';
      case ParticipantStatus.AWAY:
        return 'bg-orange-500';
      case ParticipantStatus.OFFLINE:
        return 'bg-gray-500';
      default:
        return 'bg-gray-500';
    }
  };

  return (
    <div className={`flex items-center space-x-1 ${className}`}>
      {visibleParticipants.map((participant) => (
        <div
          key={participant.id}
          className="relative"
          title={`${participant.displayName} (${participant.status})`}
        >
          <div
            className={`${sizeClasses[size]} rounded-full flex items-center justify-center font-medium text-white`}
            style={{ backgroundColor: participant.color }}
          >
            {participant.avatarUrl ? (
              <img
                src={participant.avatarUrl}
                alt={participant.displayName}
                className="w-full h-full rounded-full object-cover"
              />
            ) : (
              <span>{participant.displayName.charAt(0).toUpperCase()}</span>
            )}
          </div>

          {showStatus && (
            <div
              className={`absolute bottom-0 right-0 w-2 h-2 rounded-full border border-white ${getStatusColor(participant.status)}`}
            />
          )}
        </div>
      ))}

      {remainingCount > 0 && (
        <div
          className={`${sizeClasses[size]} rounded-full bg-gray-300 flex items-center justify-center font-medium text-gray-700`}
        >
          +{remainingCount}
        </div>
      )}
    </div>
  );
};
