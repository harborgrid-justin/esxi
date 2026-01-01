/**
 * Dashboard Components Module
 * @module @harborgrid/enterprise-analytics/components/dashboard
 */

export * from './DashboardBuilder';
export * from './MetricCard';
export * from './DataTable';

// Type-only exports for other components
export type { WidgetContainerProps } from './WidgetContainer';
export type { FilterPanelProps } from './FilterPanel';
export type { DateRangePickerProps } from './DateRangePicker';

// Placeholder exports
export const WidgetContainer = () => null;
export const FilterPanel = () => null;
export const DateRangePicker = () => null;
