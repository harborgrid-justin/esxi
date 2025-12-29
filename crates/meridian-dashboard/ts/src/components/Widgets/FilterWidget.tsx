/**
 * FilterWidget - Interactive filter/slicer widget
 */

import React, { useState, useEffect } from 'react';
import { Widget, FilterWidgetConfig } from '../../types';
import { useDashboardContext } from '../../context/DashboardContext';
import { useDataSource } from '../../hooks/useDataSource';

export interface FilterWidgetProps {
  widget: Widget;
}

export const FilterWidget: React.FC<FilterWidgetProps> = ({ widget }) => {
  const { applyFilter } = useDashboardContext();
  const { data } = useDataSource(widget.data_source);
  const config = widget.config as FilterWidgetConfig;

  const [selectedValue, setSelectedValue] = useState<any>(
    config.default_value || null
  );

  const uniqueValues = React.useMemo(() => {
    if (!Array.isArray(data)) return [];
    const values = data
      .map((item) => item[config.field])
      .filter((val) => val !== null && val !== undefined);
    return [...new Set(values)].sort();
  }, [data, config.field]);

  useEffect(() => {
    if (selectedValue !== null) {
      applyFilter({
        id: widget.id,
        field: config.field,
        operator: 'equals',
        value: selectedValue,
        applies_to: [], // Will apply to all widgets
      });
    }
  }, [selectedValue, widget.id, config.field, applyFilter]);

  const renderFilterControl = () => {
    switch (config.filter_type) {
      case 'select':
        return (
          <select
            value={selectedValue || ''}
            onChange={(e) => setSelectedValue(e.target.value || null)}
            className="filter-select"
          >
            <option value="">All</option>
            {uniqueValues.map((value, index) => (
              <option key={index} value={String(value)}>
                {String(value)}
              </option>
            ))}
          </select>
        );

      case 'multi_select':
        return (
          <div className="filter-checkboxes">
            {uniqueValues.map((value, index) => {
              const selected = Array.isArray(selectedValue)
                ? selectedValue.includes(value)
                : false;

              return (
                <label key={index} className="checkbox-label">
                  <input
                    type="checkbox"
                    checked={selected}
                    onChange={(e) => {
                      const current = Array.isArray(selectedValue)
                        ? [...selectedValue]
                        : [];
                      if (e.target.checked) {
                        setSelectedValue([...current, value]);
                      } else {
                        setSelectedValue(current.filter((v) => v !== value));
                      }
                    }}
                  />
                  <span>{String(value)}</span>
                </label>
              );
            })}
          </div>
        );

      case 'date_range':
        return (
          <div className="date-range">
            <input
              type="date"
              value={selectedValue?.start || ''}
              onChange={(e) =>
                setSelectedValue({
                  ...selectedValue,
                  start: e.target.value,
                })
              }
              className="filter-input"
            />
            <span>to</span>
            <input
              type="date"
              value={selectedValue?.end || ''}
              onChange={(e) =>
                setSelectedValue({
                  ...selectedValue,
                  end: e.target.value,
                })
              }
              className="filter-input"
            />
          </div>
        );

      case 'number_range':
        return (
          <div className="number-range">
            <input
              type="number"
              placeholder="Min"
              value={selectedValue?.min || ''}
              onChange={(e) =>
                setSelectedValue({
                  ...selectedValue,
                  min: e.target.value,
                })
              }
              className="filter-input"
            />
            <span>to</span>
            <input
              type="number"
              placeholder="Max"
              value={selectedValue?.max || ''}
              onChange={(e) =>
                setSelectedValue({
                  ...selectedValue,
                  max: e.target.value,
                })
              }
              className="filter-input"
            />
          </div>
        );

      case 'text':
        return (
          <input
            type="text"
            value={selectedValue || ''}
            onChange={(e) => setSelectedValue(e.target.value || null)}
            placeholder={`Search ${config.field}...`}
            className="filter-input"
          />
        );

      default:
        return <div>Unsupported filter type</div>;
    }
  };

  return (
    <div className="filter-widget">
      <div className="filter-label">{config.field}</div>
      <div className="filter-control">{renderFilterControl()}</div>

      {selectedValue && (
        <button
          className="clear-button"
          onClick={() => setSelectedValue(config.default_value || null)}
        >
          Clear
        </button>
      )}

      <style jsx>{`
        .filter-widget {
          padding: 8px 0;
        }

        .filter-label {
          font-size: 12px;
          font-weight: 600;
          color: #666;
          margin-bottom: 8px;
          text-transform: uppercase;
        }

        .filter-control {
          margin-bottom: 8px;
        }

        .filter-select,
        .filter-input {
          width: 100%;
          padding: 8px 12px;
          border: 1px solid #ddd;
          border-radius: 4px;
          font-size: 14px;
          transition: border-color 0.2s;
        }

        .filter-select:focus,
        .filter-input:focus {
          outline: none;
          border-color: #1976d2;
        }

        .filter-checkboxes {
          max-height: 200px;
          overflow-y: auto;
        }

        .checkbox-label {
          display: flex;
          align-items: center;
          gap: 8px;
          padding: 6px 0;
          cursor: pointer;
          font-size: 14px;
        }

        .checkbox-label input[type='checkbox'] {
          cursor: pointer;
        }

        .date-range,
        .number-range {
          display: flex;
          align-items: center;
          gap: 8px;
        }

        .date-range input,
        .number-range input {
          flex: 1;
        }

        .clear-button {
          padding: 6px 12px;
          border: 1px solid #ddd;
          background: white;
          border-radius: 4px;
          font-size: 13px;
          cursor: pointer;
          transition: all 0.2s;
          width: 100%;
        }

        .clear-button:hover {
          background: #f5f5f5;
          border-color: #999;
        }
      `}</style>
    </div>
  );
};
