/**
 * Utility functions for formatting data in accessibility reports
 */

/**
 * Format a date to a readable string
 */
export function formatDate(date: Date): string {
  return date.toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  });
}

/**
 * Format a date to a short string
 */
export function formatDateShort(date: Date): string {
  return date.toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
}

/**
 * Format a date and time
 */
export function formatDateTime(date: Date): string {
  return date.toLocaleString('en-US', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

/**
 * Format a number as percentage
 */
export function formatPercentage(value: number, decimals: number = 1): string {
  return `${value.toFixed(decimals)}%`;
}

/**
 * Format a number with commas
 */
export function formatNumber(value: number): string {
  return value.toLocaleString('en-US');
}

/**
 * Format file size in bytes to human-readable format
 */
export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 Bytes';

  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
}

/**
 * Get color for severity level
 */
export function getSeverityColor(severity: string): string {
  const colors: Record<string, string> = {
    critical: '#dc3545',
    serious: '#fd7e14',
    moderate: '#ffc107',
    minor: '#28a745',
  };
  return colors[severity.toLowerCase()] || '#666666';
}

/**
 * Get color for WCAG level
 */
export function getWCAGLevelColor(level: string): string {
  const colors: Record<string, string> = {
    A: '#28a745',
    AA: '#0066cc',
    AAA: '#9c27b0',
  };
  return colors[level.toUpperCase()] || '#666666';
}

/**
 * Get status color
 */
export function getStatusColor(status: string): string {
  const colors: Record<string, string> = {
    open: '#dc3545',
    'in-progress': '#ffc107',
    resolved: '#28a745',
    'wont-fix': '#999999',
  };
  return colors[status.toLowerCase()] || '#666666';
}

/**
 * Format WCAG criteria as readable string
 */
export function formatWCAGCriteria(criteria: string[]): string {
  return criteria.map((c) => c.replace(/_/g, ' ')).join(', ');
}

/**
 * Calculate compliance score based on issues
 */
export function calculateComplianceScore(
  totalCriteria: number,
  passedCriteria: number
): number {
  if (totalCriteria === 0) return 0;
  return Math.round((passedCriteria / totalCriteria) * 100);
}

/**
 * Get effort estimate in hours
 */
export function getEffortEstimate(effort: string): string {
  const estimates: Record<string, string> = {
    low: '1-2 hours',
    medium: '1-3 days',
    high: '1-2 weeks',
  };
  return estimates[effort.toLowerCase()] || 'Unknown';
}

/**
 * Truncate text to specified length
 */
export function truncateText(text: string, maxLength: number): string {
  if (text.length <= maxLength) return text;
  return `${text.substring(0, maxLength - 3)}...`;
}

/**
 * Capitalize first letter of string
 */
export function capitalize(text: string): string {
  return text.charAt(0).toUpperCase() + text.slice(1);
}

/**
 * Format array as list
 */
export function formatList(items: string[], conjunction: string = 'and'): string {
  if (items.length === 0) return '';
  if (items.length === 1) return items[0];
  if (items.length === 2) return `${items[0]} ${conjunction} ${items[1]}`;

  return `${items.slice(0, -1).join(', ')}, ${conjunction} ${items[items.length - 1]}`;
}

/**
 * Generate unique ID
 */
export function generateId(prefix: string = 'id'): string {
  return `${prefix}-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
}

/**
 * Safe division to avoid divide by zero
 */
export function safeDivide(numerator: number, denominator: number): number {
  return denominator === 0 ? 0 : numerator / denominator;
}

/**
 * Calculate percentage change between two values
 */
export function calculatePercentageChange(oldValue: number, newValue: number): number {
  if (oldValue === 0) return newValue === 0 ? 0 : 100;
  return ((newValue - oldValue) / oldValue) * 100;
}

/**
 * Format trend direction
 */
export function formatTrendDirection(change: number): {
  direction: 'up' | 'down' | 'stable';
  icon: string;
  color: string;
} {
  if (Math.abs(change) < 0.1) {
    return { direction: 'stable', icon: '→', color: '#666666' };
  }
  if (change > 0) {
    return { direction: 'up', icon: '↑', color: '#28a745' };
  }
  return { direction: 'down', icon: '↓', color: '#dc3545' };
}

/**
 * Convert hex color to RGB
 */
export function hexToRgb(hex: string): { r: number; g: number; b: number } | null {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  return result
    ? {
        r: parseInt(result[1], 16),
        g: parseInt(result[2], 16),
        b: parseInt(result[3], 16),
      }
    : null;
}

/**
 * Check if color meets WCAG contrast requirements
 */
export function checkContrastRatio(
  foreground: string,
  background: string
): {
  ratio: number;
  passAA: boolean;
  passAAA: boolean;
} {
  const fg = hexToRgb(foreground);
  const bg = hexToRgb(background);

  if (!fg || !bg) {
    return { ratio: 0, passAA: false, passAAA: false };
  }

  const ratio = calculateLuminanceRatio(fg, bg);

  return {
    ratio,
    passAA: ratio >= 4.5,
    passAAA: ratio >= 7,
  };
}

/**
 * Calculate luminance ratio for contrast
 */
function calculateLuminanceRatio(
  color1: { r: number; g: number; b: number },
  color2: { r: number; g: number; b: number }
): number {
  const l1 = getRelativeLuminance(color1);
  const l2 = getRelativeLuminance(color2);

  const lighter = Math.max(l1, l2);
  const darker = Math.min(l1, l2);

  return (lighter + 0.05) / (darker + 0.05);
}

/**
 * Get relative luminance of a color
 */
function getRelativeLuminance(color: { r: number; g: number; b: number }): number {
  const rsRGB = color.r / 255;
  const gsRGB = color.g / 255;
  const bsRGB = color.b / 255;

  const r = rsRGB <= 0.03928 ? rsRGB / 12.92 : Math.pow((rsRGB + 0.055) / 1.055, 2.4);
  const g = gsRGB <= 0.03928 ? gsRGB / 12.92 : Math.pow((gsRGB + 0.055) / 1.055, 2.4);
  const b = bsRGB <= 0.03928 ? bsRGB / 12.92 : Math.pow((bsRGB + 0.055) / 1.055, 2.4);

  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

/**
 * Sort issues by priority
 */
export function sortIssuesByPriority<T extends { severity: string; remediation: { priority: number } }>(
  issues: T[]
): T[] {
  const severityOrder: Record<string, number> = {
    critical: 0,
    serious: 1,
    moderate: 2,
    minor: 3,
  };

  return [...issues].sort((a, b) => {
    const severityDiff = severityOrder[a.severity] - severityOrder[b.severity];
    if (severityDiff !== 0) return severityDiff;
    return b.remediation.priority - a.remediation.priority;
  });
}

/**
 * Group array items by key
 */
export function groupBy<T>(array: T[], key: keyof T): Record<string, T[]> {
  return array.reduce((result, item) => {
    const groupKey = String(item[key]);
    if (!result[groupKey]) {
      result[groupKey] = [];
    }
    result[groupKey].push(item);
    return result;
  }, {} as Record<string, T[]>);
}

/**
 * Debounce function calls
 */
export function debounce<T extends (...args: any[]) => any>(
  func: T,
  wait: number
): (...args: Parameters<T>) => void {
  let timeout: NodeJS.Timeout | null = null;

  return (...args: Parameters<T>) => {
    if (timeout) clearTimeout(timeout);
    timeout = setTimeout(() => func(...args), wait);
  };
}

/**
 * Throttle function calls
 */
export function throttle<T extends (...args: any[]) => any>(
  func: T,
  limit: number
): (...args: Parameters<T>) => void {
  let inThrottle: boolean = false;

  return (...args: Parameters<T>) => {
    if (!inThrottle) {
      func(...args);
      inThrottle = true;
      setTimeout(() => (inThrottle = false), limit);
    }
  };
}
