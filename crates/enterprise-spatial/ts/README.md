# Enterprise Spatial Analysis Tools

Advanced GIS spatial analysis and geospatial processing library for TypeScript/JavaScript applications.

## Version 0.4.0

## Features

### Core Geometry Operations
- **GeometryFactory** - Create and manipulate geometric objects
- **TopologyEngine** - Topological operations and spatial relationships
- **BufferAnalysis** - Generate buffer zones around geometries
- **OverlayAnalysis** - Union, intersection, difference operations
- **SimplificationEngine** - Douglas-Peucker, Visvalingam-Whyatt algorithms
- **ValidationEngine** - Geometry validation and topology checking

### Spatial Analysis
- **ProximityAnalysis** - Distance calculations, nearest neighbor, K-NN
- **DensityAnalysis** - Kernel density estimation, heat maps
- **ClusterAnalysis** - DBSCAN, K-means, hierarchical clustering
- **NetworkAnalysis** - Dijkstra, A*, routing, TSP
- **TerrainAnalysis** - Slope, aspect, hillshade, flow analysis
- **ViewshedAnalysis** - Line-of-sight, visibility analysis

### Raster Processing
- **RasterCalculator** - Map algebra and raster calculations
- **RasterInterpolation** - IDW, Kriging, Spline interpolation
- **RasterClassification** - Supervised and unsupervised classification
- **RasterMosaic** - Combine multiple rasters, resampling
- **ContourGeneration** - Extract contour lines from DEM

### Projection System
- **ProjectionEngine** - Coordinate transformations (WGS84, UTM, Web Mercator)
- **DatumTransform** - Datum conversions with Helmert transformations
- **ProjectionRegistry** - EPSG database with 100+ projections
- **CustomProjection** - Create custom coordinate systems

### Services
- **GeocodingService** - Address lookup and reverse geocoding
- **TileService** - Vector and raster tile management
- **FeatureService** - CRUD operations for spatial features
- **SpatialIndexService** - R-tree spatial indexing

### React Components
- **SpatialAnalyzer** - Analysis operations UI
- **LayerManager** - Layer control and management
- **FeatureEditor** - Edit geometries and properties
- **QueryBuilder** - Build spatial queries
- **ResultsPanel** - Display analysis results
- **ProjectionPicker** - CRS selection interface
- **MeasurementTool** - Distance and area measurement

## Installation

```bash
npm install @harborgrid/enterprise-spatial
```

## Usage Examples

### Basic Geometry Operations

```typescript
import { GeometryFactory, BufferAnalysis } from '@harborgrid/enterprise-spatial';

// Create a point
const point = GeometryFactory.createPoint(-122.4194, 37.7749);

// Create a buffer
const buffered = BufferAnalysis.buffer(point, {
  distance: 1000,
  units: 'meters',
  steps: 32
});
```

### Spatial Analysis

```typescript
import { ProximityAnalysis, ClusterAnalysis } from '@harborgrid/enterprise-spatial';

// Find nearest features
const nearest = ProximityAnalysis.nearest(geometry, features, {
  limit: 5,
  maxDistance: 10000
});

// Cluster points
const clusters = ClusterAnalysis.cluster(points, {
  algorithm: 'dbscan',
  epsilon: 100,
  minPoints: 3
});
```

### Coordinate Transformations

```typescript
import { projectionEngine } from '@harborgrid/enterprise-spatial';

// Transform coordinates
const transformed = projectionEngine.transform(
  [-122.4194, 37.7749],
  'EPSG:4326',  // WGS84
  'EPSG:3857'   // Web Mercator
);
```

### Raster Processing

```typescript
import { RasterCalculator, ContourGeneration } from '@harborgrid/enterprise-spatial';

// Raster math
const result = RasterCalculator.add(raster1, raster2);

// Generate contours
const contours = ContourGeneration.generateContours(dem, {
  interval: 10,
  smooth: true
});
```

### React Components

```typescript
import { SpatialAnalyzer, LayerManager } from '@harborgrid/enterprise-spatial';

function App() {
  return (
    <div>
      <LayerManager
        layers={layers}
        onLayerToggle={(id, visible) => console.log(id, visible)}
      />

      <SpatialAnalyzer
        features={features}
        onResult={(results) => console.log(results)}
      />
    </div>
  );
}
```

## API Documentation

### Geometry Factory

Create various geometry types:
- `createPoint(x, y, z?)` - Create point geometry
- `createLineString(positions)` - Create line string
- `createPolygon(rings)` - Create polygon
- `createCircle(center, radius, steps?)` - Create circle as polygon
- `getBounds(geometry)` - Calculate bounding box
- `getCentroid(geometry)` - Calculate centroid

### Topology Engine

Test spatial relationships:
- `intersects(geom1, geom2)` - Test intersection
- `contains(geom1, geom2)` - Test containment
- `within(geom1, geom2)` - Test if within
- `overlaps(geom1, geom2)` - Test overlap
- `distance(geom1, geom2)` - Calculate distance

### Projection Registry

Access coordinate systems:
- `search(query)` - Search projections by name/code
- `getByEPSG(code)` - Get projection by EPSG code
- `getUTMZone(lon, lat)` - Get UTM zone for location
- `suggestProjection(bounds)` - Suggest best projection

## Development

```bash
# Install dependencies
npm install

# Build the library
npm run build

# Run tests
npm test

# Lint code
npm run lint

# Format code
npm run format
```

## Requirements

- Node.js >= 18.0.0
- React >= 18.0.0 (for UI components)

## License

MIT

## Author

HarborGrid

## Contributing

Contributions are welcome! Please follow the existing code style and include tests for new features.

## Support

For issues and questions, please open an issue on GitHub.
