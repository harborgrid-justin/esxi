//! YAML-based pipeline configuration.

use crate::config::{PipelineConfigFile, PipelineDefinition, SinkConfig, SourceConfig, TransformConfig};
use crate::error::Result;
use crate::pipeline::PipelineBuilder;
use crate::Pipeline;
use serde::{Deserialize, Serialize};

/// YAML pipeline configuration builder.
pub struct YamlPipelineConfig {
    config: PipelineConfigFile,
}

impl YamlPipelineConfig {
    /// Load from YAML file.
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let config = PipelineConfigFile::from_yaml_file(path)?;
        Ok(Self { config })
    }

    /// Load from YAML string.
    pub fn from_str(yaml: &str) -> Result<Self> {
        let config = PipelineConfigFile::from_yaml_str(yaml)?;
        Ok(Self { config })
    }

    /// Get the underlying config.
    pub fn config(&self) -> &PipelineConfigFile {
        &self.config
    }

    /// Build a pipeline from the configuration.
    pub fn build(&self) -> Result<Pipeline> {
        let mut builder = PipelineBuilder::new(&self.config.pipeline.name)
            .version(&self.config.pipeline.version);

        // Set execution mode
        match self.config.pipeline.execution_mode.as_str() {
            "batch" | "" => builder = builder.batch(),
            "streaming" => builder = builder.streaming(),
            "micro-batch" => builder = builder.micro_batch(),
            _ => {}
        }

        // Set parallelism
        builder = builder.with_parallelism(self.config.pipeline.parallelism);

        // Set batch size
        builder = builder.batch_size(self.config.pipeline.batch_size);

        // Set checkpointing
        if self.config.pipeline.checkpointing {
            builder = builder.with_checkpointing(true);
            if let Some(ref checkpoint_dir) = self.config.pipeline.checkpoint_dir {
                builder = builder.checkpoint_dir(checkpoint_dir);
            }
        }

        // Build the pipeline
        let pipeline = builder.build()?;

        tracing::info!(
            pipeline = %pipeline.name,
            version = %pipeline.version,
            "Built pipeline from YAML configuration"
        );

        Ok(pipeline)
    }
}

/// Example YAML configuration templates.
pub struct ConfigTemplates;

impl ConfigTemplates {
    /// Basic ETL pipeline template.
    pub fn basic_etl() -> &'static str {
        r#"
version: "1.0"
pipeline:
  name: basic-etl-pipeline
  version: "1.0.0"
  description: "Basic ETL pipeline for geospatial data"
  execution_mode: batch
  parallelism: 4
  checkpointing: false
  batch_size: 1000

  source:
    type: file
    format: geojson
    path: /data/input.geojson

  transforms:
    - type: projection
      from: EPSG:4326
      to: EPSG:3857
    - type: filter
      expression: "population > 100000"
    - type: validate
      action: filter

  sink:
    type: file
    format: geojson
    path: /data/output.geojson
"#
    }

    /// Database to database pipeline template.
    pub fn database_pipeline() -> &'static str {
        r#"
version: "1.0"
pipeline:
  name: database-etl-pipeline
  version: "1.0.0"
  description: "ETL pipeline from PostgreSQL to PostGIS"
  execution_mode: batch
  parallelism: 8
  checkpointing: true
  checkpoint_dir: /tmp/checkpoints

  source:
    type: database
    db_type: postgis
    connection_string: postgresql://user:pass@localhost/source_db
    table: source_table
    geometry_column: geom

  transforms:
    - type: projection
      from: EPSG:4326
      to: EPSG:3857
    - type: buffer
      distance: 100
    - type: aggregate
      group_by: ["region"]
      aggregations:
        - column: population
          function: sum

  sink:
    type: database
    db_type: postgis
    connection_string: postgresql://user:pass@localhost/target_db
    table: target_table
    geometry_column: geom
    write_mode: overwrite
"#
    }

    /// Streaming pipeline template.
    pub fn streaming_pipeline() -> &'static str {
        r#"
version: "1.0"
pipeline:
  name: streaming-pipeline
  version: "1.0.0"
  description: "Real-time geospatial data processing"
  execution_mode: streaming
  parallelism: 4

  source:
    type: stream
    stream_type: kafka
    brokers: ["localhost:9092"]
    topic: geo-events
    group_id: pipeline-consumer

  transforms:
    - type: validate
      action: filter
    - type: projection
      from: EPSG:4326
      to: EPSG:3857
    - type: enrich
      source: api
      url: https://api.example.com/enrich
      key_field: location_id

  sink:
    type: database
    db_type: postgis
    connection_string: postgresql://user:pass@localhost/realtime_db
    table: events
    write_mode: append
"#
    }

    /// Vector tile generation template.
    pub fn vector_tiles_pipeline() -> &'static str {
        r#"
version: "1.0"
pipeline:
  name: vector-tiles-pipeline
  version: "1.0.0"
  description: "Generate vector tiles from PostGIS"
  execution_mode: batch
  parallelism: 16

  source:
    type: database
    db_type: postgis
    connection_string: postgresql://user:pass@localhost/geo_db
    table: buildings
    geometry_column: geom

  transforms:
    - type: projection
      from: EPSG:4326
      to: EPSG:3857
    - type: simplify
      tolerance: 0.5
    - type: validate
      action: fix

  sink:
    type: vector_tiles
    output_dir: /data/tiles
    layer_name: buildings
    min_zoom: 10
    max_zoom: 16
    buffer: 64
    extent: 4096
"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yaml_config_from_str() {
        let yaml = ConfigTemplates::basic_etl();
        let config = YamlPipelineConfig::from_str(yaml).unwrap();
        assert_eq!(config.config().pipeline.name, "basic-etl-pipeline");
    }

    #[test]
    fn test_build_pipeline_from_yaml() {
        let yaml = ConfigTemplates::basic_etl();
        let config = YamlPipelineConfig::from_str(yaml).unwrap();
        let pipeline = config.build().unwrap();
        assert_eq!(pipeline.name, "basic-etl-pipeline");
        assert_eq!(pipeline.version, "1.0.0");
    }

    #[test]
    fn test_database_pipeline_template() {
        let yaml = ConfigTemplates::database_pipeline();
        let config = YamlPipelineConfig::from_str(yaml).unwrap();
        assert_eq!(config.config().pipeline.execution_mode, "batch");
        assert_eq!(config.config().pipeline.parallelism, 8);
        assert!(config.config().pipeline.checkpointing);
    }

    #[test]
    fn test_streaming_pipeline_template() {
        let yaml = ConfigTemplates::streaming_pipeline();
        let config = YamlPipelineConfig::from_str(yaml).unwrap();
        assert_eq!(config.config().pipeline.execution_mode, "streaming");
    }

    #[test]
    fn test_vector_tiles_pipeline_template() {
        let yaml = ConfigTemplates::vector_tiles_pipeline();
        let config = YamlPipelineConfig::from_str(yaml).unwrap();
        assert_eq!(config.config().pipeline.name, "vector-tiles-pipeline");
        assert!(config.config().sink.is_some());
    }
}
