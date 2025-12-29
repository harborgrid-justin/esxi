/**
 * Layout utilities for grid management
 */

import { Layout, Layouts, WidgetPosition, Widget } from '../types';

/**
 * Convert widget position to grid layout
 */
export const widgetToLayout = (widget: Widget): Layout => {
  return {
    i: widget.id,
    x: widget.position.x,
    y: widget.position.y,
    w: widget.position.w,
    h: widget.position.h,
    minW: widget.position.min_w,
    minH: widget.position.min_h,
    maxW: widget.position.max_w,
    maxH: widget.position.max_h,
  };
};

/**
 * Convert grid layout to widget position
 */
export const layoutToPosition = (layout: Layout): WidgetPosition => {
  return {
    x: layout.x,
    y: layout.y,
    w: layout.w,
    h: layout.h,
    min_w: layout.minW,
    min_h: layout.minH,
    max_w: layout.maxW,
    max_h: layout.maxH,
  };
};

/**
 * Convert array of widgets to layouts for all breakpoints
 */
export const widgetsToLayouts = (widgets: Widget[]): Layouts => {
  const layout = widgets.map(widgetToLayout);

  return {
    lg: layout,
    md: layout,
    sm: layout,
    xs: layout,
    xxs: layout,
  };
};

/**
 * Find available position for new widget
 */
export const findAvailablePosition = (
  existingWidgets: Widget[],
  width: number = 6,
  height: number = 4,
  cols: number = 12
): WidgetPosition => {
  const occupiedCells = new Set<string>();

  // Mark all occupied cells
  existingWidgets.forEach((widget) => {
    const pos = widget.position;
    for (let y = pos.y; y < pos.y + pos.h; y++) {
      for (let x = pos.x; x < pos.x + pos.w; x++) {
        occupiedCells.add(`${x},${y}`);
      }
    }
  });

  // Find first available position
  let y = 0;
  while (true) {
    for (let x = 0; x <= cols - width; x++) {
      let fits = true;

      // Check if widget fits at this position
      for (let dy = 0; dy < height; dy++) {
        for (let dx = 0; dx < width; dx++) {
          if (occupiedCells.has(`${x + dx},${y + dy}`)) {
            fits = false;
            break;
          }
        }
        if (!fits) break;
      }

      if (fits) {
        return {
          x,
          y,
          w: width,
          h: height,
        };
      }
    }
    y++;

    // Safety limit to prevent infinite loop
    if (y > 1000) {
      return { x: 0, y: 0, w: width, h: height };
    }
  }
};

/**
 * Check if two widgets overlap
 */
export const widgetsOverlap = (a: WidgetPosition, b: WidgetPosition): boolean => {
  return !(
    a.x + a.w <= b.x ||
    b.x + b.w <= a.x ||
    a.y + a.h <= b.y ||
    b.y + b.h <= a.y
  );
};

/**
 * Compact layout vertically (remove gaps)
 */
export const compactLayout = (widgets: Widget[]): Widget[] => {
  const sorted = [...widgets].sort((a, b) => {
    if (a.position.y === b.position.y) {
      return a.position.x - b.position.x;
    }
    return a.position.y - b.position.y;
  });

  const compacted = sorted.map((widget) => {
    let newY = 0;

    // Find the highest position this widget can move to
    while (newY < widget.position.y) {
      const testPos = { ...widget.position, y: newY };
      const hasOverlap = sorted.some((other) => {
        if (other.id === widget.id) return false;
        return widgetsOverlap(testPos, other.position);
      });

      if (hasOverlap) {
        newY++;
      } else {
        break;
      }
    }

    return {
      ...widget,
      position: {
        ...widget.position,
        y: newY,
      },
    };
  });

  return compacted;
};

/**
 * Calculate total layout height
 */
export const getLayoutHeight = (widgets: Widget[]): number => {
  if (widgets.length === 0) return 0;

  return Math.max(
    ...widgets.map((w) => w.position.y + w.position.h)
  );
};

/**
 * Validate widget position
 */
export const validatePosition = (
  position: WidgetPosition,
  cols: number = 12
): { valid: boolean; errors: string[] } => {
  const errors: string[] = [];

  if (position.x < 0) {
    errors.push('X position cannot be negative');
  }

  if (position.y < 0) {
    errors.push('Y position cannot be negative');
  }

  if (position.w <= 0) {
    errors.push('Width must be greater than 0');
  }

  if (position.h <= 0) {
    errors.push('Height must be greater than 0');
  }

  if (position.x + position.w > cols) {
    errors.push(`Widget extends beyond grid width (${cols} columns)`);
  }

  if (position.min_w && position.w < position.min_w) {
    errors.push(`Width is less than minimum (${position.min_w})`);
  }

  if (position.min_h && position.h < position.min_h) {
    errors.push(`Height is less than minimum (${position.min_h})`);
  }

  if (position.max_w && position.w > position.max_w) {
    errors.push(`Width exceeds maximum (${position.max_w})`);
  }

  if (position.max_h && position.h > position.max_h) {
    errors.push(`Height exceeds maximum (${position.max_h})`);
  }

  return {
    valid: errors.length === 0,
    errors,
  };
};

/**
 * Snap position to grid
 */
export const snapToGrid = (
  position: WidgetPosition,
  gridSize: number = 1
): WidgetPosition => {
  return {
    ...position,
    x: Math.round(position.x / gridSize) * gridSize,
    y: Math.round(position.y / gridSize) * gridSize,
    w: Math.max(1, Math.round(position.w / gridSize) * gridSize),
    h: Math.max(1, Math.round(position.h / gridSize) * gridSize),
  };
};

/**
 * Get responsive breakpoint based on width
 */
export const getBreakpoint = (width: number): string => {
  if (width >= 1200) return 'lg';
  if (width >= 996) return 'md';
  if (width >= 768) return 'sm';
  if (width >= 480) return 'xs';
  return 'xxs';
};

/**
 * Calculate columns for breakpoint
 */
export const getColumnsForBreakpoint = (breakpoint: string): number => {
  const columnMap: Record<string, number> = {
    lg: 12,
    md: 10,
    sm: 6,
    xs: 4,
    xxs: 2,
  };

  return columnMap[breakpoint] || 12;
};
