//! Error types for routing operations

use thiserror::Error;

/// Result type for routing operations
pub type Result<T> = std::result::Result<T, RoutingError>;

/// Errors that can occur during routing operations
#[derive(Error, Debug)]
pub enum RoutingError {
    /// No route found between origin and destination
    #[error("No route found from {origin:?} to {destination:?}")]
    NoRouteFound {
        origin: geo_types::Point,
        destination: geo_types::Point,
    },

    /// Invalid coordinates
    #[error("Invalid coordinates: {0}")]
    InvalidCoordinates(String),

    /// Node not found in graph
    #[error("Node {0} not found in graph")]
    NodeNotFound(usize),

    /// Edge not found in graph
    #[error("Edge {0} not found in graph")]
    EdgeNotFound(usize),

    /// Graph is empty
    #[error("Graph is empty, no nodes or edges")]
    EmptyGraph,

    /// Graph construction error
    #[error("Graph construction failed: {0}")]
    GraphConstruction(String),

    /// Preprocessing error
    #[error("Preprocessing failed: {0}")]
    PreprocessingFailed(String),

    /// Invalid routing profile
    #[error("Invalid routing profile: {0}")]
    InvalidProfile(String),

    /// Invalid time window
    #[error("Invalid time window: {0}")]
    InvalidTimeWindow(String),

    /// Optimization error
    #[error("Optimization failed: {0}")]
    OptimizationFailed(String),

    /// GTFS parsing error
    #[cfg(feature = "transit")]
    #[error("GTFS parsing error: {0}")]
    GtfsError(#[from] gtfs_structures::Error),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

impl RoutingError {
    /// Create a generic error from a message
    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }
}
