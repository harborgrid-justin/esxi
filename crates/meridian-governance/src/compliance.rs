//! Compliance framework management (GDPR, CCPA, SOC2)

use crate::error::{GovernanceError, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Compliance framework manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceManager {
    /// Registered compliance frameworks
    frameworks: HashMap<String, ComplianceFramework>,
    /// Compliance assessments
    assessments: HashMap<String, ComplianceAssessment>,
    /// Data subject requests (GDPR, CCPA)
    subject_requests: HashMap<String, DataSubjectRequest>,
    /// Consent records
    consent_records: HashMap<String, ConsentRecord>,
}

/// Compliance framework definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFramework {
    /// Framework identifier
    pub id: String,
    /// Framework name
    pub name: String,
    /// Framework description
    pub description: String,
    /// Framework type
    pub framework_type: FrameworkType,
    /// Required controls
    pub controls: Vec<ComplianceControl>,
    /// Applicable regions/jurisdictions
    pub jurisdictions: Vec<String>,
    /// Enabled status
    pub enabled: bool,
}

/// Framework type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FrameworkType {
    /// General Data Protection Regulation (EU)
    Gdpr,
    /// California Consumer Privacy Act
    Ccpa,
    /// Service Organization Control 2
    Soc2,
    /// Health Insurance Portability and Accountability Act
    Hipaa,
    /// Payment Card Industry Data Security Standard
    PciDss,
    /// Custom framework
    Custom(String),
}

/// Compliance control requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceControl {
    /// Control identifier
    pub id: String,
    /// Control name
    pub name: String,
    /// Control description
    pub description: String,
    /// Control category
    pub category: ControlCategory,
    /// Implementation status
    pub status: ControlStatus,
    /// Responsible party
    pub owner: Option<String>,
    /// Evidence/documentation
    pub evidence: Vec<String>,
    /// Last assessed date
    pub last_assessed: Option<DateTime<Utc>>,
    /// Next review date
    pub next_review: Option<DateTime<Utc>>,
}

/// Control category
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ControlCategory {
    DataProtection,
    AccessControl,
    Encryption,
    AuditLogging,
    DataRetention,
    DataDeletion,
    ConsentManagement,
    DataPortability,
    BreachNotification,
    PrivacyByDesign,
    DataMinimization,
    PurposeLimitation,
    TransparencyAndNotice,
    DataSubjectRights,
    VendorManagement,
    Custom(String),
}

/// Control implementation status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ControlStatus {
    NotImplemented,
    InProgress,
    Implemented,
    Verified,
    NonCompliant,
}

/// Compliance assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceAssessment {
    /// Assessment identifier
    pub id: String,
    /// Framework being assessed
    pub framework_id: String,
    /// Assessment date
    pub assessed_at: DateTime<Utc>,
    /// Assessor
    pub assessor: String,
    /// Overall compliance status
    pub status: ComplianceStatus,
    /// Control results
    pub control_results: HashMap<String, ControlResult>,
    /// Findings
    pub findings: Vec<Finding>,
    /// Remediation plan
    pub remediation_plan: Option<String>,
    /// Next assessment date
    pub next_assessment: Option<DateTime<Utc>>,
}

/// Compliance status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ComplianceStatus {
    Compliant,
    PartiallyCompliant,
    NonCompliant,
    NotAssessed,
}

/// Control assessment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlResult {
    /// Control ID
    pub control_id: String,
    /// Pass/fail status
    pub passed: bool,
    /// Score (0.0 - 1.0)
    pub score: f64,
    /// Comments
    pub comments: Option<String>,
    /// Evidence reviewed
    pub evidence_reviewed: Vec<String>,
}

/// Compliance finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Finding identifier
    pub id: String,
    /// Severity
    pub severity: FindingSeverity,
    /// Control ID
    pub control_id: String,
    /// Description
    pub description: String,
    /// Recommendation
    pub recommendation: String,
    /// Status
    pub status: FindingStatus,
    /// Due date for remediation
    pub due_date: Option<DateTime<Utc>>,
}

/// Finding severity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum FindingSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Finding status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FindingStatus {
    Open,
    InProgress,
    Resolved,
    Accepted,
    Rejected,
}

