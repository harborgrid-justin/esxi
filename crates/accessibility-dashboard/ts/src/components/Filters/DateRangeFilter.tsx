/**
 * Date Range Filter Component
 * Accessible date range picker with presets
 */

import React, { useId } from 'react';
import { format, subDays, subMonths, startOfDay, endOfDay } from 'date-fns';
import { useDashboardContext } from '../../context/DashboardContext';

const DATE_PRESETS = [
  { value: 'week', label: 'Last 7 days', days: 7 },
  { value: 'month', label: 'Last 30 days', days: 30 },
  { value: 'quarter', label: 'Last 90 days', days: 90 },
  { value: 'year', label: 'Last 365 days', days: 365 },
] as const;

export const DateRangeFilter: React.FC = () => {
  const { filters, setDateRange, dateRangePreset, setDateRangePreset } =
    useDashboardContext();
  const startId = useId();
  const endId = useId();
  const presetId = useId();

  const handlePresetChange = (preset: typeof dateRangePreset) => {
    setDateRangePreset(preset);

    if (preset === 'custom') {
      return;
    }

    const presetConfig = DATE_PRESETS.find((p) => p.value === preset);
    if (!presetConfig) return;

    const end = endOfDay(new Date());
    const start = startOfDay(subDays(end, presetConfig.days));

    setDateRange(start, end);
  };

  const handleStartDateChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const date = e.target.value ? startOfDay(new Date(e.target.value)) : null;
    setDateRange(date, filters.dateRange.end);
    setDateRangePreset('custom');
  };

  const handleEndDateChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const date = e.target.value ? endOfDay(new Date(e.target.value)) : null;
    setDateRange(filters.dateRange.start, date);
    setDateRangePreset('custom');
  };

  const handleClear = () => {
    setDateRange(null, null);
    setDateRangePreset('month');
  };

  const formatDateForInput = (date: Date | null): string => {
    if (!date) return '';
    return format(date, 'yyyy-MM-dd');
  };

  return (
    <fieldset className="border border-gray-300 rounded-lg p-4 bg-white">
      <legend className="text-sm font-semibold text-gray-900 px-2">
        Date Range
      </legend>

      <div className="mt-2 space-y-3">
        {/* Preset selector */}
        <div>
          <label
            htmlFor={presetId}
            className="block text-sm font-medium text-gray-700 mb-1"
          >
            Quick Select
          </label>
          <select
            id={presetId}
            value={dateRangePreset}
            onChange={(e) =>
              handlePresetChange(
                e.target.value as typeof dateRangePreset
              )
            }
            className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 text-sm"
          >
            {DATE_PRESETS.map((preset) => (
              <option key={preset.value} value={preset.value}>
                {preset.label}
              </option>
            ))}
            <option value="custom">Custom Range</option>
          </select>
        </div>

        {/* Start date */}
        <div>
          <label
            htmlFor={startId}
            className="block text-sm font-medium text-gray-700 mb-1"
          >
            Start Date
          </label>
          <input
            id={startId}
            type="date"
            value={formatDateForInput(filters.dateRange.start)}
            onChange={handleStartDateChange}
            className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 text-sm"
            aria-describedby={`${startId}-hint`}
          />
          <p id={`${startId}-hint`} className="mt-1 text-xs text-gray-500">
            Filter issues detected after this date
          </p>
        </div>

        {/* End date */}
        <div>
          <label
            htmlFor={endId}
            className="block text-sm font-medium text-gray-700 mb-1"
          >
            End Date
          </label>
          <input
            id={endId}
            type="date"
            value={formatDateForInput(filters.dateRange.end)}
            onChange={handleEndDateChange}
            className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 text-sm"
            aria-describedby={`${endId}-hint`}
          />
          <p id={`${endId}-hint`} className="mt-1 text-xs text-gray-500">
            Filter issues detected before this date
          </p>
        </div>

        {/* Clear button */}
        <button
          type="button"
          onClick={handleClear}
          className="w-full px-3 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
          aria-label="Clear date range filter"
        >
          Clear Date Range
        </button>
      </div>

      {/* Selected range display */}
      {(filters.dateRange.start || filters.dateRange.end) && (
        <div
          className="mt-3 pt-3 border-t border-gray-200 text-xs text-gray-600"
          role="status"
          aria-live="polite"
        >
          <strong>Selected Range:</strong>
          <br />
          {filters.dateRange.start
            ? format(filters.dateRange.start, 'MMM d, yyyy')
            : 'Beginning'}{' '}
          to{' '}
          {filters.dateRange.end
            ? format(filters.dateRange.end, 'MMM d, yyyy')
            : 'Present'}
        </div>
      )}
    </fieldset>
  );
};
