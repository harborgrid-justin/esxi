# Enterprise Dashboard UI

**Production-ready dashboard for $983M Enterprise SaaS Platform**

A comprehensive, enterprise-grade dashboard UI package built with React, TypeScript, and modern web technologies. Features real-time data updates, advanced analytics visualizations, and a flexible widget system.

## Features

- 🎯 **Real-time Data** - WebSocket-based live updates for KPIs, alerts, and metrics
- 📊 **Advanced Charts** - Revenue, Usage, Performance, and Geographic visualizations
- 🎨 **Customizable Layout** - Drag-and-drop widget grid with save/restore
- 🚨 **Alert Management** - Real-time alert monitoring with severity levels
- 📈 **KPI Cards** - Beautiful metric displays with trends and sparklines
- 🔍 **Activity Tracking** - Comprehensive audit logs and activity feeds
- 📊 **Quota Management** - Resource usage tracking with forecasting
- 🎭 **Dark Mode** - Optimized for low-light environments
- ⚡ **Performance** - Optimized rendering with React best practices
- 🔒 **Enterprise Ready** - Built for scale and reliability

## Installation

```bash
npm install @esxi/enterprise-dashboard
```

### Peer Dependencies

```bash
npm install react@^18.2.0 react-dom@^18.2.0
```

## Quick Start

```tsx
import { ExecutiveDashboard } from '@esxi/enterprise-dashboard';

function App() {
  return (
    <ExecutiveDashboard
      layoutId="default-layout"
      defaultTimeRange="24h"
      enableRealTime={true}
    />
  );
}
```

## Components

### Executive Dashboard

Main dashboard component with comprehensive monitoring capabilities.

```tsx
import { ExecutiveDashboard } from '@esxi/enterprise-dashboard';

<ExecutiveDashboard
  layoutId="exec-dashboard"
  defaultTimeRange="24h"
  enableRealTime={true}
/>
```

### KPI Cards

Display key performance indicators with trends.

```tsx
import { KPICard } from '@esxi/enterprise-dashboard';

const metric = {
  id: 'revenue',
  label: 'Monthly Revenue',
  value: 1250000,
  unit: 'USD',
  format: 'currency',
  trend: 'up',
  trendValue: 12.5,
  sparklineData: [100, 120, 115, 130, 125],
  status: 'healthy',
};

<KPICard metric={metric} showSparkline={true} />
```

### Charts

#### Revenue Chart

```tsx
import { RevenueChart } from '@esxi/enterprise-dashboard';

<RevenueChart
  data={revenueData}
  type="composed"
  showForecast={true}
  showProfit={true}
  height={400}
/>
```

#### Usage Chart

```tsx
import { UsageChart } from '@esxi/enterprise-dashboard';

<UsageChart
  data={usageMetrics}
  metrics={['activeUsers', 'apiCalls', 'requestLatency']}
  type="area"
  height={350}
/>
```

#### Performance Chart

```tsx
import { PerformanceChart } from '@esxi/enterprise-dashboard';

<PerformanceChart
  data={performanceMetrics}
  type="timeline"
  height={400}
/>
```

#### Geographic Chart

```tsx
import { GeoChart } from '@esxi/enterprise-dashboard';

<GeoChart
  data={geoData}
  metric="users"
  height={500}
  interactive={true}
/>
```

### Widgets

#### Alert Widget

```tsx
import { AlertWidget } from '@esxi/enterprise-dashboard';

<AlertWidget
  alerts={alerts}
  maxDisplay={10}
  onAcknowledge={(id) => console.log('Acknowledged:', id)}
  onResolve={(id) => console.log('Resolved:', id)}
/>
```

#### Activity Widget

```tsx
import { ActivityWidget } from '@esxi/enterprise-dashboard';

<ActivityWidget
  activities={activities}
  maxDisplay={20}
  showFilters={true}
/>
```

#### Quota Widget

