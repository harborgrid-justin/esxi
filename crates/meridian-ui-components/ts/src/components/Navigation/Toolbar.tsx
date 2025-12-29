/**
 * Toolbar Component
 * Main application toolbar with tool buttons
 * @module @meridian/ui-components/Navigation
 */

import React from 'react';
import type { ToolbarItem, BaseComponentProps } from '../../types';

export interface ToolbarProps extends BaseComponentProps {
  /** Toolbar items */
  items: ToolbarItem[];
  /** Toolbar position */
  position?: 'top' | 'bottom';
  /** Toolbar size */
  size?: 'small' | 'medium' | 'large';
  /** Callback when item is clicked */
  onItemClick?: (item: ToolbarItem) => void;
}

/**
 * Main toolbar component
 */
export const Toolbar: React.FC<ToolbarProps> = ({
  items,
  position = 'top',
  size = 'medium',
  onItemClick,
  className = '',
  ...props
}) => {
  const sizeClasses = {
    small: 'h-12',
    medium: 'h-14',
    large: 'h-16',
  };

  const buttonSizeClasses = {
    small: 'w-8 h-8',
    medium: 'w-10 h-10',
    large: 'w-12 h-12',
  };

  return (
    <div
      className={`meridian-toolbar bg-white border-b border-gray-200 shadow-sm ${sizeClasses[size]} ${className}`}
      role="toolbar"
      {...props}
    >
      <div className="h-full px-4 flex items-center gap-2">
        {/* Logo/Title */}
        <div className="flex items-center gap-2 mr-4">
          <div className="w-8 h-8 bg-blue-600 rounded-lg flex items-center justify-center">
            <svg
              className="w-5 h-5 text-white"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M9 20l-5.447-2.724A1 1 0 013 16.382V5.618a1 1 0 011.447-.894L9 7m0 13l6-3m-6 3V7m6 10l4.553 2.276A1 1 0 0021 18.382V7.618a1 1 0 00-.553-.894L15 4m0 13V4m0 0L9 7"
              />
            </svg>
          </div>
          <span className="font-semibold text-gray-900 hidden sm:block">
            Meridian GIS
          </span>
        </div>

        {/* Divider */}
        <div className="w-px h-8 bg-gray-200" />

        {/* Toolbar items */}
        <div className="flex items-center gap-1 flex-1 overflow-x-auto">
          {items.map((item) => (
            <ToolbarButton
              key={item.id}
              item={item}
              size={size}
              onClick={() => onItemClick?.(item)}
            />
          ))}
        </div>
      </div>
    </div>
  );
};

/**
 * Individual toolbar button
 */
interface ToolbarButtonProps {
  item: ToolbarItem;
  size: 'small' | 'medium' | 'large';
  onClick: () => void;
}

const ToolbarButton: React.FC<ToolbarButtonProps> = ({
  item,
  size,
  onClick,
}) => {
  const sizeClasses = {
    small: 'w-8 h-8',
    medium: 'w-10 h-10',
    large: 'w-12 h-12',
  };

  const handleClick = () => {
    if (!item.disabled) {
      item.onClick?.();
      onClick();
    }
  };

  return (
    <button
      onClick={handleClick}
      disabled={item.disabled}
      className={`${sizeClasses[size]} flex items-center justify-center rounded-lg transition-colors ${
        item.active
          ? 'bg-blue-100 text-blue-700'
          : 'text-gray-700 hover:bg-gray-100'
      } ${item.disabled ? 'opacity-50 cursor-not-allowed' : ''}`}
      aria-label={item.label}
      title={item.tooltip || item.label}
      aria-pressed={item.active}
    >
      {item.icon || (
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
            d="M12 6v6m0 0v6m0-6h6m-6 0H6"
          />
        </svg>
      )}
    </button>
  );
};

export default Toolbar;
