/**
 * Filter Panel Component
 * @module @harborgrid/enterprise-analytics/components/dashboard
 */

import type { DashboardFilter } from '../../types';

export interface FilterPanelProps {
  filters: DashboardFilter[];
  values: Record<string, unknown>;
  onChange: (values: Record<string, unknown>) => void;
}
