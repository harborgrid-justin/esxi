# MERIDIAN-BACKUP v0.1.5

Enterprise-grade disaster recovery and backup system for the Meridian GIS Platform.

## Overview

MERIDIAN-BACKUP provides a comprehensive backup and disaster recovery solution with enterprise features including incremental backups, point-in-time recovery, multi-region replication, and automated failover capabilities.

## Features

### Core Backup Features
- **Incremental and Differential Backups**: Minimize storage and bandwidth with efficient backup strategies
- **Point-in-Time Recovery (PITR)**: Restore to any point in time with Write-Ahead Log (WAL) support
- **Backup Verification**: Automated integrity checking with multiple verification levels (Quick, Standard, Deep)
- **Compression & Deduplication**: Support for Gzip, Zstd, and LZ4 compression with content-defined chunking

### Storage & Replication
- **Multi-Cloud Support**:
  - AWS S3
  - Google Cloud Storage (GCS)
  - Azure Blob Storage
- **Multi-Region Replication**: Asynchronous, synchronous, and quorum-based replication strategies
- **Encrypted Storage**: AES-256-GCM encryption for data at rest

### Disaster Recovery
- **Automated Failover**: Health-check based automatic failover and failback
- **DR Runbooks**: Automated and manual recovery procedures with step tracking
- **RTO/RPO Monitoring**: Track Recovery Time Objective and Recovery Point Objective
- **SLA Compliance**: Monitor and report on backup SLA violations

### Automation
- **Backup Scheduling**: Cron-based scheduling with templates (hourly, daily, weekly, monthly)
- **Retention Policies**:
  - Time-based (delete after N days)
  - Count-based (keep last N backups)
  - GFS (Grandfather-Father-Son) rotation
- **Automated Scrubbing**: Periodic verification of all backups

## Architecture

```
meridian-backup/
├── src/
│   ├── backup.rs           # Main orchestration
│   ├── compression.rs      # Compression & deduplication
│   ├── encryption.rs       # AES-256-GCM encryption
│   ├── error.rs            # Error types
│   ├── failover.rs         # Automated failover
│   ├── incremental.rs      # Incremental backups
│   ├── pitr.rs             # Point-in-time recovery
│   ├── replication.rs      # Multi-region replication
│   ├── retention.rs        # Retention policies
│   ├── rto.rs              # RTO/RPO monitoring
│   ├── runbook.rs          # DR runbooks
│   ├── scheduler.rs        # Backup scheduling
│   ├── verification.rs     # Integrity verification
│   └── storage/
│       ├── mod.rs          # Storage abstraction
│       ├── s3.rs           # AWS S3 backend
│       ├── gcs.rs          # Google Cloud Storage
│       └── azure.rs        # Azure Blob Storage
└── Cargo.toml
```

## Usage Examples

### Basic Backup

```rust
use meridian_backup::prelude::*;
use bytes::Bytes;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // Configure backup system
    let config = BackupConfig::default();

    // Create orchestrator
    let orchestrator = BackupOrchestrator::new(config).await?;

    // Create a full backup
    let files = vec![
        (PathBuf::from("data/file1.dat"), Bytes::from("data1")),
        (PathBuf::from("data/file2.dat"), Bytes::from("data2")),
    ];

    let manifest = orchestrator.create_full_backup(files).await?;
    println!("Backup created: {}", manifest.id);

    Ok(())
}
```

### Incremental Backup

```rust
// Create incremental backup based on previous full backup
let files_changed = vec![
    (PathBuf::from("data/file2.dat"), Bytes::from("updated_data2")),
    (PathBuf::from("data/file3.dat"), Bytes::from("new_file")),
];

let incremental_manifest = orchestrator
    .create_incremental_backup(manifest.id, files_changed)
    .await?;
```

### Point-in-Time Recovery

```rust
use chrono::Utc;

// Restore to a specific point in time
let target_time = Utc::now() - chrono::Duration::hours(2);
let restored_files = orchestrator
    .restore_to_point(target_time, None)
    .await?;

println!("Restored {} files", restored_files.len());
```

### Backup Verification

```rust
use meridian_backup::verification::VerificationLevel;

// Verify backup integrity
let report = orchestrator
    .verify_backup(manifest.id, VerificationLevel::Deep)
    .await?;

println!("Verification: {:?}", report.overall_status);
println!("Passed: {}, Failed: {}", report.passed, report.failed);
```

### Scheduled Backups

```rust
use meridian_backup::scheduler::{BackupSchedule, ScheduleTemplates};
use uuid::Uuid;

let scheduler = orchestrator.scheduler();

// Create daily backup schedule
let schedule = BackupSchedule {
    id: Uuid::new_v4(),
    name: "Daily Backups".to_string(),
    cron_expression: ScheduleTemplates::daily().to_string(),
    backup_type: BackupType::Incremental,
    enabled: true,
    retention_days: 30,
    tags: vec!["production".to_string()],
    notification_email: Some("admin@example.com".to_string()),
};

scheduler.add_schedule(schedule).await?;
scheduler.start().await;
```

### Retention Policies

```rust
use meridian_backup::retention::{RetentionPolicy, RetentionPolicyType};

// Apply GFS retention policy
let gfs_policy = RetentionPolicy {
    id: Uuid::new_v4(),
    name: "GFS Standard".to_string(),
    policy_type: RetentionPolicyType::Gfs {
        daily: 7,
        weekly: 4,
        monthly: 12,
        yearly: 5,
    },
    min_backups_to_keep: 1,
    max_backups_to_keep: None,
    enabled: true,
};

orchestrator.apply_retention_policy(gfs_policy, backups).await?;
```

