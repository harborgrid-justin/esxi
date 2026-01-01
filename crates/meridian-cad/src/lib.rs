//! # Meridian CAD Engine
//!
//! Enterprise-grade Computer-Aided Design (CAD) engine for the $983M Meridian SaaS Platform.
//! Provides high-precision vector graphics, constraint-based parametric design, and
//! professional CAD tools for engineering and architectural applications.
//!
//! ## Features
//!
//! - **Vector Primitives**: Points, Lines, Arcs, Bezier curves, Splines, Polygons, Ellipses
//! - **Canvas System**: Multi-layer drawing with viewport transforms and world coordinates
//! - **Drawing Tools**: Professional CAD tools including Pen, Rectangle, Circle, Text, Dimensions, Measurement
//! - **Geometric Constraints**: Parallel, Perpendicular, Tangent, Coincident, Fixed, Angle, Distance constraints
//! - **Constraint Solver**: Newton-Raphson iteration solver for parametric design
//! - **High Precision**: 128-bit decimal arithmetic for engineering accuracy
//! - **Smart Snapping**: Grid snapping, object snapping, and intelligent guides
//! - **Export Formats**: DXF, SVG, and PDF export capabilities
//! - **Undo/Redo**: Command pattern with full history management
//!
//! ## Architecture
//!
//! The CAD engine is built on a layered architecture:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │              Export Layer (DXF/SVG/PDF)             │
//! └─────────────────────────────────────────────────────┘
//! ┌─────────────────────────────────────────────────────┐
//! │           Tools Layer (Drawing & Editing)           │
//! └─────────────────────────────────────────────────────┘
//! ┌─────────────────────────────────────────────────────┐
//! │     Constraints & Solver (Parametric Design)        │
//! └─────────────────────────────────────────────────────┘
//! ┌─────────────────────────────────────────────────────┐
//! │    Canvas & Viewport (Rendering & Transforms)       │
//! └─────────────────────────────────────────────────────┘
//! ┌─────────────────────────────────────────────────────┐
//! │   Primitives Layer (Vector Geometry Foundation)     │
//! └─────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use meridian_cad::prelude::*;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a new CAD canvas
//! let mut canvas = Canvas::new("Engineering Drawing");
//!
//! // Add a layer for construction geometry
//! let layer_id = canvas.add_layer("Construction", LayerStyle::default());
//!
//! // Draw a rectangle using the rectangle tool
//! let rect_tool = RectangleTool::new();
//! let rectangle = rect_tool.draw(
//!     Point::new(0.0, 0.0),
//!     Point::new(100.0, 50.0),
//! )?;
//!
//! // Add geometric constraints
//! let mut solver = ConstraintSolver::new();
//! solver.add_constraint(Constraint::perpendicular(
//!     rectangle.edge(0),
//!     rectangle.edge(1),
//! ))?;
//!
//! // Solve constraints
//! solver.solve(100)?;
//!
//! // Export to DXF
//! let exporter = DxfExporter::new();
//! exporter.export(&canvas, "output.dxf")?;
//! # Ok(())
//! # }
//! ```

pub mod canvas;
pub mod constraints;
pub mod export;
pub mod precision;
pub mod primitives;
pub mod snapping;
pub mod solver;
pub mod tools;
pub mod undo;

// Re-exports for convenience
pub use canvas::{Canvas, Layer, LayerStyle, Viewport};
pub use constraints::{Constraint, ConstraintType};
pub use export::{DxfExporter, PdfExporter, SvgExporter};
pub use precision::{Decimal128, EngineeringPrecision};
pub use primitives::{Arc, Bezier, Ellipse, Line, Point, Polygon, Spline};
pub use snapping::{GridSnap, ObjectSnap, SnapResult, SmartGuide};
pub use solver::{ConstraintSolver, SolverConfig};
pub use tools::{
    CircleTool, DimensionTool, MeasureTool, PenTool, RectangleTool, TextTool, Tool,
};
pub use undo::{Command, CommandHistory, UndoManager};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::canvas::*;
    pub use crate::constraints::*;
    pub use crate::export::*;
    pub use crate::precision::*;
    pub use crate::primitives::*;
    pub use crate::snapping::*;
    pub use crate::solver::*;
    pub use crate::tools::*;
    pub use crate::undo::*;
}

/// CAD Engine errors
#[derive(Debug, thiserror::Error)]
pub enum CadError {
    #[error("Invalid geometry: {0}")]
    InvalidGeometry(String),

    #[error("Constraint error: {0}")]
    ConstraintError(String),

    #[error("Solver failed to converge: {0}")]
    SolverConvergence(String),

    #[error("Export error: {0}")]
    ExportError(String),

    #[error("Import error: {0}")]
    ImportError(String),

    #[error("Layer not found: {0}")]
    LayerNotFound(String),

    #[error("Tool error: {0}")]
    ToolError(String),

    #[error("Precision error: {0}")]
    PrecisionError(String),

    #[error("Undo/Redo error: {0}")]
    UndoError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Result type for CAD operations
pub type CadResult<T> = Result<T, CadError>;

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const ENGINE_NAME: &str = "Meridian CAD Engine";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(VERSION, "0.5.0");
    }

    #[test]
    fn test_engine_name() {
        assert_eq!(ENGINE_NAME, "Meridian CAD Engine");
    }
}
