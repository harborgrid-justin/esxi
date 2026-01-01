/**
 * Data Table Component - Tabular Data Display
 * @module @harborgrid/enterprise-analytics/components/dashboard
 */

import React, { useState, useMemo } from 'react';
import type { TableColumn, PaginationConfig } from '../../types';

export interface DataTableProps<T = Record<string, unknown>> {
  data: T[];
  columns: TableColumn[];
  pagination?: PaginationConfig;
  onRowClick?: (row: T) => void;
  striped?: boolean;
  hoverable?: boolean;
}

export function DataTable<T = Record<string, unknown>>({
  data,
  columns,
  pagination,
  onRowClick,
  striped = true,
  hoverable = true,
}: DataTableProps<T>) {
  const [currentPage, setCurrentPage] = useState(1);
  const [sortColumn, setSortColumn] = useState<string | null>(null);
  const [sortDirection, setSortDirection] = useState<'asc' | 'desc'>('asc');
  const [filters, setFilters] = useState<Record<string, string>>({});

  const pageSize = pagination?.pageSize || 25;

  // Apply filtering
  const filteredData = useMemo(() => {
    return data.filter((row) => {
      for (const [field, filterValue] of Object.entries(filters)) {
        if (!filterValue) continue;
        const cellValue = String((row as Record<string, unknown>)[field] || '').toLowerCase();
        if (!cellValue.includes(filterValue.toLowerCase())) {
          return false;
        }
      }
      return true;
    });
  }, [data, filters]);

  // Apply sorting
  const sortedData = useMemo(() => {
    if (!sortColumn) return filteredData;

    return [...filteredData].sort((a, b) => {
      const aVal = (a as Record<string, unknown>)[sortColumn];
      const bVal = (b as Record<string, unknown>)[sortColumn];

      if (aVal === bVal) return 0;

      let comparison = 0;
      if (typeof aVal === 'number' && typeof bVal === 'number') {
        comparison = aVal - bVal;
      } else {
        comparison = String(aVal).localeCompare(String(bVal));
      }

      return sortDirection === 'asc' ? comparison : -comparison;
    });
  }, [filteredData, sortColumn, sortDirection]);

  // Apply pagination
  const paginatedData = useMemo(() => {
    if (!pagination) return sortedData;
    const startIndex = (currentPage - 1) * pageSize;
    return sortedData.slice(startIndex, startIndex + pageSize);
  }, [sortedData, currentPage, pageSize, pagination]);

  const totalPages = Math.ceil(sortedData.length / pageSize);

  const handleSort = (column: string) => {
    if (sortColumn === column) {
      setSortDirection(sortDirection === 'asc' ? 'desc' : 'asc');
    } else {
      setSortColumn(column);
      setSortDirection('asc');
    }
  };

  const handleFilter = (column: string, value: string) => {
    setFilters({ ...filters, [column]: value });
    setCurrentPage(1);
  };

  return (
    <div style={{ width: '100%', overflow: 'auto' }}>
      <table style={{ width: '100%', borderCollapse: 'collapse', backgroundColor: 'white' }}>
        <thead>
          <tr style={{ borderBottom: '2px solid #e0e0e0' }}>
            {columns.map((column) => (
              <th
                key={column.field}
                style={{
                  padding: '12px',
                  textAlign: column.align || 'left',
                  fontWeight: '600',
                  fontSize: '13px',
                  color: '#333',
                  backgroundColor: '#f5f5f5',
                  cursor: column.sortable ? 'pointer' : 'default',
                  userSelect: 'none',
                  width: column.width,
                }}
                onClick={() => column.sortable && handleSort(column.field)}
              >
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                  {column.header}
                  {column.sortable && sortColumn === column.field && (
                    <span>{sortDirection === 'asc' ? '↑' : '↓'}</span>
                  )}
                </div>
                {column.filterable && (
                  <input
                    type="text"
                    placeholder="Filter..."
                    value={filters[column.field] || ''}
                    onChange={(e) => handleFilter(column.field, e.target.value)}
                    onClick={(e) => e.stopPropagation()}
                    style={{
                      marginTop: '8px',
                      padding: '4px 8px',
                      width: '100%',
                      border: '1px solid #ddd',
                      borderRadius: '4px',
                      fontSize: '12px',
                    }}
                  />
                )}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {paginatedData.map((row, rowIndex) => (
            <tr
              key={rowIndex}
              onClick={() => onRowClick?.(row)}
              style={{
                borderBottom: '1px solid #e0e0e0',
                backgroundColor: striped && rowIndex % 2 === 1 ? '#f9f9f9' : 'white',
                cursor: onRowClick ? 'pointer' : 'default',
              }}
              onMouseEnter={(e) => {
                if (hoverable) {
                  e.currentTarget.style.backgroundColor = '#f0f0f0';
                }
              }}
              onMouseLeave={(e) => {
                if (hoverable) {
                  e.currentTarget.style.backgroundColor =
                    striped && rowIndex % 2 === 1 ? '#f9f9f9' : 'white';
                }
              }}
            >
              {columns.map((column) => {
                const value = (row as Record<string, unknown>)[column.field];
                const formatted = column.render
                  ? column.render(value, row as Record<string, unknown>)
                  : column.format
                  ? formatValue(value, column.format)
                  : String(value);

                return (
                  <td
                    key={column.field}
                    style={{
                      padding: '12px',
                      textAlign: column.align || 'left',
                      fontSize: '13px',
                      color: '#666',
                    }}
                  >
                    {formatted}
                  </td>
                );
              })}
            </tr>
          ))}
        </tbody>
      </table>

      {/* Pagination */}
      {pagination && totalPages > 1 && (
        <div
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
            padding: '16px',
            borderTop: '1px solid #e0e0e0',
          }}
        >
          <div style={{ fontSize: '13px', color: '#666' }}>
            Showing {(currentPage - 1) * pageSize + 1} to{' '}
            {Math.min(currentPage * pageSize, sortedData.length)} of {sortedData.length} entries
          </div>
          <div style={{ display: 'flex', gap: '8px' }}>
            <button
              onClick={() => setCurrentPage(Math.max(1, currentPage - 1))}
              disabled={currentPage === 1}
              style={{
                padding: '6px 12px',
                border: '1px solid #ddd',
                borderRadius: '4px',
                backgroundColor: 'white',
                cursor: currentPage === 1 ? 'not-allowed' : 'pointer',
                opacity: currentPage === 1 ? 0.5 : 1,
              }}
            >
              Previous
            </button>
            <div style={{ display: 'flex', gap: '4px' }}>
              {Array.from({ length: totalPages }, (_, i) => i + 1)
                .filter((page) => {
                  return (
                    page === 1 ||
                    page === totalPages ||
                    (page >= currentPage - 2 && page <= currentPage + 2)
                  );
                })
                .map((page, index, array) => {
                  if (index > 0 && page - (array[index - 1] || 0) > 1) {
                    return (
                      <React.Fragment key={`ellipsis-${page}`}>
                        <span style={{ padding: '6px' }}>...</span>
                        <button
                          onClick={() => setCurrentPage(page)}
                          style={{
                            padding: '6px 12px',
                            border: '1px solid #ddd',
                            borderRadius: '4px',
                            backgroundColor: page === currentPage ? '#1f77b4' : 'white',
                            color: page === currentPage ? 'white' : '#333',
                            cursor: 'pointer',
                          }}
                        >
                          {page}
                        </button>
                      </React.Fragment>
                    );
                  }
                  return (
                    <button
                      key={page}
                      onClick={() => setCurrentPage(page)}
                      style={{
                        padding: '6px 12px',
                        border: '1px solid #ddd',
                        borderRadius: '4px',
                        backgroundColor: page === currentPage ? '#1f77b4' : 'white',
                        color: page === currentPage ? 'white' : '#333',
                        cursor: 'pointer',
                      }}
                    >
                      {page}
                    </button>
                  );
                })}
            </div>
            <button
              onClick={() => setCurrentPage(Math.min(totalPages, currentPage + 1))}
              disabled={currentPage === totalPages}
              style={{
                padding: '6px 12px',
                border: '1px solid #ddd',
                borderRadius: '4px',
                backgroundColor: 'white',
                cursor: currentPage === totalPages ? 'not-allowed' : 'pointer',
                opacity: currentPage === totalPages ? 0.5 : 1,
              }}
            >
              Next
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

function formatValue(value: unknown, format: string): string {
  if (value === null || value === undefined) return '';

  if (format.startsWith(',.')) {
    const decimals = parseInt(format.substring(2), 10) || 0;
    if (typeof value === 'number') {
      return value.toLocaleString(undefined, {
        minimumFractionDigits: decimals,
        maximumFractionDigits: decimals,
      });
    }
  } else if (format === '%') {
    if (typeof value === 'number') {
      return `${(value * 100).toFixed(1)}%`;
    }
  } else if (format.includes('date')) {
    if (value instanceof Date) {
      return value.toLocaleDateString();
    }
  }

  return String(value);
}
