# Meridian Accessibility Dashboard

Enterprise WCAG Compliance Monitoring and Reporting Dashboard

## Overview

A production-ready, enterprise-grade accessibility dashboard built with React, TypeScript, and Chart.js. This dashboard provides comprehensive WCAG 2.1 compliance monitoring, issue tracking, and trend analysis.

## Features

- **Real-time Compliance Monitoring**: Track WCAG 2.1 Level A, AA, and AAA compliance
- **Interactive Data Visualization**: Charts, gauges, heatmaps, and trend analysis
- **Issue Management**: Filter, sort, and manage accessibility issues by severity and category
- **Page Ranking**: Identify pages with the most accessibility issues
- **Trend Analysis**: Historical compliance trends with interactive line charts and heatmaps
- **Accessible Design**: All components are WCAG 2.1 AA compliant
- **Type-Safe**: Full TypeScript coverage with comprehensive type definitions

## Architecture

### Components

#### Dashboard Components
- `AccessibilityDashboard.tsx` (273 lines) - Main dashboard container with tabs and filters
- `ComplianceOverview.tsx` (177 lines) - Overall compliance metrics and KPIs
- `IssueBreakdown.tsx` (169 lines) - Detailed issue analysis by category/severity
- `TrendAnalysis.tsx` (231 lines) - Historical trends and patterns

#### Chart Components
- `ComplianceGauge.tsx` (114 lines) - Circular gauge for compliance percentage
- `IssueDistribution.tsx` (169 lines) - Pie/donut charts for issue breakdown
- `TrendLineChart.tsx` (241 lines) - Multi-axis line chart for trends
- `HeatmapCalendar.tsx` (182 lines) - GitHub-style activity heatmap

#### Widget Components
- `ScoreCard.tsx` (106 lines) - KPI metric cards with trends
- `IssueList.tsx` (244 lines) - Paginated, sortable issue table
- `PageRanking.tsx` (167 lines) - Pages ranked by issue count

#### Filter Components
- `WCAGLevelFilter.tsx` (99 lines) - Filter by WCAG conformance level
- `SeverityFilter.tsx` (128 lines) - Filter by issue severity
- `DateRangeFilter.tsx` (169 lines) - Date range picker with presets

### Core Infrastructure

- `types/index.ts` (152 lines) - Comprehensive TypeScript type definitions
- `context/DashboardContext.tsx` (135 lines) - Zustand-based state management
- `hooks/useCompliance.ts` (273 lines) - Compliance data and calculations
- `hooks/useIssues.ts` (214 lines) - Issue filtering and management
- `utils/calculations.ts` (288 lines) - Score calculation utilities

## Installation

```bash
npm install
```

## Usage

```typescript
import { AccessibilityDashboard } from '@meridian/accessibility-dashboard';
import type { AccessibilityIssue } from '@meridian/accessibility-dashboard';

const issues: AccessibilityIssue[] = [
  // Your accessibility issues
];

function App() {
  return (
    <AccessibilityDashboard
      issues={issues}
      onIssueClick={(issue) => console.log(issue)}
      onRefresh={async () => {
        // Fetch fresh data
      }}
    />
  );
}
```

## Type Definitions

### Core Types

- `WCAGLevel`: 'A' | 'AA' | 'AAA'
- `IssueSeverity`: 'critical' | 'serious' | 'moderate' | 'minor'
- `IssueCategory`: 'perceivable' | 'operable' | 'understandable' | 'robust'
- `IssueStatus`: 'open' | 'in-progress' | 'resolved' | 'wont-fix'

### Data Structures

- `AccessibilityIssue`: Complete issue with WCAG criterion, severity, and metadata
- `ComplianceScore`: Overall compliance metrics
- `PageCompliance`: Per-page compliance data
- `TrendDataPoint`: Historical trend data point
- `HeatmapDataPoint`: Calendar heatmap data

## Accessibility

All components are designed to be WCAG 2.1 AA compliant:

- Semantic HTML with proper ARIA labels
- Keyboard navigation support
- Screen reader compatible
- Sufficient color contrast ratios
- Focus indicators
- Responsive and touch-friendly

## File Structure

```
accessibility-dashboard/ts/
├── package.json (52 lines)
├── tsconfig.json (31 lines)
├── src/
│   ├── index.ts (56 lines)
│   ├── types/
│   │   └── index.ts (152 lines)
│   ├── utils/
│   │   └── calculations.ts (288 lines)
│   ├── context/
│   │   └── DashboardContext.tsx (135 lines)
│   ├── hooks/
│   │   ├── useCompliance.ts (273 lines)
│   │   └── useIssues.ts (214 lines)
│   ├── components/
│   │   ├── Dashboard/
│   │   │   ├── AccessibilityDashboard.tsx (273 lines)
│   │   │   ├── ComplianceOverview.tsx (177 lines)
│   │   │   ├── IssueBreakdown.tsx (169 lines)
│   │   │   └── TrendAnalysis.tsx (231 lines)
│   │   ├── Charts/
│   │   │   ├── ComplianceGauge.tsx (114 lines)
│   │   │   ├── IssueDistribution.tsx (169 lines)
│   │   │   ├── TrendLineChart.tsx (241 lines)
│   │   │   └── HeatmapCalendar.tsx (182 lines)
│   │   ├── Widgets/
│   │   │   ├── ScoreCard.tsx (106 lines)
│   │   │   ├── IssueList.tsx (244 lines)
│   │   │   └── PageRanking.tsx (167 lines)
│   │   └── Filters/
│   │       ├── WCAGLevelFilter.tsx (99 lines)
│   │       ├── SeverityFilter.tsx (128 lines)
│   │       └── DateRangeFilter.tsx (169 lines)
```

**Total: 3,670 lines of production-ready code**

## Dependencies

### Core
- React 18.2+
- TypeScript 5.3+
- Chart.js 4.4+
- react-chartjs-2 5.2+

### Utilities
- date-fns 2.30+
- clsx 2.0+
- zustand 4.4+

### Development
- TailwindCSS 3.3+
- ESLint with accessibility plugins
- TypeScript strict mode

## License

MIT

## Author

HarborGrid - Meridian GIS Platform Team
