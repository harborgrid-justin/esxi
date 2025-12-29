/**
 * Severity levels for accessibility issues
 * @module constants/severityLevels
 */

import type { SeverityLevel } from '../types';

/**
 * Severity level definitions
 */
export const SEVERITY_LEVELS = {
  CRITICAL: 'critical',
  SERIOUS: 'serious',
  MODERATE: 'moderate',
  MINOR: 'minor',
  INFO: 'info',
} as const satisfies Record<string, SeverityLevel>;

/**
 * Severity level priorities (higher number = more severe)
 */
export const SEVERITY_PRIORITY: Record<SeverityLevel, number> = {
  critical: 5,
  serious: 4,
  moderate: 3,
  minor: 2,
  info: 1,
};

/**
 * Severity level descriptions
 */
export const SEVERITY_DESCRIPTIONS: Record<SeverityLevel, string> = {
  critical: 'Critical accessibility barrier that prevents access for many users',
  serious: 'Serious accessibility issue that significantly impacts user experience',
  moderate: 'Moderate accessibility issue that affects some users',
  minor: 'Minor accessibility improvement that would enhance user experience',
  info: 'Informational note about accessibility best practices',
};

/**
 * Severity level colors for UI display
 */
export const SEVERITY_COLORS: Record<SeverityLevel, { bg: string; text: string; border: string }> = {
  critical: {
    bg: '#FEE2E2',
    text: '#991B1B',
    border: '#DC2626',
  },
  serious: {
    bg: '#FED7AA',
    text: '#9A3412',
    border: '#EA580C',
  },
  moderate: {
    bg: '#FEF3C7',
    text: '#92400E',
    border: '#F59E0B',
  },
  minor: {
    bg: '#DBEAFE',
    text: '#1E40AF',
    border: '#3B82F6',
  },
  info: {
    bg: '#E0E7FF',
    text: '#3730A3',
    border: '#6366F1',
  },
};

/**
 * Compare two severity levels
 * @returns Positive if a > b, negative if a < b, 0 if equal
 */
export function compareSeverity(a: SeverityLevel, b: SeverityLevel): number {
  return SEVERITY_PRIORITY[a] - SEVERITY_PRIORITY[b];
}

/**
 * Check if severity level is at least as severe as threshold
 */
export function isSeverityAtLeast(level: SeverityLevel, threshold: SeverityLevel): boolean {
  return SEVERITY_PRIORITY[level] >= SEVERITY_PRIORITY[threshold];
}

/**
 * Get severity level from priority number
 */
export function getSeverityFromPriority(priority: number): SeverityLevel {
  const entry = Object.entries(SEVERITY_PRIORITY).find(([, p]) => p === priority);
  return (entry?.[0] as SeverityLevel) ?? 'info';
}

/**
 * Sort severity levels from most to least severe
 */
export function sortSeverities(severities: SeverityLevel[]): SeverityLevel[] {
  return [...severities].sort((a, b) => SEVERITY_PRIORITY[b] - SEVERITY_PRIORITY[a]);
}

/**
 * Get severity level icon
 */
export function getSeverityIcon(level: SeverityLevel): string {
  const icons: Record<SeverityLevel, string> = {
    critical: '🔴',
    serious: '🟠',
    moderate: '🟡',
    minor: '🔵',
    info: '🟢',
  };
  return icons[level];
}

/**
 * Get severity level label
 */
export function getSeverityLabel(level: SeverityLevel): string {
  const labels: Record<SeverityLevel, string> = {
    critical: 'Critical',
    serious: 'Serious',
    moderate: 'Moderate',
    minor: 'Minor',
    info: 'Info',
  };
  return labels[level];
}