```tsx
import { QuotaWidget } from '@esxi/enterprise-dashboard';

<QuotaWidget
  quotas={quotas}
  warningThreshold={75}
  criticalThreshold={90}
/>
```

### Dashboard Grid

Draggable and resizable widget layout.

```tsx
import { DashboardGrid } from '@esxi/enterprise-dashboard';

<DashboardGrid
  widgets={widgets}
  onLayoutChange={(updated) => saveLayout(updated)}
  editable={true}
  renderWidget={(widget) => <YourWidget {...widget} />}
/>
```

## Hooks

### useDashboard

Main hook for dashboard state management.

```tsx
import { useDashboard } from '@esxi/enterprise-dashboard';

function MyDashboard() {
  const {
    layout,
    kpis,
    alerts,
    activities,
    quotas,
    isLoading,
    refresh,
    updateTimeRange,
  } = useDashboard({
    layoutId: 'my-dashboard',
    autoLoad: true,
    autoRefresh: true,
  });

  return (
    <div>
      {/* Your dashboard UI */}
    </div>
  );
}
```

### useRealTimeData

WebSocket-based real-time data subscription.

```tsx
import { useRealTimeData } from '@esxi/enterprise-dashboard';

function LiveMetrics() {
  const { connected, lastMessage, messageCount } = useRealTimeData({
    channels: ['kpis', 'alerts', 'metrics'],
    enabled: true,
    onMessage: (data) => {
      console.log('Real-time update:', data);
    },
  });

  return (
    <div>
      Status: {connected ? 'Connected' : 'Disconnected'}
      Messages: {messageCount}
    </div>
  );
}
```

## State Management

Uses Zustand for efficient state management.

```tsx
import { useDashboardStore } from '@esxi/enterprise-dashboard';

function MyComponent() {
  const kpis = useDashboardStore((state) => state.kpis);
  const setKPIs = useDashboardStore((state) => state.setKPIs);

  // Use state...
}
```

## API Service

Built-in API service for data fetching.

```tsx
import { dashboardService } from '@esxi/enterprise-dashboard';

// Fetch KPIs
const response = await dashboardService.getKPIs('24h', filters);

// Fetch alerts
const alerts = await dashboardService.getAlerts(pagination, filters);

// Export dashboard
const exportUrl = await dashboardService.exportDashboard({
  format: 'pdf',
  includeCharts: true,
});
```

## TypeScript Support

Fully typed with comprehensive TypeScript definitions.

```tsx
import type {
  KPIMetric,
  Alert,
  DashboardWidget,
  TimeRange,
} from '@esxi/enterprise-dashboard';

const metric: KPIMetric = {
  id: 'users',
  label: 'Active Users',
  value: 15000,
  trend: 'up',
  trendValue: 8.2,
};
```

## Styling

The package uses Tailwind CSS for styling. Make sure Tailwind is configured in your project:

```js
// tailwind.config.js
module.exports = {
  content: [
    './src/**/*.{js,jsx,ts,tsx}',
    './node_modules/@esxi/enterprise-dashboard/**/*.{js,jsx,ts,tsx}',
  ],
  theme: {
    extend: {},
  },
  plugins: [],
};
```

## Configuration

Default configuration can be customized:

```tsx
import { initializeDashboard } from '@esxi/enterprise-dashboard';

const config = initializeDashboard({
  timeRange: '7d',
  refreshInterval: 60000, // 1 minute
  autoRefresh: true,
  theme: 'dark',
  animations: true,
  realTimeEnabled: true,
});
```

## Performance

- Optimized re-renders with React.memo and useMemo
- Efficient state updates with Zustand
- Virtual scrolling for large datasets
- Lazy loading for charts and heavy components
- WebSocket connection pooling
- Request debouncing and caching

## Browser Support

- Chrome/Edge (latest 2 versions)
- Firefox (latest 2 versions)
- Safari (latest 2 versions)

## License

PROPRIETARY - Enterprise SaaS Platform

## Support

For enterprise support, contact: support@esxi-platform.com
