# Meridian CAD Engine v0.5.0

Enterprise-grade Computer-Aided Design (CAD) engine for the $983M Meridian SaaS Platform. Provides high-precision vector graphics, constraint-based parametric design, and professional CAD tools for engineering and architectural applications.

## Features

### Vector Primitives (`src/primitives.rs`)
- **Point**: 2D/3D points with distance calculations, transformations
- **Line**: Straight line segments with intersection detection
- **Arc**: Circular arcs with full/partial circles
- **Bezier**: Cubic Bezier curves with De Casteljau algorithm
- **Spline**: B-spline curves with Cox-de Boor recursion
- **Polygon**: Closed polygons with area/perimeter calculations
- **Ellipse**: Rotated ellipses with containment testing
- Full geometric operations (distance, midpoint, rotation, scaling, translation)

### Canvas System (`src/canvas.rs`)
- **Multi-Layer Support**: Organize drawings with unlimited layers
- **Viewport Management**: Pan, zoom, rotate with world-to-screen transforms
- **Entity Management**: Add, remove, modify entities with layer isolation
- **Transform Pipeline**: Efficient nalgebra-based matrix transformations
- **Metadata**: Units, grid spacing, snap settings, timestamps

### Drawing Tools (`src/tools.rs`)
- **PenTool**: Freehand drawing with smooth Bezier curves
- **RectangleTool**: Rectangle drawing (two-corner and center modes)
- **CircleTool**: Circle/arc creation
- **LineTool**: Straight lines with optional chain mode
- **ArcTool**: Three-point arc creation
- **TextTool**: Text annotations with fonts and rotation
- **DimensionTool**: Measurement dimensions (linear, horizontal, vertical, angular)
- **MeasureTool**: Distance and angle measurement without entity creation

### Geometric Constraints (`src/constraints.rs`)
14 constraint types for parametric design:
- **Parallel/Perpendicular**: Line alignment
- **Tangent**: Curve tangency
- **Coincident**: Point alignment
- **Fixed**: Anchor points
- **Angle/Distance**: Dimensional constraints
- **Horizontal/Vertical**: Orthogonal constraints
- **Equal Length/Radius**: Symmetry constraints
- **Midpoint/Symmetric**: Alignment constraints
- **Concentric/Collinear**: Advanced geometric relationships

### Constraint Solver (`src/solver.rs`)
- **Newton-Raphson Iteration**: Non-linear constraint solving
- **Jacobian Calculation**: Numerical differentiation
- **Least Squares Solver**: LU decomposition for overdetermined systems
- **Incremental Solving**: Real-time constraint resolution
- **Multiple Configurations**: Fast, Precise, Robust modes
- **Convergence Detection**: Automatic tolerance checking

### High-Precision Arithmetic (`src/precision.rs`)
- **Decimal128**: 128-bit decimal arithmetic for engineering accuracy
- **Engineering Precision**: Architectural, Mechanical, Scientific presets
- **Unit Conversion**: mm, cm, m, inches, feet, micrometers, nanometers
- **Rounding Modes**: HalfUp, HalfDown, HalfEven, Up, Down
- **Tolerance Comparison**: Approximate equality with configurable tolerance

### Smart Snapping (`src/snapping.rs`)
- **Grid Snap**: Configurable grid with subdivisions and angle snapping
- **Object Snap**: 12 snap types (Endpoint, Midpoint, Center, Intersection, Perpendicular, Nearest, Tangent, Quadrant)
- **Smart Guides**: Horizontal/vertical alignment, distribution
- **Priority System**: Intelligent snap selection
- **Visual Feedback**: Snap messages and distance display

### Export Formats (`src/export.rs`)
- **DXF**: AutoCAD Drawing Exchange Format export
- **SVG**: Scalable Vector Graphics with layers as groups
- **PDF**: Printable PDF documents
- **Batch Export**: Export to multiple formats simultaneously
- **Configuration**: DPI, units, layer visibility, line width scaling

### Undo/Redo System (`src/undo.rs`)
- **Command Pattern**: Full undo/redo with command history
- **Command Types**: Add, Delete, Modify, Move, AddLayer
- **Command Grouping**: Batch operations with transaction support
- **History Management**: Configurable history limits
- **Macros**: `command_group!` for easy transaction creation

