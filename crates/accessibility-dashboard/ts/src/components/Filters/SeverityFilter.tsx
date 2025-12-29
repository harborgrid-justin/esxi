/**
 * Severity Filter Component
 * Accessible multi-select filter for issue severity levels
 */

import React, { useId } from 'react';
import { useDashboardContext } from '../../context/DashboardContext';
import { getSeverityColor } from '../../utils/calculations';
import type { IssueSeverity } from '../../types';

const SEVERITIES: {
  value: IssueSeverity;
  label: string;
  description: string;
}[] = [
  {
    value: 'critical',
    label: 'Critical',
    description: 'Blocks accessibility for users',
  },
  {
    value: 'serious',
    label: 'Serious',
    description: 'Significant barrier to accessibility',
  },
  {
    value: 'moderate',
    label: 'Moderate',
    description: 'May cause difficulty',
  },
  {
    value: 'minor',
    label: 'Minor',
    description: 'Minor inconvenience',
  },
];

export const SeverityFilter: React.FC = () => {
  const { filters, setSeverities } = useDashboardContext();
  const groupId = useId();

  const handleToggle = (severity: IssueSeverity) => {
    const newSeverities = filters.severities.includes(severity)
      ? filters.severities.filter((s) => s !== severity)
      : [...filters.severities, severity];

    setSeverities(newSeverities);
  };

  const handleSelectAll = () => {
    setSeverities(['critical', 'serious', 'moderate', 'minor']);
  };

  const handleDeselectAll = () => {
    setSeverities([]);
  };

  return (
    <fieldset className="border border-gray-300 rounded-lg p-4 bg-white">
      <legend className="text-sm font-semibold text-gray-900 px-2">
        Issue Severity
      </legend>

      <div className="mt-2 space-y-2">
        {SEVERITIES.map((severity) => {
          const checkboxId = `${groupId}-${severity.value}`;
          const isChecked = filters.severities.includes(severity.value);
          const color = getSeverityColor(severity.value);

          return (
            <div key={severity.value} className="flex items-start">
              <div className="flex items-center h-5">
                <input
                  id={checkboxId}
                  type="checkbox"
                  checked={isChecked}
                  onChange={() => handleToggle(severity.value)}
                  className="w-4 h-4 border-gray-300 rounded focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
                  style={{ accentColor: color }}
                  aria-describedby={`${checkboxId}-description`}
                />
              </div>
              <div className="ml-3 text-sm">
                <label
                  htmlFor={checkboxId}
                  className="font-medium text-gray-700 cursor-pointer flex items-center gap-2"
                >
                  <span
                    className="w-3 h-3 rounded-full"
                    style={{ backgroundColor: color }}
                    aria-hidden="true"
                  />
                  {severity.label}
                </label>
                <p
                  id={`${checkboxId}-description`}
                  className="text-gray-500 text-xs"
                >
                  {severity.description}
                </p>
              </div>
            </div>
          );
        })}
      </div>

      <div className="mt-3 pt-3 border-t border-gray-200 flex gap-2">
        <button
          type="button"
          onClick={handleSelectAll}
          className="text-xs text-blue-600 hover:text-blue-800 font-medium focus:outline-none focus:underline"
          aria-label="Select all severity levels"
        >
          Select All
        </button>
        <span className="text-gray-300">|</span>
        <button
          type="button"
          onClick={handleDeselectAll}
          className="text-xs text-blue-600 hover:text-blue-800 font-medium focus:outline-none focus:underline"
          aria-label="Deselect all severity levels"
        >
          Deselect All
        </button>
      </div>
    </fieldset>
  );
};
