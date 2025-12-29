/**
 * Meridian Dashboard - Main Entry Point
 *
 * Analytics dashboard and reporting system for Meridian GIS Platform
 */

// Components
export { DashboardGrid } from './components/Dashboard/DashboardGrid';
export { DashboardWidget } from './components/Dashboard/DashboardWidget';
export { DashboardToolbar } from './components/Dashboard/DashboardToolbar';

export { MapWidget } from './components/Widgets/MapWidget';
export { ChartWidget } from './components/Widgets/ChartWidget';
export { TableWidget } from './components/Widgets/TableWidget';
export { KPIWidget } from './components/Widgets/KPIWidget';
export { TimelineWidget } from './components/Widgets/TimelineWidget';
export { FilterWidget } from './components/Widgets/FilterWidget';

export { LineChart } from './components/Charts/LineChart';
export { BarChart } from './components/Charts/BarChart';
export { PieChart } from './components/Charts/PieChart';
export { ScatterChart } from './components/Charts/ScatterChart';
export { HeatmapChart } from './components/Charts/HeatmapChart';

export { ReportBuilder } from './components/Reports/ReportBuilder';
export { ReportViewer } from './components/Reports/ReportViewer';
export { ExportDialog } from './components/Reports/ExportDialog';

// Hooks
export { useDashboard } from './hooks/useDashboard';
export { useWidgets } from './hooks/useWidgets';
export { useDataSource } from './hooks/useDataSource';

// Context
export { DashboardProvider, useDashboardContext } from './context/DashboardContext';

// Types
export * from './types';

// Utils
export * from './utils/layout';
