/**
 * Date Range Picker Component
 * @module @harborgrid/enterprise-analytics/components/dashboard
 */

export interface DateRangePickerProps {
  start: Date;
  end: Date;
  onChange: (start: Date, end: Date) => void;
  presets?: Array<{ label: string; start: Date; end: Date }>;
}
