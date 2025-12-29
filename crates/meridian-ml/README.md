# Meridian ML - Machine Learning for Spatial Analysis

Version 0.2.5

## Overview

`meridian-ml` provides comprehensive machine learning capabilities for geospatial data analysis in the Meridian GIS Platform. This crate implements state-of-the-art algorithms specifically designed for spatial data, including classification, regression, clustering, and predictive modeling with spatial awareness.

## Features

- **ONNX Model Support**: Import and export models in ONNX format for interoperability
- **GPU Acceleration**: Optional CUDA/OpenCL support for high-performance computing
- **Spatial Cross-Validation**: Cross-validation methods that respect spatial autocorrelation
- **AutoML Capabilities**: Automated model selection and hyperparameter tuning
- **Model Explainability**: SHAP values and feature importance for model interpretation
- **Incremental Learning**: Online learning support for streaming data
- **Distributed Training**: Multi-worker training with data and model parallelism

## Module Structure

### Core Modules
- **error** - Error types and result handling
- **models** - Model management, registry, and serialization
- **features** - Feature engineering and extraction

### Feature Engineering
- **features/spatial** - Spatial feature extraction (geometry-based features)
- **features/raster** - Raster feature extraction (texture, statistics)
- **features/temporal** - Temporal feature extraction (time series features)
- **features/scaler** - Feature scaling and normalization

### Classification
- **classification/land_cover** - Land cover classification
- **classification/change_detection** - Change detection algorithms

### Regression
- **regression/spatial_regression** - Geographically Weighted Regression (GWR)
- **regression/kriging** - Kriging interpolation methods

### Clustering
- **clustering/spatial** - Spatial clustering (DBSCAN, OPTICS)
- **clustering/hotspot** - Hotspot analysis (Getis-Ord Gi*)

### Prediction
- **prediction/time_series** - Spatial time series forecasting
- **prediction/anomaly** - Anomaly detection

### Inference
- **inference/runtime** - ONNX runtime integration
- **inference/batch** - Batch inference for large datasets

### Training
- **training** - Model training and optimization
- **training/distributed** - Distributed training support

### Evaluation
- **evaluation** - Model evaluation framework
- **evaluation/metrics** - Spatial accuracy metrics

## Quick Start

```rust
use meridian_ml::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Extract spatial features
    let features = SpatialFeatureExtractor::new()
        .with_features(vec![
            SpatialFeatureType::Area,
            SpatialFeatureType::Compactness,
        ])
        .extract_from_geometries(&geometries)?;

    // Train land cover classifier
    let mut classifier = LandCoverClassifier::new();
    classifier.train(&features, &labels)?;

    // Classify new data
    let predictions = classifier.classify(&new_features)?;

    // Export to ONNX
    model.export_onnx("land_cover.onnx").await?;

    Ok(())
}
```

## Feature Flags

- `default` - ONNX and AutoML support
- `onnx` - Enable ONNX model import/export
- `gpu-cuda` - Enable CUDA GPU acceleration
- `gpu-opencl` - Enable OpenCL GPU acceleration
- `gpu` - Enable GPU support (defaults to CUDA)
- `automl` - Enable AutoML capabilities
- `explainability` - Enable model explainability features
- `full` - Enable all features

## Dependencies

Core ML libraries:
- ndarray - N-dimensional arrays
- linfa - Machine learning framework
- smartcore - Machine learning algorithms
- tract - ONNX runtime

## Architecture

The crate is designed with modularity and extensibility in mind:

1. **Trait-based abstractions**: Common traits for classifiers, regressors, and clusterers
2. **Feature engineering pipeline**: Composable feature extractors
3. **Model registry**: Version control and management for models
4. **Spatial awareness**: Built-in support for spatial autocorrelation and spatial CV
5. **Production-ready**: Async/await, error handling, logging, and monitoring

## License

MIT OR Apache-2.0
