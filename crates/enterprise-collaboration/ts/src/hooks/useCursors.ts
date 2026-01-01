/**
 * useCursors Hook
 * Hook for managing multi-user cursors
 */

import { useState, useEffect, useCallback } from 'react';
import { useCollaborationContext } from '../components/CollaborationProvider';
import type { Cursor, Selection, Position, Participant } from '../types';

export interface UseCursorsOptions {
  throttleMs?: number;
  showOwnCursor?: boolean;
}

export interface UseCursorsResult {
  cursors: Map<string, Cursor>;
  selections: Map<string, Selection>;
  updateCursor: (position: Position) => void;
  updateSelection: (selection: Selection) => void;
  getCursorForParticipant: (participantId: string) => Cursor | undefined;
  getSelectionForParticipant: (participantId: string) => Selection | undefined;
  visibleCursors: Array<{ participant: Participant; cursor: Cursor }>;
}

/**
 * Hook for managing cursors
 */
export const useCursors = (options: UseCursorsOptions = {}): UseCursorsResult => {
  const { throttleMs = 100, showOwnCursor = false } = options;
  const context = useCollaborationContext();

  const [cursors, setCursors] = useState<Map<string, Cursor>>(new Map());
  const [selections, setSelections] = useState<Map<string, Selection>>(new Map());
  const [lastUpdate, setLastUpdate] = useState<number>(0);

  // Extract cursors and selections from presence states
  useEffect(() => {
    const newCursors = new Map<string, Cursor>();
    const newSelections = new Map<string, Selection>();

    context.presenceStates.forEach((presence, participantId) => {
      if (presence.cursor) {
        newCursors.set(participantId, presence.cursor);
      }
      if (presence.selection) {
        newSelections.set(participantId, presence.selection);
      }
    });

    setCursors(newCursors);
    setSelections(newSelections);
  }, [context.presenceStates]);

  /**
   * Update cursor position with throttling
   */
  const updateCursor = useCallback(
    (position: Position) => {
      const now = Date.now();
      if (now - lastUpdate < throttleMs) {
        return;
      }

      const cursor: Cursor = {
        participantId: context.currentParticipant?.id || '',
        position,
        timestamp: new Date(),
      };

      context.updatePresence({ cursor });
      setLastUpdate(now);
    },
    [context, lastUpdate, throttleMs]
  );

  /**
   * Update selection with throttling
   */
  const updateSelection = useCallback(
    (selection: Selection) => {
      const now = Date.now();
      if (now - lastUpdate < throttleMs) {
        return;
      }

      context.updatePresence({ selection });
      setLastUpdate(now);
    },
    [context, lastUpdate, throttleMs]
  );

  /**
   * Get cursor for a specific participant
   */
  const getCursorForParticipant = useCallback(
    (participantId: string): Cursor | undefined => {
      return cursors.get(participantId);
    },
    [cursors]
  );

  /**
   * Get selection for a specific participant
   */
  const getSelectionForParticipant = useCallback(
    (participantId: string): Selection | undefined => {
      return selections.get(participantId);
    },
    [selections]
  );

  /**
   * Get visible cursors with participant info
   */
  const visibleCursors = Array.from(cursors.entries())
    .filter(([participantId]) => {
      if (!showOwnCursor && participantId === context.currentParticipant?.id) {
        return false;
      }
      return true;
    })
    .map(([participantId, cursor]) => {
      const participant = context.participants.get(participantId);
      return participant ? { participant, cursor } : null;
    })
    .filter((item): item is { participant: Participant; cursor: Cursor } => item !== null);

  return {
    cursors,
    selections,
    updateCursor,
    updateSelection,
    getCursorForParticipant,
    getSelectionForParticipant,
    visibleCursors,
  };
};
