/**
 * useConflictResolution Hook
 * Hook for handling conflict resolution
 */

import { useState, useCallback, useEffect } from 'react';
import { useCollaborationContext } from '../components/CollaborationProvider';
import type {
  Conflict,
  ConflictResolution,
  ConflictResolutionStrategy,
  Operation,
} from '../types';

export interface UseConflictResolutionOptions {
  autoResolve?: boolean;
  defaultStrategy?: ConflictResolutionStrategy;
  onConflictDetected?: (conflict: Conflict) => void;
  onConflictResolved?: (conflict: Conflict, resolution: ConflictResolution) => void;
}

export interface UseConflictResolutionResult {
  conflicts: Conflict[];
  unresolvedConflicts: Conflict[];
  resolvedConflicts: Conflict[];
  hasConflicts: boolean;
  resolveConflict: (conflictId: string, strategy: ConflictResolutionStrategy, selectedOperation?: string) => void;
  resolveAll: (strategy: ConflictResolutionStrategy) => void;
  dismissConflict: (conflictId: string) => void;
  getConflict: (conflictId: string) => Conflict | undefined;
}

/**
 * Hook for conflict resolution
 */
export const useConflictResolution = (
  options: UseConflictResolutionOptions = {}
): UseConflictResolutionResult => {
  const {
    autoResolve = false,
    defaultStrategy = ConflictResolutionStrategy.LAST_WRITE_WINS,
    onConflictDetected,
    onConflictResolved,
  } = options;

  const context = useCollaborationContext();
  const [dismissedConflicts, setDismissedConflicts] = useState<Set<string>>(new Set());

  // Auto-resolve conflicts if enabled
  useEffect(() => {
    if (!autoResolve || context.conflicts.length === 0) return;

    context.conflicts
      .filter((conflict) => !conflict.resolved)
      .forEach((conflict) => {
        resolveConflict(conflict.id, defaultStrategy);
      });
  }, [autoResolve, context.conflicts, defaultStrategy]);

  // Notify on conflict detection
  useEffect(() => {
    if (!onConflictDetected) return;

    const newConflicts = context.conflicts.filter(
      (conflict) => !conflict.resolved && !dismissedConflicts.has(conflict.id)
    );

    newConflicts.forEach((conflict) => {
      onConflictDetected(conflict);
    });
  }, [context.conflicts, onConflictDetected, dismissedConflicts]);

  /**
   * Resolve a specific conflict
   */
  const resolveConflict = useCallback(
    (
      conflictId: string,
      strategy: ConflictResolutionStrategy,
      selectedOperation?: string
    ) => {
      const conflict = context.conflicts.find((c) => c.id === conflictId);
      if (!conflict) return;

      const resolution: ConflictResolution = {
        conflictId,
        strategy,
        selectedOperation,
        resolvedBy: context.currentParticipant?.id || 'unknown',
        resolvedAt: new Date(),
      };

      context.resolveConflict(conflictId, resolution);

      if (onConflictResolved) {
        onConflictResolved(conflict, resolution);
      }
    },
    [context, onConflictResolved]
  );

  /**
   * Resolve all conflicts with the same strategy
   */
  const resolveAll = useCallback(
    (strategy: ConflictResolutionStrategy) => {
      context.conflicts
        .filter((conflict) => !conflict.resolved)
        .forEach((conflict) => {
          resolveConflict(conflict.id, strategy);
        });
    },
    [context.conflicts, resolveConflict]
  );

  /**
   * Dismiss a conflict without resolving
   */
  const dismissConflict = useCallback((conflictId: string) => {
    setDismissedConflicts((prev) => new Set(prev).add(conflictId));
  }, []);

  /**
   * Get a specific conflict
   */
  const getConflict = useCallback(
    (conflictId: string): Conflict | undefined => {
      return context.conflicts.find((c) => c.id === conflictId);
    },
    [context.conflicts]
  );

  const unresolvedConflicts = context.conflicts.filter(
    (conflict) => !conflict.resolved && !dismissedConflicts.has(conflict.id)
  );

  const resolvedConflicts = context.conflicts.filter((conflict) => conflict.resolved);

  return {
    conflicts: context.conflicts,
    unresolvedConflicts,
    resolvedConflicts,
    hasConflicts: unresolvedConflicts.length > 0,
    resolveConflict,
    resolveAll,
    dismissConflict,
    getConflict,
  };
};