/// Data subject request (GDPR Article 15-22, CCPA)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSubjectRequest {
    /// Request identifier
    pub id: String,
    /// Request type
    pub request_type: RequestType,
    /// Subject identifier
    pub subject_id: String,
    /// Subject email
    pub subject_email: String,
    /// Request date
    pub requested_at: DateTime<Utc>,
    /// Request status
    pub status: RequestStatus,
    /// Deadline for completion (30 days for GDPR)
    pub deadline: DateTime<Utc>,
    /// Completed date
    pub completed_at: Option<DateTime<Utc>>,
    /// Assigned to
    pub assigned_to: Option<String>,
    /// Response/notes
    pub response: Option<String>,
    /// Affected datasets
    pub affected_datasets: Vec<String>,
}

/// Data subject request type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RequestType {
    /// GDPR Article 15 - Right to access
    Access,
    /// GDPR Article 16 - Right to rectification
    Rectification,
    /// GDPR Article 17 - Right to erasure ("right to be forgotten")
    Erasure,
    /// GDPR Article 18 - Right to restriction of processing
    Restriction,
    /// GDPR Article 20 - Right to data portability
    Portability,
    /// GDPR Article 21 - Right to object
    Objection,
    /// GDPR Article 22 - Rights related to automated decision making
    AutomatedDecision,
    /// CCPA - Do not sell my personal information
    DoNotSell,
    /// CCPA - Opt-out of sale
    OptOut,
}

/// Request processing status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RequestStatus {
    Received,
    InProgress,
    Completed,
    Rejected,
    Expired,
}

/// Consent record for data processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRecord {
    /// Consent identifier
    pub id: String,
    /// Subject identifier
    pub subject_id: String,
    /// Purpose of processing
    pub purpose: String,
    /// Consent given
    pub granted: bool,
    /// Consent timestamp
    pub timestamp: DateTime<Utc>,
    /// Expiration date
    pub expires_at: Option<DateTime<Utc>>,
    /// Consent withdrawn
    pub withdrawn: bool,
    /// Withdrawal timestamp
    pub withdrawn_at: Option<DateTime<Utc>>,
    /// Processing categories
    pub categories: Vec<String>,
    /// Legal basis
    pub legal_basis: LegalBasis,
    /// Consent method (e.g., "web form", "email", "paper")
    pub method: String,
}

/// Legal basis for processing (GDPR Article 6)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LegalBasis {
    /// Consent of the data subject
    Consent,
    /// Performance of a contract
    Contract,
    /// Compliance with legal obligation
    LegalObligation,
    /// Protection of vital interests
    VitalInterests,
    /// Public interest or official authority
    PublicInterest,
    /// Legitimate interests
    LegitimateInterests,
}

impl ComplianceManager {
    /// Create a new compliance manager
    pub fn new() -> Self {
        let mut manager = Self {
            frameworks: HashMap::new(),
            assessments: HashMap::new(),
            subject_requests: HashMap::new(),
            consent_records: HashMap::new(),
        };

        // Initialize default frameworks
        manager.initialize_default_frameworks();

        manager
    }

    /// Initialize default compliance frameworks
    fn initialize_default_frameworks(&mut self) {
        // GDPR Framework
        let gdpr = ComplianceFramework {
            id: "gdpr".to_string(),
            name: "GDPR".to_string(),
            description: "General Data Protection Regulation".to_string(),
            framework_type: FrameworkType::Gdpr,
            controls: Self::create_gdpr_controls(),
            jurisdictions: vec!["EU".to_string(), "EEA".to_string()],
            enabled: true,
        };

        // CCPA Framework
        let ccpa = ComplianceFramework {
            id: "ccpa".to_string(),
            name: "CCPA".to_string(),
            description: "California Consumer Privacy Act".to_string(),
            framework_type: FrameworkType::Ccpa,
            controls: Self::create_ccpa_controls(),
            jurisdictions: vec!["California".to_string()],
            enabled: true,
        };

        // SOC2 Framework
        let soc2 = ComplianceFramework {
            id: "soc2".to_string(),
            name: "SOC 2".to_string(),
            description: "Service Organization Control 2".to_string(),
            framework_type: FrameworkType::Soc2,
            controls: Self::create_soc2_controls(),
            jurisdictions: vec!["Global".to_string()],
            enabled: true,
        };

        self.frameworks.insert(gdpr.id.clone(), gdpr);
        self.frameworks.insert(ccpa.id.clone(), ccpa);
        self.frameworks.insert(soc2.id.clone(), soc2);
    }

