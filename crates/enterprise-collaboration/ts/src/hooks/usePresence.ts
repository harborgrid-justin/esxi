/**
 * usePresence Hook
 * Hook for managing user presence state
 */

import { useEffect, useCallback } from 'react';
import { useCollaborationContext } from '../components/CollaborationProvider';
import type { PresenceState, Participant } from '../types';

export interface UsePresenceOptions {
  updateInterval?: number;
  idleTimeout?: number;
}

export interface UsePresenceResult {
  presenceStates: Map<string, PresenceState>;
  activeParticipants: Participant[];
  updatePresence: (state: Partial<PresenceState>) => void;
  setTyping: (isTyping: boolean) => void;
  setViewport: (viewport: PresenceState['viewport']) => void;
}

/**
 * Hook for managing user presence
 */
export const usePresence = (options: UsePresenceOptions = {}): UsePresenceResult => {
  const { updateInterval = 5000, idleTimeout = 30000 } = options;
  const context = useCollaborationContext();

  // Auto-update presence
  useEffect(() => {
    if (!context.isConnected) return;

    const interval = setInterval(() => {
      context.updatePresence({
        lastActivity: new Date(),
      });
    }, updateInterval);

    return () => clearInterval(interval);
  }, [context, updateInterval]);

  // Track idle state
  useEffect(() => {
    if (!context.isConnected) return;

    let idleTimer: NodeJS.Timeout;

    const resetIdleTimer = () => {
      clearTimeout(idleTimer);
      idleTimer = setTimeout(() => {
        context.updatePresence({
          isTyping: false,
        });
      }, idleTimeout);
    };

    const handleActivity = () => {
      context.updatePresence({
        lastActivity: new Date(),
      });
      resetIdleTimer();
    };

    window.addEventListener('mousemove', handleActivity);
    window.addEventListener('keydown', handleActivity);
    window.addEventListener('click', handleActivity);

    resetIdleTimer();

    return () => {
      window.removeEventListener('mousemove', handleActivity);
      window.removeEventListener('keydown', handleActivity);
      window.removeEventListener('click', handleActivity);
      clearTimeout(idleTimer);
    };
  }, [context, idleTimeout]);

  const setTyping = useCallback(
    (isTyping: boolean) => {
      context.updatePresence({ isTyping });
    },
    [context]
  );

  const setViewport = useCallback(
    (viewport: PresenceState['viewport']) => {
      context.updatePresence({ viewport });
    },
    [context]
  );

  const activeParticipants = Array.from(context.participants.values()).filter(
    (participant) => {
      const presence = context.presenceStates.get(participant.id);
      if (!presence) return false;

      const timeSinceActivity =
        Date.now() - new Date(presence.lastActivity).getTime();
      return timeSinceActivity < idleTimeout;
    }
  );

  return {
    presenceStates: context.presenceStates,
    activeParticipants,
    updatePresence: context.updatePresence,
    setTyping,
    setViewport,
  };
};
