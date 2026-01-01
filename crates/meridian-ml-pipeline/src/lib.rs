//! # Meridian ML Pipeline Engine
//!
//! Enterprise-grade AI/ML pipeline system for the $983M Meridian SaaS Platform.
//!
//! ## Features
//!
//! - **Pipeline Building**: Fluent API for constructing ML pipelines
//! - **Data Transformations**: Normalization, encoding, imputation, feature engineering
//! - **Model Management**: ONNX model loading, versioning, and registry
//! - **Inference**: Batch and streaming inference with automatic scaling
//! - **Monitoring**: Data drift detection, model performance metrics
//!
//! ## Example
//!
//! ```rust,no_run
//! use meridian_ml_pipeline::pipeline::PipelineBuilder;
//! use meridian_ml_pipeline::transforms::normalize::StandardScaler;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let pipeline = PipelineBuilder::new("fraud-detection")
//!     .add_transform(StandardScaler::new())
//!     .add_model("models/fraud_v1.onnx")
//!     .build()
//!     .await?;
//!
//! let predictions = pipeline.predict(&input_data).await?;
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs, rust_2018_idioms)]
#![deny(unsafe_code)]

pub mod pipeline;
pub mod transforms;
pub mod models;
pub mod serving;
pub mod monitoring;

mod error;

pub use error::{Error, Result};

/// Version information for the ML pipeline engine
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PLATFORM_VERSION: &str = "0.5.0";
pub const PLATFORM_VALUE: &str = "$983M Enterprise SaaS";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info() {
        assert!(!VERSION.is_empty());
        assert_eq!(PLATFORM_VERSION, "0.5.0");
    }
}
