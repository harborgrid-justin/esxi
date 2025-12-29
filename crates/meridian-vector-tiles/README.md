# Meridian Vector Tiles

Enterprise-grade vector tile generation and serving library for the Meridian GIS Platform.

## Features

### Core Capabilities

- **MVT 2.0 Compliance**: Full Mapbox Vector Tile specification support
- **High Performance**: Parallel tile generation with efficient geometry processing
- **Multiple Formats**: MVT, PMTiles, MBTiles support
- **Dynamic Simplification**: Automatic geometry simplification per zoom level
- **Proper Geometry Clipping**: Accurate tile boundary clipping
- **Attribute Optimization**: Efficient encoding and filtering

### Data Sources

- **PostGIS**: Direct database tile generation with spatial queries
- **File Sources**: GeoJSON and other file formats
- **PMTiles**: Read from existing PMTiles archives

### Storage Backends

- **MBTiles**: SQLite-based tile storage
- **PMTiles**: Single-file archive format
- **Directory**: File system with ZXY/TMS/Quadkey layouts
- **S3**: Cloud storage with AWS S3

### Built-in Server

- **Production-Ready**: Axum-based HTTP server
- **Caching**: In-memory LRU cache with configurable size
- **Compression**: Gzip and Brotli support
- **ETag Support**: HTTP caching headers
- **CORS**: Cross-origin resource sharing
- **TileJSON**: Metadata endpoint

### Style Support

- **Mapbox Style Spec**: Compatible with Mapbox GL JS
- **Sprite Sheets**: Icon and symbol management
- **Glyph Management**: Font handling for labels

### Tile Seeding

- **Multiple Strategies**: Bounds, zoom range, route, pyramid
- **Parallel Processing**: Multi-threaded tile generation
- **Progress Tracking**: Real-time statistics
- **Resume Support**: Skip existing tiles

## Quick Start

### Generate Tiles from PostGIS

```rust
use meridian_vector_tiles::{
    generation::TileGenerator,
    source::postgis::{PostGISSource, PostGISConfig, LayerConfig},
    encoding::mvt::MvtEncoder,
    tile::coordinate::TileCoordinate,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure PostGIS source
    let mut config = PostGISConfig {
        connection_string: "postgresql://user:pass@localhost/db".to_string(),
        layers: vec![
            LayerConfig {
                name: "roads".to_string(),
                table: "osm_roads".to_string(),
                geometry_column: "geom".to_string(),
                properties: vec!["name".to_string(), "highway".to_string()],
                min_zoom: 0,
                max_zoom: 14,
                ..Default::default()
            }
        ],
        ..Default::default()
    };

    let source = PostGISSource::with_config(config).await?;

    // Generate a tile
    let generator = TileGenerator::new();
    let tile = TileCoordinate::new(10, 512, 384);

    if let Some(mvt_tile) = generator.generate(&source, tile).await? {
        let encoder = MvtEncoder::new();
        let mvt_bytes = encoder.encode(&mvt_tile)?;
        println!("Generated tile: {} bytes", mvt_bytes.len());
    }

    Ok(())
}
```

### Run Tile Server

```rust
use meridian_vector_tiles::{
    server::{TileServer, ServerConfig},
    source::postgis::PostGISSource,
    storage::mbtiles::MBTilesStorage,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup source and storage
    let source = PostGISSource::new("postgresql://localhost/db").await?;
    let storage = MBTilesStorage::new("tiles.mbtiles").await?;

    // Configure server
    let config = ServerConfig {
        bind_addr: "0.0.0.0:8080".parse().unwrap(),
        cors: true,
        cache_enabled: true,
        cache_size: 1000,
        ..Default::default()
    };

    // Run server
    let server = TileServer::with_config(source, storage, config);
    server.run().await?;

    Ok(())
}
```

### Seed Tiles

