/**
 * Widget Container Component
 * @module @harborgrid/enterprise-analytics/components/dashboard
 */

import type { Widget } from '../../types';

export interface WidgetContainerProps {
  widget: Widget;
  children?: React.ReactNode;
}
