//! # Meridian ML - Machine Learning for Spatial Analysis
//!
//! `meridian-ml` provides comprehensive machine learning capabilities for geospatial data,
//! including classification, regression, clustering, and predictive modeling with spatial awareness.
//!
//! ## Features
//!
//! - **ONNX Model Support**: Import and export models in ONNX format
//! - **GPU Acceleration**: Optional CUDA/OpenCL support for high-performance computing
//! - **Spatial Cross-Validation**: Cross-validation methods that respect spatial autocorrelation
//! - **AutoML**: Automated model selection and hyperparameter tuning
//! - **Model Explainability**: SHAP values and feature importance for model interpretation
//! - **Incremental Learning**: Online learning support for streaming data
//!
//! ## Example
//!
//! ```rust,no_run
//! use meridian_ml::prelude::*;
//!
//! # async fn example() -> Result<()> {
//! // Load spatial features
//! let features = SpatialFeatureExtractor::new()
//!     .extract_from_raster("data.tif").await?;
//!
//! // Train land cover classifier
//! let model = LandCoverClassifier::new()
//!     .with_spatial_cv(5)
//!     .train(&features).await?;
//!
//! // Export to ONNX
//! model.export_onnx("land_cover.onnx").await?;
//! # Ok(())
//! # }
//! ```

pub mod error;
pub mod models;
pub mod features;
pub mod classification;
pub mod regression;
pub mod clustering;
pub mod prediction;
pub mod inference;
pub mod training;
pub mod evaluation;

pub use error::{MlError, Result};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::error::{MlError, Result};
    pub use crate::models::{Model, ModelRegistry, ModelMetadata};
    pub use crate::features::{
        FeatureExtractor, SpatialFeatures, RasterFeatures,
        TemporalFeatures, FeatureScaler, ScalerType,
    };
    pub use crate::classification::{
        Classifier, LandCoverClassifier, ChangeDetector,
    };
    pub use crate::regression::{
        Regressor, SpatialRegression, Kriging, KrigingType,
    };
    pub use crate::clustering::{
        Clusterer, SpatialClusterer, HotspotAnalyzer, HotspotStatistic,
    };
    pub use crate::prediction::{
        Predictor, SpatialTimeSeries, AnomalyDetector,
    };
    pub use crate::inference::{
        InferenceRuntime, BatchInference, InferenceConfig,
    };
    pub use crate::training::{
        Trainer, TrainingConfig, DistributedTrainer,
    };
    pub use crate::evaluation::{
        Evaluator, SpatialMetrics, ConfusionMatrix,
    };
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Check if GPU support is available
pub fn gpu_available() -> bool {
    #[cfg(feature = "gpu")]
    {
        gpu::check_availability()
    }
    #[cfg(not(feature = "gpu"))]
    {
        false
    }
}

#[cfg(feature = "gpu")]
mod gpu {
    pub fn check_availability() -> bool {
        #[cfg(feature = "gpu-cuda")]
        {
            cudarc::driver::safe::CudaDevice::new(0).is_ok()
        }
        #[cfg(all(feature = "gpu-opencl", not(feature = "gpu-cuda")))]
        {
            ocl::Platform::list().map(|p| !p.is_empty()).unwrap_or(false)
        }
        #[cfg(not(any(feature = "gpu-cuda", feature = "gpu-opencl")))]
        {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_gpu_check() {
        // Should not panic
        let _ = gpu_available();
    }
}
