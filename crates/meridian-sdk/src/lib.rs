//! Meridian GIS Platform SDK
//!
//! This crate provides a client SDK for interacting with the Meridian GIS Platform API.
//!
//! # Examples
//!
//! ```no_run
//! use meridian_sdk::{Client, ClientConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = ClientConfig::new("http://localhost:3000");
//!     let client = Client::new(config)?;
//!
//!     // List all layers
//!     let layers = client.layers().list().await?;
//!
//!     for layer in layers {
//!         println!("Layer: {}", layer.name);
//!     }
//!
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod layers;
pub mod features;
pub mod query;
pub mod analysis;
pub mod error;

// Re-export main types
pub use client::{Client, ClientConfig};
pub use error::{Error, Result};
pub use layers::{Layer, LayerClient};
pub use features::{Feature, FeatureClient};
pub use query::{QueryBuilder, SpatialPredicate};
pub use analysis::{AnalysisClient, BufferOptions, ClipOptions};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_config() {
        let config = ClientConfig::new("http://localhost:3000");
        assert_eq!(config.base_url(), "http://localhost:3000");
    }
}
