# Enterprise Visualization Engine

Advanced visualization library for the $983M Enterprise SaaS Platform v0.5. Provides comprehensive 2D and 3D data visualization capabilities with D3.js, Three.js, Deck.gl, and Recharts.

## Features

### 2D Charts
- **BarChart** - Animated vertical/horizontal bars with grouping and stacking
- **LineChart** - Multi-series line charts with areas and custom curves
- **PieChart** - Interactive pie and donut charts with animations
- **ScatterPlot** - Scatter plots with linear/polynomial/exponential regression
- **HeatMap** - Color-coded heat maps with legends
- **TreeMap** - Hierarchical tree maps with multiple tiling algorithms
- **SankeyDiagram** - Flow diagrams for process visualization
- **NetworkGraph** - Force-directed network graphs with physics simulation

### 3D Visualizations
- **Scene3D** - Three.js scene wrapper with camera and lighting
- **DataVisualization3D** - 3D data visualization (bars, scatter, surface, network)
- **GlobeVisualization** - Interactive 3D globe with geographic data points

### Animation System
- **AnimationEngine** - D3 transition-based animation system
- **Interpolators** - Custom interpolation functions for complex animations
- Advanced easing functions (spring, elastic, bounce, etc.)
- Path morphing and custom animation sequences

### Interaction
- **ZoomPan** - Zoom and pan controls for visualizations
- **Tooltip** - Rich, context-aware tooltips with custom content
- Interactive event handling (click, hover, drag)

### Theming
- Multiple built-in themes (light, dark, high-contrast, pastel, corporate)
- Custom theme support
- Comprehensive color palettes
- Typography and spacing system

## Installation

```bash
npm install @enterprise-saas/visualization
```

## Quick Start

### Bar Chart Example

```typescript
import { BarChart } from '@enterprise-saas/visualization';

const data = [
  { label: 'Q1', value: 100 },
  { label: 'Q2', value: 150 },
  { label: 'Q3', value: 200 },
  { label: 'Q4', value: 175 },
];

const config = {
  dimensions: { width: 800, height: 400 },
  orientation: 'vertical',
  theme: { colorScheme: ['#3b82f6'] },
  animation: { duration: 750, enabled: true },
};

<BarChart data={data} config={config} />
```

### Line Chart Example

```typescript
import { LineChart } from '@enterprise-saas/visualization';

const seriesData = {
  'Revenue': [
    { timestamp: '2024-01', value: 1000 },
    { timestamp: '2024-02', value: 1500 },
    { timestamp: '2024-03', value: 1800 },
  ],
  'Expenses': [
    { timestamp: '2024-01', value: 800 },
    { timestamp: '2024-02', value: 900 },
    { timestamp: '2024-03', value: 1000 },
  ],
};

<LineChart
  data={[]}
  config={config}
  multiSeries={true}
  seriesData={seriesData}
/>
```

### 3D Visualization Example

```typescript
import { DataVisualization3D } from '@enterprise-saas/visualization';

const data3D = [
  { x: 0, y: 5, z: 0, value: 100 },
  { x: 2, y: 8, z: 2, value: 150 },
  { x: -2, y: 6, z: -2, value: 120 },
];

const config3D = {
  dataType: 'bars',
  heightScale: 0.5,
  camera: { position: { x: 10, y: 10, z: 10 } },
};

<DataVisualization3D data={data3D} config={config3D} />
```

### Globe Visualization Example

```typescript
import { GlobeVisualization } from '@enterprise-saas/visualization';

const geoData = [
  { lat: 40.7128, lng: -74.0060, value: 100, label: 'New York' },
  { lat: 51.5074, lng: -0.1278, value: 85, label: 'London' },
  { lat: 35.6762, lng: 139.6503, value: 120, label: 'Tokyo' },
];

<GlobeVisualization data={geoData} config={{ radius: 5 }} />
```

## Animation

```typescript
import { animationEngine, Easing } from '@enterprise-saas/visualization';

// Fade in animation
animationEngine.fadeIn(selection, {
  duration: 500,
  easing: Easing.cubicOut,
});

// Custom path animation
animationEngine.animatePath(pathSelection, {
  duration: 1000,
  easing: Easing.linear,
});

// Spring interpolator
import { springInterpolator } from '@enterprise-saas/visualization';
const spring = springInterpolator(100, 10, 1);
```

## Interaction

```typescript
import { zoomPan, useTooltip } from '@enterprise-saas/visualization';

// Initialize zoom/pan
zoomPan.initialize(svgElement, targetGroup, {
  minZoom: 0.5,
  maxZoom: 5,
  enableZoom: true,
  enablePan: true,
});

// Use tooltip hook
const { show, hide, TooltipComponent } = useTooltip({
  position: 'mouse',
  offset: { x: 10, y: 10 },
});
```

## Theming

```typescript
import { themeManager, themes } from '@enterprise-saas/visualization';

// Set theme
themeManager.setTheme('dark');

// Create custom theme
const customTheme = themeManager.createCustomTheme(
  'myTheme',
  themes.light,
  {
    colorScheme: ['#ff0000', '#00ff00', '#0000ff'],
    backgroundColor: '#f5f5f5',
  }
);
```

## TypeScript Support

This package is written in TypeScript and includes comprehensive type definitions.

```typescript
import type {
  DataPoint,
  ChartConfig,
  ThemeConfig,
  AnimationConfig,
} from '@enterprise-saas/visualization';
```

## API Reference

### Chart Components

All chart components accept the following base props:

- `data` - Data array or object specific to chart type
- `config` - Chart configuration object
- `className` - Optional CSS class name
- `style` - Optional inline styles
- `onEvent` - Optional event handler

### Configuration Objects

#### ChartConfig
```typescript
interface ChartConfig {
  dimensions: Dimensions;
  theme?: ThemeConfig;
  animation?: AnimationConfig;
  interaction?: InteractionConfig;
  accessibility?: AccessibilityConfig;
}
```

#### ThemeConfig
```typescript
interface ThemeConfig {
  colorScheme?: string[];
  backgroundColor?: string;
  textColor?: string;
  gridColor?: string;
  fontFamily?: string;
  fontSize?: number;
}
```

#### AnimationConfig
```typescript
interface AnimationConfig {
  duration?: number;
  delay?: number;
  easing?: string | ((t: number) => number);
  enabled?: boolean;
}
```

## Performance

- WebGL acceleration for 3D visualizations
- Optimized D3 transitions
- Efficient data binding and updates
- Support for large datasets (100K+ data points)
- Decimation algorithms for performance optimization

## Accessibility

- ARIA labels and descriptions
- Keyboard navigation support
- Screen reader optimization
- High contrast theme
- Semantic SVG structure

## Browser Support

- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+

## License

Proprietary - Enterprise SaaS Platform

## Version

0.5.0

---

Built with enterprise-grade visualization for mission-critical applications.
