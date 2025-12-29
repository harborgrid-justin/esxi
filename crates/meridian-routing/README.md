# Meridian Routing Engine v0.2.5

Advanced routing and network analysis for the Meridian GIS Platform.

## Features

### High-Performance Routing Algorithms
- **Dijkstra**: Classic shortest path algorithm
- **A\***: Heuristic-guided search with geographic distance
- **Contraction Hierarchies (CH)**: Ultra-fast preprocessing-based routing
- **Hub Labeling**: Constant-time distance queries
- **ALT (A\*, Landmarks, Triangle)**: Enhanced A\* with landmarks
- **Many-to-Many**: Parallel distance matrix computation

### Isochrone Generation
- Time-based reachability analysis
- Multi-contour isochrone polygons
- Multimodal isochrones (walking + transit + cycling)
- Convex hull polygon generation

### Route Optimization
- **TSP (Traveling Salesman Problem)**: Multiple solving strategies
  - Nearest neighbor heuristic
  - 2-opt local search
  - Simulated annealing
  - Genetic algorithms
- **VRP (Vehicle Routing Problem)**: Fleet optimization
  - Clarke-Wright savings algorithm
  - Sweep algorithm
  - Capacity constraints
- **Pickup-Delivery**: Specialized PDP solver

### Traffic Integration
- **Historical Patterns**: Weekly traffic profiles (168 hourly slots)
- **Real-time Updates**: Live traffic incidents and congestion
- **Prediction**: ML-based traffic forecasting
- Traffic speed classifications (Free, Light, Moderate, Heavy, Congested)

### Multimodal Routing
- **Public Transit**: GTFS support for buses, trains, trams
- **Walking**: Pedestrian-optimized routing with gradient constraints
- **Cycling**: Bicycle routing with bike type profiles
  - Road bikes, Mountain bikes, Hybrid, Electric
  - Surface type awareness
  - Gradient tolerance

### Graph Features
- Memory-efficient compressed adjacency lists
- Spatial indexing for fast node lookup
- Turn restrictions and penalties
- Time-dependent edge costs
- Graph partitioning for hierarchical routing
- Serialization with compression (gzip)

## Performance Targets

- **Sub-second routing** for continental-scale networks
- **Millions of queries per second** with Contraction Hierarchies
- **Parallel processing** with Rayon for batch operations
- **Memory-efficient** graph storage

## Architecture

```
meridian-routing/
├── algorithms/          # Routing algorithms
│   ├── dijkstra.rs     # Basic shortest path
│   ├── astar.rs        # Heuristic search
│   ├── ch.rs           # Contraction hierarchies
│   ├── hub_labels.rs   # Hub labeling
│   ├── alt.rs          # ALT preprocessing
│   └── many_to_many.rs # Distance matrices
├── graph/              # Graph data structures
│   ├── mod.rs          # Core graph
│   ├── node.rs         # Node representation
│   ├── edge.rs         # Edge with costs
│   ├── builder.rs      # Graph construction
│   └── partition.rs    # Graph partitioning
├── isochrone/          # Reachability analysis
│   ├── builder.rs      # Isochrone generation
│   └── multimodal.rs   # Multimodal isochrones
├── optimization/       # Route optimization
│   ├── tsp.rs          # Traveling salesman
│   ├── vrp.rs          # Vehicle routing
│   └── pickup_delivery.rs
├── traffic/            # Traffic integration
│   ├── historical.rs   # Historical patterns
│   ├── realtime.rs     # Live updates
│   └── prediction.rs   # ML prediction
├── multimodal/         # Multi-mode routing
│   ├── transit.rs      # GTFS integration
│   ├── walking.rs      # Pedestrian
│   └── cycling.rs      # Bicycle
├── api/                # Request/Response API
│   ├── request.rs      # Routing requests
│   └── response.rs     # Route responses
└── profile/            # Routing profiles
    ├── mod.rs          # Profile definitions
    └── vehicle.rs      # Vehicle specs

35 files, ~6,800 lines of code
```

## Usage Example

```rust
use meridian_routing::{RoutingEngine, RoutingRequest, RoutingProfile};
use geo_types::Point;

// Create routing engine
let mut engine = RoutingEngine::new();

// Load graph (from OSM or serialized format)
engine.load_from_osm("map.osm.pbf")?;

// Preprocess with Contraction Hierarchies for fast queries
engine.preprocess_ch()?;

// Create routing request
let request = RoutingRequest::new(
    Point::new(-122.4194, 37.7749),  // San Francisco
    Point::new(-118.2437, 34.0522),  // Los Angeles
    RoutingProfile::driving(),
);

// Find route
let response = engine.route(&request)?;
println!("Distance: {:.2} km", response.distance / 1000.0);
println!("Duration: {:.0} min", response.duration / 60.0);

// Generate isochrone (30-minute reachability)
let isochrone = engine.isochrone(
    Point::new(-122.4194, 37.7749),
    1800.0,  // 30 minutes
    &RoutingProfile::driving(),
)?;
println!("Reachable area: {:.2} km²", isochrone.area() / 1_000_000.0);

// Solve TSP for delivery route
let waypoints = vec![
    Point::new(-122.41, 37.77),
    Point::new(-122.42, 37.78),
    Point::new(-122.43, 37.79),
];
let tsp_solution = engine.optimize_tsp(&waypoints, &RoutingProfile::driving())?;
println!("Optimal tour cost: {:.2}", tsp_solution.total_cost);
```

## Dependencies

- `petgraph` - Graph algorithms
- `ordered-float` - Floating-point ordering
- `rayon` - Parallel processing
- `hashbrown` - High-performance hash maps
- `geo` / `geo-types` - Geographic types
- `gtfs-structures` - GTFS transit data (optional)
- `priority-queue` - Priority queue implementation
- `chrono` - Date/time handling

## Features

- `default` - Standard features including transit
- `transit` - GTFS public transit support
- `parallel` - Parallel routing algorithms
- `experimental` - Experimental features

## License

MIT OR Apache-2.0

## Status

**v0.2.5** - Production-ready routing engine for enterprise GIS applications.
