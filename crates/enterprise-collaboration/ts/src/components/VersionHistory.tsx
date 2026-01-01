/**
 * Version History Component
 * Displays document version history with diff view
 */

import React, { useState } from 'react';
import { Version, VersionDiff } from '../types';

export interface VersionHistoryProps {
  versions: Version[];
  currentVersion: number;
  onRestore?: (versionId: string) => void;
  onCompare?: (fromVersion: number, toVersion: number) => void;
  className?: string;
}

export const VersionHistory: React.FC<VersionHistoryProps> = ({
  versions,
  currentVersion,
  onRestore,
  onCompare,
  className = '',
}) => {
  const [selectedVersion, setSelectedVersion] = useState<string | null>(null);
  const [compareMode, setCompareMode] = useState(false);
  const [compareFrom, setCompareFrom] = useState<number | null>(null);
  const [compareTo, setCompareTo] = useState<number | null>(null);

  const sortedVersions = [...versions].sort((a, b) => b.number - a.number);

  const handleVersionClick = (version: Version) => {
    if (compareMode) {
      if (compareFrom === null) {
        setCompareFrom(version.number);
      } else if (compareTo === null) {
        setCompareTo(version.number);
        onCompare?.(compareFrom, version.number);
      } else {
        setCompareFrom(version.number);
        setCompareTo(null);
      }
    } else {
      setSelectedVersion(version.id);
    }
  };

  const handleRestore = (versionId: string) => {
    if (window.confirm('Are you sure you want to restore this version?')) {
      onRestore?.(versionId);
      setSelectedVersion(null);
    }
  };

  const formatTimestamp = (date: Date): string => {
    const now = new Date();
    const diff = now.getTime() - new Date(date).getTime();
    const seconds = Math.floor(diff / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);

    if (days > 0) return `${days} day${days > 1 ? 's' : ''} ago`;
    if (hours > 0) return `${hours} hour${hours > 1 ? 's' : ''} ago`;
    if (minutes > 0) return `${minutes} minute${minutes > 1 ? 's' : ''} ago`;
    return 'Just now';
  };

  return (
    <div className={`bg-white border border-gray-200 rounded-lg shadow ${className}`}>
      <div className="border-b border-gray-200 px-4 py-3">
        <div className="flex items-center justify-between">
          <h3 className="text-lg font-semibold text-gray-900">Version History</h3>
          <button
            onClick={() => {
              setCompareMode(!compareMode);
              setCompareFrom(null);
              setCompareTo(null);
            }}
            className={`px-3 py-1 text-sm rounded ${
              compareMode
                ? 'bg-blue-600 text-white'
                : 'bg-gray-200 text-gray-700 hover:bg-gray-300'
            } transition-colors`}
          >
            {compareMode ? 'Exit Compare' : 'Compare'}
          </button>
        </div>
      </div>

      <div className="divide-y divide-gray-200 max-h-96 overflow-y-auto">
        {sortedVersions.map((version) => {
          const isCurrent = version.number === currentVersion;
          const isSelected = selectedVersion === version.id;
          const isCompareSelected =
            compareFrom === version.number || compareTo === version.number;

          return (
            <div
              key={version.id}
              className={`px-4 py-3 hover:bg-gray-50 cursor-pointer transition-colors ${
                isSelected || isCompareSelected ? 'bg-blue-50' : ''
              }`}
              onClick={() => handleVersionClick(version)}
            >
              <div className="flex items-start justify-between">
                <div className="flex-1 min-w-0">
                  <div className="flex items-center space-x-2">
                    <span className="text-sm font-medium text-gray-900">
                      Version {version.number}
                    </span>
                    {isCurrent && (
                      <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-green-100 text-green-800">
                        Current
                      </span>
                    )}
                    {version.label && (
                      <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800">
                        {version.label}
                      </span>
                    )}
                    {isCompareSelected && (
                      <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-purple-100 text-purple-800">
                        {compareFrom === version.number ? 'From' : 'To'}
                      </span>
                    )}
                  </div>

                  {version.description && (
                    <p className="mt-1 text-sm text-gray-600 truncate">
                      {version.description}
                    </p>
                  )}

                  <div className="mt-2 flex items-center space-x-4 text-xs text-gray-500">
                    <span>{formatTimestamp(version.createdAt)}</span>
                    <span>By: {version.createdBy}</span>
                    <span>{version.operations.length} changes</span>
                  </div>
                </div>

                {!isCurrent && onRestore && !compareMode && (
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      handleRestore(version.id);
                    }}
                    className="ml-4 px-3 py-1 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
                  >
                    Restore
                  </button>
                )}
              </div>

              {isSelected && version.tags && version.tags.length > 0 && (
                <div className="mt-3 flex flex-wrap gap-1">
                  {version.tags.map((tag) => (
                    <span
                      key={tag}
                      className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-700"
                    >
                      {tag}
                    </span>
                  ))}
                </div>
              )}
            </div>
          );
        })}

        {sortedVersions.length === 0 && (
          <div className="px-4 py-8 text-center text-gray-500">
            No version history available
          </div>
        )}
      </div>

      {compareMode && compareFrom !== null && compareTo !== null && (
        <div className="border-t border-gray-200 px-4 py-3 bg-gray-50">
          <div className="text-sm text-gray-700">
            Comparing version {Math.min(compareFrom, compareTo)} to{' '}
            {Math.max(compareFrom, compareTo)}
          </div>
        </div>
      )}
    </div>
  );
};