```rust
use meridian_vector_tiles::{
    seeding::{TileSeeder, SeedingConfig},
    seeding::strategy::BoundsSeedingStrategy,
    source::postgis::PostGISSource,
    storage::mbtiles::MBTilesStorage,
    tile::bounds::TileBounds,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = PostGISSource::new("postgresql://localhost/db").await?;
    let storage = MBTilesStorage::new("tiles.mbtiles").await?;

    let config = SeedingConfig {
        workers: 8,
        skip_existing: true,
        progress: true,
        ..Default::default()
    };

    let seeder = TileSeeder::with_config(source, storage, config);

    // Seed tiles within bounds
    let bounds = TileBounds::new(-122.5, 37.5, -122.0, 38.0);
    let strategy = BoundsSeedingStrategy::new(bounds, 0, 14);

    let stats = seeder.seed(strategy).await?;
    println!("Seeded {} tiles", stats.generated);

    Ok(())
}
```

## Architecture

### Tile Generation Pipeline

```
Source (PostGIS/Files)
    → Feature Extraction
    → Geometry Simplification (zoom-aware)
    → Geometry Clipping (tile bounds)
    → Attribute Filtering
    → MVT Encoding
    → Compression (gzip/brotli)
    → Storage (MBTiles/PMTiles/Directory/S3)
```

### Server Request Flow

```
HTTP Request
    → Parse tile coordinates (z/x/y)
    → Check cache (LRU)
    → Check storage
    → Generate if needed
        → Fetch from source
        → Apply generation pipeline
        → Store for future requests
    → Compress response
    → Add cache headers
    → Return tile
```

## Configuration

### Generation Config

```rust
use meridian_vector_tiles::generation::GenerationConfig;

let config = GenerationConfig {
    extent: 4096,              // Tile extent (MVT standard)
    buffer: 64,                // Buffer around tile
    simplify: true,            // Enable simplification
    simplify_tolerance: 1.0,   // Simplification tolerance
    clip: true,                // Enable clipping
    max_features: Some(10000), // Max features per tile
    max_tile_size: Some(500_000), // Max 500KB
    enable_overzoom: true,     // Allow overzooming
    max_overzoom: 5,           // Max 5 levels
};
```

### Storage Options

#### MBTiles
```rust
let storage = MBTilesStorage::new("tiles.mbtiles").await?;
storage.initialize("My Tiles", "pbf").await?;
```

#### Directory
```rust
let storage = DirectoryStorage::with_layout(
    "/path/to/tiles",
    DirectoryLayout::ZXY
).await?;
```

#### S3
```rust
let config = S3Config {
    bucket: "my-tiles".to_string(),
    prefix: "tiles/".to_string(),
    region: Region::UsEast1,
    ..Default::default()
};
let storage = S3Storage::new(config);
```

## Performance

### Optimizations

- **Parallel Generation**: Uses Rayon for multi-threaded tile processing
- **Geometry Simplification**: Reduces complexity based on zoom level
- **Efficient Clipping**: Fast tile boundary clipping algorithms
- **Smart Caching**: LRU cache for frequently accessed tiles
- **Streaming Queries**: Memory-efficient database queries
- **Connection Pooling**: SQLx connection pooling for databases

### Benchmarks

On a typical server (8 cores, 16GB RAM):
- **Generation**: ~500-1000 tiles/second (depends on complexity)
- **Serving**: ~10,000 req/sec with caching
- **Encoding**: ~5,000 MVT tiles/second

## MVT Specification

This library implements the Mapbox Vector Tile Specification 2.0:

- ✅ Protocol Buffer encoding
- ✅ Command-based geometry encoding
- ✅ ZigZag integer encoding
- ✅ 4096 default extent
- ✅ Multiple layers per tile
- ✅ Feature IDs
- ✅ Property encoding with key/value tables
- ✅ Geometry types: Point, LineString, Polygon

## Contributing

This is part of the Meridian GIS Platform. See main repository for contribution guidelines.

## License

MIT OR Apache-2.0