## Architecture

```
┌─────────────────────────────────────────────────────┐
│              Export Layer (DXF/SVG/PDF)             │
└─────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────┐
│           Tools Layer (Drawing & Editing)           │
└─────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────┐
│     Constraints & Solver (Parametric Design)        │
└─────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────┐
│    Canvas & Viewport (Rendering & Transforms)       │
└─────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────┐
│   Primitives Layer (Vector Geometry Foundation)     │
└─────────────────────────────────────────────────────┘
```

## Usage Example

```rust
use meridian_cad::prelude::*;

async fn example() -> CadResult<()> {
    // Create a new CAD canvas
    let mut canvas = Canvas::new("Engineering Drawing");
    canvas.metadata.units = Units::Millimeters;

    // Add a layer for construction geometry
    let layer_id = canvas.add_layer("Construction", LayerStyle::default());

    // Draw a rectangle using the rectangle tool
    let rect_tool = RectangleTool::new();
    let rectangle = rect_tool.draw(
        Point::new(0.0, 0.0),
        Point::new(100.0, 50.0),
    )?;

    // Add to canvas
    let mut layer = canvas.get_layer_mut(layer_id)?;
    layer.add_entity(Entity::Polygon(rectangle))?;

    // Add geometric constraints
    let mut solver = ConstraintSolver::new();
    let point_id = Uuid::new_v4();
    solver.set_point(point_id, Point::new(10.0, 20.0));
    solver.add_constraint(Constraint::fixed(
        point_id,
        Point::new(10.0, 20.0)
    ))?;

    // Solve constraints
    let result = solver.solve(None)?;
    println!("Converged: {}, Error: {}", result.converged, result.final_error);

    // Export to multiple formats
    let exporter = BatchExporter::new()
        .with_dxf()
        .with_svg()
        .with_pdf();

    let files = exporter.export_all(&canvas, "/tmp/drawing")?;
    println!("Exported: {:?}", files);

    Ok(())
}
```

## Dependencies

- **nalgebra**: Linear algebra and matrix operations
- **euclid**: Geometric primitives
- **lyon**: Path tessellation and rendering
- **rust_decimal**: High-precision decimal arithmetic
- **dxf**: DXF file format support
- **svg**: SVG generation
- **printpdf**: PDF creation
- **serde**: Serialization/deserialization
- **tokio**: Async runtime
- **thiserror**: Error handling
- **uuid**: Unique identifiers
- **chrono**: Date/time management

## Performance

- **Constraint Solver**: ~100 iterations for convergence on typical constraints
- **Snapping**: O(n) snap detection with spatial optimization
- **Export**: Streaming writers for large drawings
- **Undo/Redo**: O(1) command execution with configurable memory limits

## Enterprise Features

- **Production-Ready**: Comprehensive error handling with `thiserror`
- **Type Safety**: Strong typing throughout with Rust's type system
- **Thread-Safe**: All types implement `Send` + `Sync` where appropriate
- **Async Support**: Built on Tokio for scalable concurrent operations
- **Logging**: Integrated `tracing` for observability
- **Testing**: Unit tests for all critical functionality

## File Structure

```
meridian-cad/
├── Cargo.toml           # Dependencies and metadata
├── src/
│   ├── lib.rs           # Public API and re-exports
│   ├── primitives.rs    # Vector primitives (2,076 lines)
│   ├── canvas.rs        # Canvas and layer management (584 lines)
│   ├── tools.rs         # Drawing tools (724 lines)
│   ├── constraints.rs   # Geometric constraints (551 lines)
│   ├── solver.rs        # Newton-Raphson solver (470 lines)
│   ├── precision.rs     # High-precision arithmetic (571 lines)
│   ├── snapping.rs      # Smart snapping system (507 lines)
│   ├── export.rs        # DXF/SVG/PDF export (612 lines)
│   └── undo.rs          # Command pattern undo/redo (661 lines)
└── README.md            # This file
```

## License

MIT

## Authors

Meridian Engineering Team

## Version History

- **v0.5.0** (2026-01-01): Initial release - Complete CAD engine with all features