    /// Create GDPR controls
    fn create_gdpr_controls() -> Vec<ComplianceControl> {
        vec![
            ComplianceControl {
                id: "gdpr_art_5_1_a".to_string(),
                name: "Lawfulness, fairness and transparency".to_string(),
                description: "Process personal data lawfully, fairly and transparently".to_string(),
                category: ControlCategory::TransparencyAndNotice,
                status: ControlStatus::NotImplemented,
                owner: None,
                evidence: Vec::new(),
                last_assessed: None,
                next_review: None,
            },
            ComplianceControl {
                id: "gdpr_art_5_1_b".to_string(),
                name: "Purpose limitation".to_string(),
                description: "Collect data for specified, explicit and legitimate purposes".to_string(),
                category: ControlCategory::PurposeLimitation,
                status: ControlStatus::NotImplemented,
                owner: None,
                evidence: Vec::new(),
                last_assessed: None,
                next_review: None,
            },
            ComplianceControl {
                id: "gdpr_art_5_1_c".to_string(),
                name: "Data minimization".to_string(),
                description: "Collect only adequate, relevant and limited data".to_string(),
                category: ControlCategory::DataMinimization,
                status: ControlStatus::NotImplemented,
                owner: None,
                evidence: Vec::new(),
                last_assessed: None,
                next_review: None,
            },
            ComplianceControl {
                id: "gdpr_art_32".to_string(),
                name: "Security of processing".to_string(),
                description: "Implement appropriate technical and organizational measures".to_string(),
                category: ControlCategory::DataProtection,
                status: ControlStatus::NotImplemented,
                owner: None,
                evidence: Vec::new(),
                last_assessed: None,
                next_review: None,
            },
        ]
    }

    /// Create CCPA controls
    fn create_ccpa_controls() -> Vec<ComplianceControl> {
        vec![
            ComplianceControl {
                id: "ccpa_1798_100".to_string(),
                name: "Notice at collection".to_string(),
                description: "Inform consumers about data collection".to_string(),
                category: ControlCategory::TransparencyAndNotice,
                status: ControlStatus::NotImplemented,
                owner: None,
                evidence: Vec::new(),
                last_assessed: None,
                next_review: None,
            },
            ComplianceControl {
                id: "ccpa_1798_105".to_string(),
                name: "Right to know".to_string(),
                description: "Disclose personal information collected and sold".to_string(),
                category: ControlCategory::DataSubjectRights,
                status: ControlStatus::NotImplemented,
                owner: None,
                evidence: Vec::new(),
                last_assessed: None,
                next_review: None,
            },
            ComplianceControl {
                id: "ccpa_1798_120".to_string(),
                name: "Right to opt-out".to_string(),
                description: "Allow consumers to opt-out of sale of personal information".to_string(),
                category: ControlCategory::ConsentManagement,
                status: ControlStatus::NotImplemented,
                owner: None,
                evidence: Vec::new(),
                last_assessed: None,
                next_review: None,
            },
        ]
    }

    /// Create SOC2 controls
    fn create_soc2_controls() -> Vec<ComplianceControl> {
        vec![
            ComplianceControl {
                id: "soc2_cc6_1".to_string(),
                name: "Logical and physical access controls".to_string(),
                description: "Implement access controls to protect information".to_string(),
                category: ControlCategory::AccessControl,
                status: ControlStatus::NotImplemented,
                owner: None,
                evidence: Vec::new(),
                last_assessed: None,
                next_review: None,
            },
            ComplianceControl {
                id: "soc2_cc6_7".to_string(),
                name: "Data encryption".to_string(),
                description: "Encrypt sensitive data in transit and at rest".to_string(),
                category: ControlCategory::Encryption,
                status: ControlStatus::NotImplemented,
                owner: None,
                evidence: Vec::new(),
                last_assessed: None,
                next_review: None,
            },
            ComplianceControl {
                id: "soc2_cc7_2".to_string(),
                name: "System monitoring".to_string(),
                description: "Monitor system activities and generate audit logs".to_string(),
                category: ControlCategory::AuditLogging,
                status: ControlStatus::NotImplemented,
                owner: None,
                evidence: Vec::new(),
                last_assessed: None,
                next_review: None,
            },
        ]
    }

