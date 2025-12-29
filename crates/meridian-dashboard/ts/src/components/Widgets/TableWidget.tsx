/**
 * TableWidget - Data table widget with sorting and filtering
 */

import React, { useState, useMemo } from 'react';
import { Widget, TableWidgetConfig } from '../../types';
import { useDataSource } from '../../hooks/useDataSource';

export interface TableWidgetProps {
  widget: Widget;
}

export const TableWidget: React.FC<TableWidgetProps> = ({ widget }) => {
  const { data, loading, error } = useDataSource(widget.data_source);
  const config = widget.config as TableWidgetConfig;

  const [currentPage, setCurrentPage] = useState(1);
  const [sortColumn, setSortColumn] = useState<string | null>(null);
  const [sortDirection, setSortDirection] = useState<'asc' | 'desc'>('asc');

  const tableData = useMemo(() => {
    if (!Array.isArray(data)) return [];

    let sorted = [...data];

    // Apply sorting
    if (sortColumn) {
      sorted.sort((a, b) => {
        const aVal = a[sortColumn];
        const bVal = b[sortColumn];
        if (aVal < bVal) return sortDirection === 'asc' ? -1 : 1;
        if (aVal > bVal) return sortDirection === 'asc' ? 1 : -1;
        return 0;
      });
    }

    // Apply pagination
    if (config.pagination) {
      const start = (currentPage - 1) * config.page_size;
      const end = start + config.page_size;
      return sorted.slice(start, end);
    }

    return sorted;
  }, [data, sortColumn, sortDirection, currentPage, config]);

  const totalPages = config.pagination
    ? Math.ceil((Array.isArray(data) ? data.length : 0) / config.page_size)
    : 1;

  const handleSort = (field: string) => {
    if (sortColumn === field) {
      setSortDirection(sortDirection === 'asc' ? 'desc' : 'asc');
    } else {
      setSortColumn(field);
      setSortDirection('asc');
    }
  };

  if (loading) {
    return <div className="table-loading">Loading data...</div>;
  }

  if (error) {
    return <div className="table-error">Error: {error.message}</div>;
  }

  return (
    <div className="table-widget">
      <div className="table-container">
        <table className="data-table">
          <thead>
            <tr>
              {config.columns.map((column) => (
                <th
                  key={column.field}
                  style={{ width: column.width ? `${column.width}px` : 'auto' }}
                  onClick={() =>
                    column.sortable ? handleSort(column.field) : undefined
                  }
                  className={column.sortable ? 'sortable' : ''}
                >
                  {column.header}
                  {sortColumn === column.field && (
                    <span className="sort-indicator">
                      {sortDirection === 'asc' ? ' ↑' : ' ↓'}
                    </span>
                  )}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {tableData.map((row, index) => (
              <tr key={index}>
                {config.columns.map((column) => (
                  <td key={column.field}>{formatValue(row[column.field], column.format)}</td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {config.pagination && (
        <div className="pagination">
          <button
            disabled={currentPage === 1}
            onClick={() => setCurrentPage(currentPage - 1)}
          >
            Previous
          </button>
          <span>
            Page {currentPage} of {totalPages}
          </span>
          <button
            disabled={currentPage === totalPages}
            onClick={() => setCurrentPage(currentPage + 1)}
          >
            Next
          </button>
        </div>
      )}

      <style jsx>{`
        .table-widget {
          width: 100%;
          height: 100%;
          display: flex;
          flex-direction: column;
        }

        .table-container {
          flex: 1;
          overflow: auto;
        }

        .data-table {
          width: 100%;
          border-collapse: collapse;
        }

        .data-table th {
          background: #f5f5f5;
          padding: 12px;
          text-align: left;
          font-weight: 600;
          border-bottom: 2px solid #ddd;
          position: sticky;
          top: 0;
        }

        .data-table th.sortable {
          cursor: pointer;
          user-select: none;
        }

        .data-table th.sortable:hover {
          background: #e8e8e8;
        }

        .sort-indicator {
          font-size: 12px;
        }

        .data-table td {
          padding: 10px 12px;
          border-bottom: 1px solid #eee;
        }

        .data-table tbody tr:hover {
          background: #f9f9f9;
        }

        .pagination {
          display: flex;
          justify-content: center;
          align-items: center;
          gap: 16px;
          padding: 12px;
          border-top: 1px solid #e0e0e0;
        }

        .pagination button {
          padding: 6px 12px;
          border: 1px solid #ddd;
          background: white;
          cursor: pointer;
          border-radius: 4px;
        }

        .pagination button:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        .table-loading,
        .table-error {
          display: flex;
          align-items: center;
          justify-content: center;
          height: 100%;
          color: #666;
        }
      `}</style>
    </div>
  );
};

function formatValue(value: any, format?: string): string {
  if (value === null || value === undefined) return '';
  if (!format) return String(value);

  // Simple formatting - extend as needed
  if (format === 'currency') {
    return `$${Number(value).toFixed(2)}`;
  }
  if (format === 'percent') {
    return `${(Number(value) * 100).toFixed(2)}%`;
  }
  if (format === 'date') {
    return new Date(value).toLocaleDateString();
  }

  return String(value);
}
