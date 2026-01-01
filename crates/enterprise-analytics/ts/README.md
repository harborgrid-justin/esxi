# @harborgrid/enterprise-analytics

Enterprise-grade Analytics & Business Intelligence Dashboard Platform with OLAP, real-time visualization, and advanced query optimization.

## Features

### Core Analytics Engine
- **SQL-like Query Builder**: Intuitive API for building complex analytics queries
- **Query Optimization**: Automatic query optimization with cost-based analysis
- **Async Execution**: Non-blocking query execution with progress tracking
- **Result Caching**: Intelligent caching with TTL and invalidation strategies
- **OLAP Data Cube**: Multi-dimensional analysis with slice, dice, drill-down operations

### Visualization Engine
- **D3.js Integration**: Full D3.js wrapper for advanced visualizations
- **Chart Factory**: Automatic chart type selection based on data
- **Color Scales**: Categorical, sequential, and diverging palettes
- **Legend Generation**: Auto-generated, interactive legends
- **Tooltip Management**: Rich, customizable tooltips

### Chart Components (React)
- **Line Chart**: Time series visualization with smooth curves
- **Bar Chart**: Categorical data with animations
- **Pie/Donut Chart**: Proportional data with interactive slices
- **Heat Map**: Density visualization with color gradients
- **Scatter Plot**: Correlation analysis with trend lines
- **Tree Map**: Hierarchical data visualization
- **Sankey Diagram**: Flow visualization
- **Geographic Map**: Spatial data mapping

### Dashboard Components
- **Dashboard Builder**: Drag-and-drop dashboard editor
- **Widget Container**: Flexible widget layout system
- **Filter Panel**: Global filtering across dashboards
- **Date Range Picker**: Temporal filtering with presets
- **Metric Cards**: KPI display with trends
- **Data Table**: Sortable, filterable data grid with pagination

### Analytics Services
- **Data Connector**: Multi-source data connections (SQL, REST, GraphQL, WebSocket)
- **ETL Pipeline**: Extract, Transform, Load with parallel execution
- **Scheduler Service**: Cron-based report scheduling
- **Export Service**: PDF, Excel, CSV, PNG, SVG export

## Installation

```bash
npm install @harborgrid/enterprise-analytics
```

## Usage

### Building Queries

```typescript
import { createQueryBuilder } from '@harborgrid/enterprise-analytics/query';

const query = createQueryBuilder('my-datasource')
  .addDimension('date', 'Date')
  .sum('revenue', 'Total Revenue')
  .avg('order_value', 'Avg Order Value')
  .where('status', 'eq', 'completed')
  .last30Days()
  .orderByDesc('date')
  .build();
```

### Creating Visualizations

```typescript
import { LineChart } from '@harborgrid/enterprise-analytics/components';

function MyDashboard() {
  return (
    <LineChart
      data={salesData}
      config={{
        xAxis: { field: 'date', label: 'Date', scale: 'time' },
        yAxis: { field: 'revenue', label: 'Revenue', zero: true },
        smooth: true,
        showGrid: true,
        tooltip: { show: true },
      }}
      width={800}
      height={400}
    />
  );
}
```

### OLAP Operations

```typescript
import { createDataCube } from '@harborgrid/enterprise-analytics/query';

const cube = createDataCube({
  id: 'sales-cube',
  name: 'Sales Analysis',
  dataSourceId: 'my-datasource',
  dimensions: [
    { id: 'date', name: 'Date', field: 'date', type: 'date' },
    { id: 'region', name: 'Region', field: 'region', type: 'string' },
    { id: 'product', name: 'Product', field: 'product', type: 'string' },
  ],
  measures: [
    { id: 'revenue', name: 'Revenue', field: 'revenue', aggregation: 'sum' },
    { id: 'quantity', name: 'Quantity', field: 'quantity', aggregation: 'sum' },
  ],
});

await cube.loadData(rawData);

// Slice by region
const westCoastSales = cube.slice('region', 'West Coast');

// Dice by multiple dimensions
const filteredSales = cube.dice({
  region: ['West Coast', 'East Coast'],
  product: ['Widget A', 'Widget B'],
});

// Roll-up to higher level
const monthlySales = cube.rollUp('date', 1);
```

### Data Export

```typescript
import { createExportService } from '@harborgrid/enterprise-analytics/services';

const exportService = createExportService();

// Export to PDF
const result = await exportService.exportDashboard(dashboard, {
  format: 'pdf',
  filename: 'Q4-Report.pdf',
  orientation: 'landscape',
  pageSize: 'A4',
});

exportService.downloadResult(result);
```

### ETL Pipeline

```typescript
import { createETLPipeline, ETLPipeline } from '@harborgrid/enterprise-analytics/services';

const pipeline = createETLPipeline({
  steps: [
    ETLPipeline.createExtractStep('extract', async (data) => {
      return await fetchDataFromAPI();
    }),
    ETLPipeline.createTransformStep('clean', async (data) => {
      return ETLPipeline.filterRows((row) => row.valid)(data);
    }),
    ETLPipeline.createTransformStep('dedupe', async (data) => {
      return ETLPipeline.deduplicate((row) => row.id)(data);
    }),
    ETLPipeline.createLoadStep('load', async (data) => {
      return await saveToDatabase(data);
    }),
  ],
  parallel: false,
  continueOnError: false,
});

const result = await pipeline.execute(initialData);
```

## Architecture

### Query Engine
- `QueryBuilder`: SQL-like query construction
- `QueryOptimizer`: Cost-based optimization
- `QueryExecutor`: Async execution with caching
- `CacheManager`: Multi-level caching (memory + IndexedDB)
- `DataCube`: OLAP operations

### Visualization Engine
- `ChartFactory`: Chart creation and management
- `D3Integration`: D3.js utilities and helpers
- `ColorScale`: Color palette management
- `LegendGenerator`: Auto-legend creation
- `TooltipManager`: Interactive tooltips

### Components
- Chart components (React + D3)
- Dashboard components (drag-drop, filters, widgets)

### Services
- `DataConnector`: Multi-source connectivity
- `ETLPipeline`: Data transformation
- `SchedulerService`: Report scheduling
- `ExportService`: Multi-format export

## TypeScript Support

Full TypeScript support with comprehensive type definitions.

```typescript
import type {
  Query,
  Visualization,
  Dashboard,
  DataCube,
  QueryResult,
} from '@harborgrid/enterprise-analytics/types';
```

## Performance

- Query result caching with configurable TTL
- Connection pooling for database sources
- Lazy loading of chart components
- Virtual scrolling for large datasets
- Optimized D3 rendering with transitions

## Browser Support

- Chrome/Edge 90+
- Firefox 88+
- Safari 14+

## License

MIT

## Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for details.
