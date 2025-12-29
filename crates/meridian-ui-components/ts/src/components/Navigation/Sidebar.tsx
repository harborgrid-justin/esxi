/**
 * Sidebar Component
 * Collapsible sidebar for tools and panels
 * @module @meridian/ui-components/Navigation
 */

import React, { useState } from 'react';
import type { BaseComponentProps } from '../../types';

export interface SidebarProps extends BaseComponentProps {
  /** Sidebar position */
  position?: 'left' | 'right';
  /** Initial width */
  width?: number;
  /** Initial collapsed state */
  collapsed?: boolean;
  /** Enable resizing */
  resizable?: boolean;
  /** Callback when collapse state changes */
  onCollapse?: (collapsed: boolean) => void;
  /** Children content */
  children?: React.ReactNode;
}

/**
 * Collapsible sidebar component
 */
export const Sidebar: React.FC<SidebarProps> = ({
  position = 'left',
  width: initialWidth = 320,
  collapsed: initialCollapsed = false,
  resizable = true,
  onCollapse,
  className = '',
  children,
  ...props
}) => {
  const [collapsed, setCollapsed] = useState(initialCollapsed);
  const [width, setWidth] = useState(initialWidth);
  const [isResizing, setIsResizing] = useState(false);

  const handleToggleCollapse = () => {
    const newCollapsed = !collapsed;
    setCollapsed(newCollapsed);
    onCollapse?.(newCollapsed);
  };

  const handleMouseDown = (e: React.MouseEvent) => {
    if (!resizable) return;
    e.preventDefault();
    setIsResizing(true);

    const startX = e.pageX;
    const startWidth = width;

    const handleMouseMove = (e: MouseEvent) => {
      const delta = position === 'left' ? e.pageX - startX : startX - e.pageX;
      const newWidth = Math.max(200, Math.min(600, startWidth + delta));
      setWidth(newWidth);
    };

    const handleMouseUp = () => {
      setIsResizing(false);
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  };

  return (
    <div
      className={`meridian-sidebar relative bg-white shadow-lg transition-all duration-300 ${
        collapsed ? 'w-12' : ''
      } ${isResizing ? 'select-none' : ''} ${className}`}
      style={{
        width: collapsed ? '48px' : `${width}px`,
        [position]: 0,
      }}
      {...props}
    >
      {/* Collapse button */}
      <button
        onClick={handleToggleCollapse}
        className={`absolute top-4 ${
          position === 'left' ? 'right-4' : 'left-4'
        } w-8 h-8 flex items-center justify-center bg-white border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors z-10`}
        aria-label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
        aria-expanded={!collapsed}
      >
        <svg
          className={`w-5 h-5 transition-transform ${
            (position === 'left' && !collapsed) || (position === 'right' && collapsed)
              ? 'rotate-0'
              : 'rotate-180'
          }`}
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M15 19l-7-7 7-7"
          />
        </svg>
      </button>

      {/* Content */}
      {!collapsed && (
        <div className="h-full overflow-y-auto p-4 pt-16">{children}</div>
      )}

      {/* Resize handle */}
      {resizable && !collapsed && (
        <div
          className={`absolute top-0 ${
            position === 'left' ? 'right-0' : 'left-0'
          } w-1 h-full cursor-col-resize hover:bg-blue-500 transition-colors ${
            isResizing ? 'bg-blue-500' : ''
          }`}
          onMouseDown={handleMouseDown}
          role="separator"
          aria-label="Resize sidebar"
        />
      )}
    </div>
  );
};

export default Sidebar;
