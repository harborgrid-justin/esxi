/**
 * AttributeTable Component
 * Tabular display of feature attributes
 * @module @meridian/ui-components/Data
 */

import React, { useState, useMemo } from 'react';
import { useSelection } from '../../hooks/useSelection';
import type { Feature } from '../../types';

export interface AttributeTableProps {
  /** Features to display */
  features: Feature[];
  /** Show only selected features */
  selectedOnly?: boolean;
  /** Enable row selection */
  selectable?: boolean;
  /** Enable sorting */
  sortable?: boolean;
  /** Callback when feature is clicked */
  onFeatureClick?: (feature: Feature) => void;
  /** Callback when table is closed */
  onClose?: () => void;
  /** Custom CSS class */
  className?: string;
}

/**
 * Attribute table for displaying feature properties
 */
export const AttributeTable: React.FC<AttributeTableProps> = ({
  features,
  selectedOnly = false,
  selectable = true,
  sortable = true,
  onFeatureClick,
  onClose,
  className = '',
}) => {
  const { selectedFeatures, selectSingle, isFeatureSelected } = useSelection();
  const [sortColumn, setSortColumn] = useState<string>('');
  const [sortDirection, setSortDirection] = useState<'asc' | 'desc'>('asc');
  const [searchTerm, setSearchTerm] = useState('');

  // Get display features
  const displayFeatures = useMemo(() => {
    return selectedOnly ? selectedFeatures : features;
  }, [selectedOnly, selectedFeatures, features]);

  // Get all unique attribute keys
  const attributeKeys = useMemo(() => {
    const keys = new Set<string>();
    displayFeatures.forEach((feature) => {
      Object.keys(feature.properties).forEach((key) => keys.add(key));
    });
    return Array.from(keys);
  }, [displayFeatures]);

  // Filter and sort features
  const processedFeatures = useMemo(() => {
    let result = [...displayFeatures];

    // Filter by search term
    if (searchTerm) {
      result = result.filter((feature) =>
        Object.values(feature.properties).some((value) =>
          String(value).toLowerCase().includes(searchTerm.toLowerCase())
        )
      );
    }

    // Sort
    if (sortColumn) {
      result.sort((a, b) => {
        const aVal = String(a.properties[sortColumn] || '');
        const bVal = String(b.properties[sortColumn] || '');
        const comparison = aVal.localeCompare(bVal);
        return sortDirection === 'asc' ? comparison : -comparison;
      });
    }

    return result;
  }, [displayFeatures, searchTerm, sortColumn, sortDirection]);

  const handleSort = (column: string) => {
    if (!sortable) return;

    if (sortColumn === column) {
      setSortDirection(sortDirection === 'asc' ? 'desc' : 'asc');
    } else {
      setSortColumn(column);
      setSortDirection('asc');
    }
  };

  const handleRowClick = (feature: Feature) => {
    if (selectable) {
      selectSingle(feature);
    }
    onFeatureClick?.(feature);
  };

  return (
    <div
      className={`meridian-attribute-table bg-white rounded-lg shadow-lg ${className}`}
      role="region"
      aria-label="Attribute table"
    >
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-gray-200">
        <div className="flex items-center gap-4">
          <h3 className="font-semibold text-gray-900">Attributes</h3>
          <span className="text-sm text-gray-500">
            {processedFeatures.length} feature{processedFeatures.length !== 1 ? 's' : ''}
          </span>
        </div>
        <div className="flex items-center gap-2">
          {/* Search */}
          <input
            type="text"
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            placeholder="Search..."
            className="px-3 py-1.5 text-sm border border-gray-300 rounded-lg w-48"
            aria-label="Search attributes"
          />
          {onClose && (
            <button
              onClick={onClose}
              className="w-8 h-8 flex items-center justify-center hover:bg-gray-100 rounded transition-colors"
              aria-label="Close table"
            >
              <svg
                className="w-5 h-5"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M6 18L18 6M6 6l12 12"
                />
              </svg>
            </button>
          )}
        </div>
      </div>

      {/* Table */}
      <div className="overflow-auto max-h-96">
        {processedFeatures.length === 0 ? (
          <div className="text-center py-12 text-gray-500">
            <svg
              className="w-12 h-12 mx-auto mb-3 text-gray-400"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4"
              />
            </svg>
            <p className="text-sm">
              {searchTerm ? 'No features match your search' : 'No features to display'}
            </p>
          </div>
        ) : (
          <table className="w-full text-sm">
            <thead className="bg-gray-50 sticky top-0">
              <tr>
                {selectable && (
                  <th className="w-10 px-3 py-2 text-left font-medium text-gray-700">
                    #
                  </th>
                )}
                {attributeKeys.map((key) => (
                  <th
                    key={key}
                    onClick={() => handleSort(key)}
                    className={`px-3 py-2 text-left font-medium text-gray-700 ${
                      sortable ? 'cursor-pointer hover:bg-gray-100' : ''
                    }`}
                  >
                    <div className="flex items-center gap-2">
                      <span className="truncate">{key}</span>
                      {sortable && sortColumn === key && (
                        <svg
                          className={`w-4 h-4 transition-transform ${
                            sortDirection === 'desc' ? 'rotate-180' : ''
                          }`}
                          fill="none"
                          stroke="currentColor"
                          viewBox="0 0 24 24"
                        >
                          <path
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            strokeWidth={2}
                            d="M5 15l7-7 7 7"
                          />
                        </svg>
                      )}
                    </div>
                  </th>
                ))}
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-200">
              {processedFeatures.map((feature, index) => {
                const selected = isFeatureSelected(feature.id!);
                return (
                  <tr
                    key={feature.id || index}
                    onClick={() => handleRowClick(feature)}
                    className={`${
                      selectable ? 'cursor-pointer hover:bg-blue-50' : ''
                    } ${selected ? 'bg-blue-100' : ''} transition-colors`}
                    aria-selected={selected}
                  >
                    {selectable && (
                      <td className="px-3 py-2 text-gray-600">{index + 1}</td>
                    )}
                    {attributeKeys.map((key) => (
                      <td key={key} className="px-3 py-2 text-gray-900">
                        <div className="truncate max-w-xs">
                          {String(feature.properties[key] ?? '')}
                        </div>
                      </td>
                    ))}
                  </tr>
                );
              })}
            </tbody>
          </table>
        )}
      </div>

      {/* Footer */}
      <div className="flex items-center justify-between p-3 border-t border-gray-200 text-sm text-gray-600">
        <div>
          Showing {processedFeatures.length} of {displayFeatures.length} features
        </div>
        <div className="flex gap-2">
          <button
            onClick={() => {
              /* TODO: Export to CSV */
            }}
            className="px-3 py-1.5 bg-gray-100 hover:bg-gray-200 rounded transition-colors text-sm"
          >
            Export CSV
          </button>
          <button
            onClick={() => {
              /* TODO: Export to JSON */
            }}
            className="px-3 py-1.5 bg-gray-100 hover:bg-gray-200 rounded transition-colors text-sm"
          >
            Export JSON
          </button>
        </div>
      </div>
    </div>
  );
};

export default AttributeTable;