    /// Register a compliance framework
    pub fn add_framework(&mut self, framework: ComplianceFramework) -> Result<()> {
        if self.frameworks.contains_key(&framework.id) {
            return Err(GovernanceError::Compliance(format!(
                "Framework already exists: {}",
                framework.id
            )));
        }

        self.frameworks.insert(framework.id.clone(), framework);
        Ok(())
    }

    /// Get a compliance framework
    pub fn get_framework(&self, framework_id: &str) -> Result<&ComplianceFramework> {
        self.frameworks.get(framework_id).ok_or_else(|| {
            GovernanceError::Compliance(format!("Framework not found: {}", framework_id))
        })
    }

    /// Submit a data subject request
    pub fn submit_request(&mut self, mut request: DataSubjectRequest) -> Result<String> {
        request.id = Uuid::new_v4().to_string();
        request.status = RequestStatus::Received;

        // Set deadline based on request type (30 days for GDPR)
        if request.deadline == DateTime::<Utc>::MIN_UTC {
            request.deadline = Utc::now() + Duration::days(30);
        }

        let id = request.id.clone();
        self.subject_requests.insert(id.clone(), request);
        Ok(id)
    }

    /// Get a data subject request
    pub fn get_request(&self, request_id: &str) -> Result<&DataSubjectRequest> {
        self.subject_requests.get(request_id).ok_or_else(|| {
            GovernanceError::Compliance(format!("Request not found: {}", request_id))
        })
    }

    /// Update request status
    pub fn update_request_status(
        &mut self,
        request_id: &str,
        status: RequestStatus,
    ) -> Result<()> {
        let request = self.subject_requests.get_mut(request_id).ok_or_else(|| {
            GovernanceError::Compliance(format!("Request not found: {}", request_id))
        })?;

        request.status = status.clone();

        if status == RequestStatus::Completed {
            request.completed_at = Some(Utc::now());
        }

        Ok(())
    }

    /// Record consent
    pub fn record_consent(&mut self, consent: ConsentRecord) -> Result<String> {
        let id = consent.id.clone();
        self.consent_records.insert(id.clone(), consent);
        Ok(id)
    }

    /// Withdraw consent
    pub fn withdraw_consent(&mut self, consent_id: &str) -> Result<()> {
        let consent = self.consent_records.get_mut(consent_id).ok_or_else(|| {
            GovernanceError::Compliance(format!("Consent record not found: {}", consent_id))
        })?;

        consent.withdrawn = true;
        consent.withdrawn_at = Some(Utc::now());

        Ok(())
    }

    /// Check if consent is valid
    pub fn is_consent_valid(&self, consent_id: &str) -> Result<bool> {
        let consent = self.consent_records.get(consent_id).ok_or_else(|| {
            GovernanceError::Compliance(format!("Consent record not found: {}", consent_id))
        })?;

        Ok(consent.granted
            && !consent.withdrawn
            && consent
                .expires_at
                .map(|exp| exp > Utc::now())
                .unwrap_or(true))
    }

    /// List all frameworks
    pub fn list_frameworks(&self) -> Vec<&ComplianceFramework> {
        self.frameworks.values().collect()
    }

    /// List pending data subject requests
    pub fn list_pending_requests(&self) -> Vec<&DataSubjectRequest> {
        self.subject_requests
            .values()
            .filter(|r| {
                r.status == RequestStatus::Received || r.status == RequestStatus::InProgress
            })
            .collect()
    }

    /// List overdue requests
    pub fn list_overdue_requests(&self) -> Vec<&DataSubjectRequest> {
        let now = Utc::now();
        self.subject_requests
            .values()
            .filter(|r| {
                (r.status == RequestStatus::Received || r.status == RequestStatus::InProgress)
                    && r.deadline < now
            })
            .collect()
    }
}

impl Default for ComplianceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compliance_manager_creation() {
        let manager = ComplianceManager::new();
        assert_eq!(manager.list_frameworks().len(), 3);
    }

    #[test]
    fn test_submit_request() {
        let mut manager = ComplianceManager::new();
        let request = DataSubjectRequest {
            id: String::new(),
            request_type: RequestType::Access,
            subject_id: "user123".to_string(),
            subject_email: "user@example.com".to_string(),
            requested_at: Utc::now(),
            status: RequestStatus::Received,
            deadline: DateTime::<Utc>::MIN_UTC,
            completed_at: None,
            assigned_to: None,
            response: None,
            affected_datasets: vec![],
        };

        let id = manager.submit_request(request).unwrap();
        assert!(!id.is_empty());
    }
}
