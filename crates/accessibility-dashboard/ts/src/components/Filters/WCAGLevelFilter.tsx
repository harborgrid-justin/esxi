/**
 * WCAG Level Filter Component
 * Accessible multi-select filter for WCAG conformance levels
 */

import React, { useId } from 'react';
import { useDashboardContext } from '../../context/DashboardContext';
import type { WCAGLevel } from '../../types';

const WCAG_LEVELS: { value: WCAGLevel; label: string; description: string }[] = [
  { value: 'A', label: 'Level A', description: 'Minimum conformance' },
  { value: 'AA', label: 'Level AA', description: 'Acceptable conformance' },
  { value: 'AAA', label: 'Level AAA', description: 'Optimal conformance' },
];

export const WCAGLevelFilter: React.FC = () => {
  const { filters, setWCAGLevels } = useDashboardContext();
  const groupId = useId();

  const handleToggle = (level: WCAGLevel) => {
    const newLevels = filters.wcagLevels.includes(level)
      ? filters.wcagLevels.filter((l) => l !== level)
      : [...filters.wcagLevels, level];

    setWCAGLevels(newLevels);
  };

  const handleSelectAll = () => {
    setWCAGLevels(['A', 'AA', 'AAA']);
  };

  const handleDeselectAll = () => {
    setWCAGLevels([]);
  };

  return (
    <fieldset className="border border-gray-300 rounded-lg p-4 bg-white">
      <legend className="text-sm font-semibold text-gray-900 px-2">
        WCAG Conformance Level
      </legend>

      <div className="mt-2 space-y-2">
        {WCAG_LEVELS.map((level) => {
          const checkboxId = `${groupId}-${level.value}`;
          const isChecked = filters.wcagLevels.includes(level.value);

          return (
            <div key={level.value} className="flex items-start">
              <div className="flex items-center h-5">
                <input
                  id={checkboxId}
                  type="checkbox"
                  checked={isChecked}
                  onChange={() => handleToggle(level.value)}
                  className="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
                  aria-describedby={`${checkboxId}-description`}
                />
              </div>
              <div className="ml-3 text-sm">
                <label
                  htmlFor={checkboxId}
                  className="font-medium text-gray-700 cursor-pointer"
                >
                  {level.label}
                </label>
                <p
                  id={`${checkboxId}-description`}
                  className="text-gray-500 text-xs"
                >
                  {level.description}
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
          aria-label="Select all WCAG levels"
        >
          Select All
        </button>
        <span className="text-gray-300">|</span>
        <button
          type="button"
          onClick={handleDeselectAll}
          className="text-xs text-blue-600 hover:text-blue-800 font-medium focus:outline-none focus:underline"
          aria-label="Deselect all WCAG levels"
        >
          Deselect All
        </button>
      </div>
    </fieldset>
  );
};