### Failover Management

```rust
use meridian_backup::failover::{FailoverConfig, FailoverManager};

let failover_manager = orchestrator.failover_manager();
let mut manager = failover_manager.write().await;

// Configure failover
let config = FailoverConfig {
    id: Uuid::new_v4(),
    name: "US-EU Failover".to_string(),
    primary_region: "us-east-1".to_string(),
    secondary_regions: vec!["eu-west-1".to_string()],
    auto_failover_enabled: true,
    auto_failback_enabled: true,
    health_check_interval_secs: 30,
    failover_threshold: 3,
    failback_threshold: 5,
};

manager.add_config(config);

// Initiate manual failover
let event_id = manager.initiate_failover(config.id, None).await?;
println!("Failover initiated: {}", event_id);
```

### RTO/RPO Monitoring

```rust
use meridian_backup::rto::{SlaConfig, RtoMonitor};

let rto_monitor = orchestrator.rto_monitor();
let mut monitor = rto_monitor.write().await;

// Configure SLA
let sla = SlaConfig {
    id: Uuid::new_v4(),
    name: "Production SLA".to_string(),
    rto_minutes: 15,  // 15 minute recovery time
    rpo_minutes: 60,  // 1 hour data loss tolerance
    enabled: true,
};

monitor.add_sla(sla);

// Get statistics
let stats = monitor.statistics();
println!("SLA Compliance: {:.2}%", stats.compliance_rate);
```

## Configuration

### Storage Configuration

```rust
use meridian_backup::storage::StorageConfig;

// AWS S3
let s3_config = StorageConfig::S3 {
    region: "us-east-1".to_string(),
    bucket: "my-backups".to_string(),
    endpoint: None,
    access_key: Some(std::env::var("AWS_ACCESS_KEY_ID").ok()),
    secret_key: Some(std::env::var("AWS_SECRET_ACCESS_KEY").ok()),
};

// Google Cloud Storage
let gcs_config = StorageConfig::Gcs {
    project_id: "my-project".to_string(),
    bucket: "my-backups".to_string(),
    credentials_path: Some("/path/to/credentials.json".to_string()),
};

// Azure Blob Storage
let azure_config = StorageConfig::Azure {
    account_name: "myaccount".to_string(),
    container: "my-backups".to_string(),
    access_key: Some(std::env::var("AZURE_STORAGE_KEY").ok()),
    connection_string: None,
};
```

### Compression Configuration

```rust
use meridian_backup::compression::{CompressionAlgorithm, CompressionLevel};

let config = BackupConfig {
    compression_algorithm: CompressionAlgorithm::Zstd,
    compression_level: CompressionLevel::Best,
    ..Default::default()
};
```

### Encryption Configuration

```rust
use meridian_backup::encryption::EncryptionConfig;

let config = BackupConfig {
    encryption_enabled: true,
    encryption_config: Some(EncryptionConfig::default()),
    ..Default::default()
};
```

## Disaster Recovery Runbooks

```rust
use meridian_backup::runbook::{Runbook, RunbookTemplates};

let runbook_manager = orchestrator.runbook_manager();
let mut manager = runbook_manager.write().await;

// Add pre-defined datacenter failover runbook
let dc_failover = RunbookTemplates::datacenter_failover();
manager.add_runbook(dc_failover.clone());

// Execute runbook
let execution_id = manager.start_execution(dc_failover.id)?;

// Execute steps
while let Some(result) = manager.execute_next_step(execution_id).await? {
    println!("Step completed: {:?}", result.status);
}
```

## Performance

- **Compression Ratios**:
  - Zstd: ~3-5x for typical data
  - LZ4: ~2-3x (faster)
  - Gzip: ~3-4x (balanced)

- **Deduplication**: Content-defined chunking with configurable chunk sizes (4KB-1MB)

- **Encryption Overhead**: ~5-10% with AES-256-GCM hardware acceleration

## Best Practices

1. **Full Backups**: Schedule weekly full backups as the baseline
2. **Incremental Backups**: Daily incrementals to minimize storage and transfer
3. **Verification**: Run deep verification weekly on random backups
4. **Retention**: Use GFS policy for balanced retention
5. **Replication**: Enable multi-region for critical data
6. **Monitoring**: Set up RTO/RPO alerts for SLA violations
7. **Testing**: Regularly test disaster recovery procedures

## Dependencies

Key dependencies include:
- `aws-sdk-s3`: AWS S3 integration
- `google-cloud-storage`: GCS integration
- `azure_storage_blobs`: Azure Blob integration
- `zstd`, `lz4`, `flate2`: Compression algorithms
- `aes-gcm`: Encryption
- `tokio`: Async runtime
- `chrono`: Time handling
- `cron`: Schedule parsing

## License

MIT OR Apache-2.0

## Contributing

This is an enterprise backup system for the Meridian GIS Platform. Contributions should focus on:
- Additional storage backends
- Enhanced compression algorithms
- Improved deduplication strategies
- Extended DR runbook templates
- Performance optimizations

## Version

Current version: 0.1.5

## Support

For enterprise support and consulting, contact the Meridian team.
