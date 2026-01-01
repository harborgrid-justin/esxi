/**
 * Conflict Resolver Component
 * UI for resolving document conflicts
 */

import React, { useState } from 'react';
import { Conflict, ConflictResolutionStrategy, Operation } from '../types';

export interface ConflictResolverProps {
  conflicts: Conflict[];
  onResolve: (conflictId: string, resolution: any) => void;
  onDismiss?: (conflictId: string) => void;
  className?: string;
}

export const ConflictResolver: React.FC<ConflictResolverProps> = ({
  conflicts,
  onResolve,
  onDismiss,
  className = '',
}) => {
  const [selectedConflict, setSelectedConflict] = useState<string | null>(null);
  const [selectedStrategy, setSelectedStrategy] = useState<ConflictResolutionStrategy>(
    ConflictResolutionStrategy.LAST_WRITE_WINS
  );

  if (conflicts.length === 0) {
    return null;
  }

  const handleResolve = (conflict: Conflict, operationId?: string) => {
    const resolution = {
      strategy: selectedStrategy,
      selectedOperation: operationId,
      resolvedAt: new Date(),
    };

    onResolve(conflict.id, resolution);
    setSelectedConflict(null);
  };

  const formatOperation = (operation: Operation): string => {
    const type = operation.type.toUpperCase();
    const content = operation.content
      ? String(operation.content).substring(0, 50)
      : '';
    const timestamp = new Date(operation.timestamp).toLocaleTimeString();

    return `${type}: "${content}" at ${timestamp}`;
  };

  return (
    <div className={`bg-white border border-red-300 rounded-lg shadow-lg ${className}`}>
      <div className="bg-red-50 border-b border-red-300 px-4 py-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-2">
            <svg
              className="w-5 h-5 text-red-600"
              fill="currentColor"
              viewBox="0 0 20 20"
            >
              <path
                fillRule="evenodd"
                d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z"
                clipRule="evenodd"
              />
            </svg>
            <h3 className="text-lg font-semibold text-red-900">
              {conflicts.length} Conflict{conflicts.length > 1 ? 's' : ''} Detected
            </h3>
          </div>
        </div>
      </div>

      <div className="p-4 space-y-4">
        {conflicts.map((conflict) => (
          <div
            key={conflict.id}
            className="border border-gray-200 rounded-lg overflow-hidden"
          >
            <div className="bg-gray-50 px-4 py-2 border-b border-gray-200">
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium text-gray-700">
                  {conflict.type.replace('_', ' ').toUpperCase()}
                </span>
                <span className="text-xs text-gray-500">
                  {new Date(conflict.detectedAt).toLocaleString()}
                </span>
              </div>
            </div>

            <div className="p-4 space-y-3">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Resolution Strategy
                </label>
                <select
                  value={selectedStrategy}
                  onChange={(e) => setSelectedStrategy(e.target.value as ConflictResolutionStrategy)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                  <option value={ConflictResolutionStrategy.LAST_WRITE_WINS}>
                    Last Write Wins
                  </option>
                  <option value={ConflictResolutionStrategy.FIRST_WRITE_WINS}>
                    First Write Wins
                  </option>
                  <option value={ConflictResolutionStrategy.MERGE}>
                    Merge Changes
                  </option>
                  <option value={ConflictResolutionStrategy.MANUAL}>
                    Manual Selection
                  </option>
                </select>
              </div>

              {selectedStrategy === ConflictResolutionStrategy.MANUAL && (
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    Choose Version
                  </label>
                  <div className="space-y-2">
                    {conflict.operations.map((operation) => (
                      <button
                        key={operation.id}
                        onClick={() => handleResolve(conflict, operation.id)}
                        className="w-full text-left px-4 py-3 border border-gray-300 rounded-md hover:bg-blue-50 hover:border-blue-500 transition-colors"
                      >
                        <div className="text-sm font-medium text-gray-900">
                          {formatOperation(operation)}
                        </div>
                        <div className="text-xs text-gray-500 mt-1">
                          By: {operation.participantId}
                        </div>
                      </button>
                    ))}
                  </div>
                </div>
              )}

              <div className="flex space-x-2">
                <button
                  onClick={() => handleResolve(conflict)}
                  className="flex-1 px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors"
                >
                  Resolve
                </button>
                {onDismiss && (
                  <button
                    onClick={() => onDismiss(conflict.id)}
                    className="px-4 py-2 bg-gray-200 text-gray-700 rounded-md hover:bg-gray-300 transition-colors"
                  >
                    Dismiss
                  </button>
                )}
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};
