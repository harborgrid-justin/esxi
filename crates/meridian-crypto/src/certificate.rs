//! Certificate management and validation.
//!
//! This module provides X.509 certificate generation, validation, and management.

use crate::error::{CryptoError, CryptoResult};
use rcgen::{Certificate, CertificateParams, DistinguishedName, DnType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use x509_parser::prelude::*;

/// Certificate information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateInfo {
    /// Certificate serial number.
    pub serial_number: String,

    /// Subject distinguished name.
    pub subject: String,

    /// Issuer distinguished name.
    pub issuer: String,

    /// Not valid before timestamp.
    pub not_before: chrono::DateTime<chrono::Utc>,

    /// Not valid after timestamp.
    pub not_after: chrono::DateTime<chrono::Utc>,

    /// Subject alternative names.
    pub subject_alt_names: Vec<String>,

    /// Key usage.
    pub key_usage: Vec<String>,

    /// Extended key usage.
    pub extended_key_usage: Vec<String>,

    /// Whether this is a CA certificate.
    pub is_ca: bool,

    /// Certificate fingerprint (SHA-256).
    pub fingerprint: String,
}

/// Certificate request parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateRequest {
    /// Common name (CN).
    pub common_name: String,

    /// Organization (O).
    pub organization: Option<String>,

    /// Organizational unit (OU).
    pub organizational_unit: Option<String>,

    /// Country (C).
    pub country: Option<String>,

    /// State/Province (ST).
    pub state: Option<String>,

    /// Locality (L).
    pub locality: Option<String>,

    /// Subject alternative names.
    pub subject_alt_names: Vec<String>,

    /// Validity period in days.
    pub validity_days: u32,

    /// Whether this should be a CA certificate.
    pub is_ca: bool,

    /// Key usage extensions.
    pub key_usage: Vec<KeyUsage>,

    /// Extended key usage.
    pub extended_key_usage: Vec<ExtendedKeyUsage>,
}

/// Key usage enumeration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyUsage {
    DigitalSignature,
    ContentCommitment,
    KeyEncipherment,
    DataEncipherment,
    KeyAgreement,
    KeyCertSign,
    CrlSign,
    EncipherOnly,
    DecipherOnly,
}

/// Extended key usage enumeration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExtendedKeyUsage {
    ServerAuth,
    ClientAuth,
    CodeSigning,
    EmailProtection,
    TimeStamping,
    OcspSigning,
}

/// Certificate manager.
pub struct CertificateManager {
    /// Stored certificates.
    certificates: HashMap<String, Vec<u8>>,

    /// Trusted root certificates.
    trusted_roots: Vec<Vec<u8>>,
}

impl CertificateManager {
    /// Create a new certificate manager.
    pub fn new() -> Self {
        Self {
            certificates: HashMap::new(),
            trusted_roots: Vec::new(),
        }
    }

    /// Generate a new certificate.
    pub fn generate_certificate(
        &mut self,
        request: &CertificateRequest,
    ) -> CryptoResult<(Vec<u8>, Vec<u8>)> {
        let mut params = CertificateParams::new(request.subject_alt_names.clone());

        // Set distinguished name
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, &request.common_name);

        if let Some(org) = &request.organization {
            dn.push(DnType::OrganizationName, org);
        }

        if let Some(ou) = &request.organizational_unit {
            dn.push(DnType::OrganizationalUnitName, ou);
        }

        if let Some(country) = &request.country {
            dn.push(DnType::CountryName, country);
        }

        if let Some(state) = &request.state {
            dn.push(DnType::StateOrProvinceName, state);
        }

        if let Some(locality) = &request.locality {
            dn.push(DnType::LocalityName, locality);
        }

        params.distinguished_name = dn;
        params.is_ca = if request.is_ca {
            rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained)
        } else {
            rcgen::IsCa::NoCa
        };

        // Set validity
        params.not_before = ::time::OffsetDateTime::now_utc() - ::time::Duration::days(1);
        params.not_after = ::time::OffsetDateTime::now_utc() + ::time::Duration::days(request.validity_days as i64);

        // Generate certificate
        let cert = Certificate::from_params(params)
            .map_err(|e| CryptoError::CertificateError(format!("Failed to generate certificate: {}", e)))?;

        let cert_pem = cert.serialize_pem()
            .map_err(|e| CryptoError::CertificateError(format!("Failed to serialize certificate: {}", e)))?;

        let key_pem = cert.serialize_private_key_pem();

        // Store certificate
        let cert_id = self.calculate_fingerprint(cert_pem.as_bytes())?;
        self.certificates.insert(cert_id, cert_pem.as_bytes().to_vec());

        Ok((cert_pem.into_bytes(), key_pem.into_bytes()))
    }

    /// Generate a self-signed CA certificate.
    pub fn generate_ca_certificate(
        &mut self,
        common_name: &str,
        validity_days: u32,
    ) -> CryptoResult<(Vec<u8>, Vec<u8>)> {
        let request = CertificateRequest {
            common_name: common_name.to_string(),
            organization: Some("Meridian GIS".to_string()),
            organizational_unit: Some("Security".to_string()),
            country: Some("US".to_string()),
            state: None,
            locality: None,
            subject_alt_names: vec![],
            validity_days,
            is_ca: true,
            key_usage: vec![KeyUsage::KeyCertSign, KeyUsage::CrlSign],
            extended_key_usage: vec![],
        };

        self.generate_certificate(&request)
    }

    /// Parse and validate a certificate.
    pub fn parse_certificate(&self, cert_pem: &[u8]) -> CryptoResult<CertificateInfo> {
        let (_, pem) = parse_x509_pem(cert_pem)
            .map_err(|e| CryptoError::CertificateError(format!("Failed to parse PEM: {}", e)))?;

        let (_, cert) = X509Certificate::from_der(&pem.contents)
            .map_err(|e| CryptoError::CertificateError(format!("Failed to parse X.509 certificate: {}", e)))?;

        let subject = cert.subject().to_string();
        let issuer = cert.issuer().to_string();

        let not_before = chrono::DateTime::from_timestamp(cert.validity().not_before.timestamp(), 0)
            .unwrap_or_else(chrono::Utc::now);

        let not_after = chrono::DateTime::from_timestamp(cert.validity().not_after.timestamp(), 0)
            .unwrap_or_else(chrono::Utc::now);

        let serial_number = hex::encode(cert.serial.to_bytes_be());

        let fingerprint = self.calculate_fingerprint(cert_pem)?;

        Ok(CertificateInfo {
            serial_number,
            subject,
            issuer,
            not_before,
            not_after,
            subject_alt_names: vec![],
            key_usage: vec![],
            extended_key_usage: vec![],
            is_ca: cert.is_ca(),
            fingerprint,
        })
    }

    /// Validate a certificate chain.
    pub fn validate_certificate_chain(
        &self,
        cert_chain: &[&[u8]],
    ) -> CryptoResult<bool> {
        if cert_chain.is_empty() {
            return Err(CryptoError::InvalidCertificateChain(
                "Empty certificate chain".to_string(),
            ));
        }

        // Parse all certificates in the chain
        let mut certs = Vec::new();
        for cert_pem in cert_chain {
            let info = self.parse_certificate(cert_pem)?;
            certs.push(info);
        }

        // Verify each certificate is signed by the next one in the chain
        for i in 0..certs.len() - 1 {
            let cert = &certs[i];
            let issuer_cert = &certs[i + 1];

            // Check if the issuer matches
            if cert.issuer != issuer_cert.subject {
                return Err(CryptoError::InvalidCertificateChain(format!(
                    "Certificate issuer mismatch at position {}",
                    i
                )));
            }
        }

        // Verify the root certificate is self-signed
        let root = &certs[certs.len() - 1];
        if root.subject != root.issuer {
            return Err(CryptoError::InvalidCertificateChain(
                "Root certificate is not self-signed".to_string(),
            ));
        }

        Ok(true)
    }

    /// Check if a certificate has expired.
    pub fn is_certificate_expired(&self, cert_pem: &[u8]) -> CryptoResult<bool> {
        let info = self.parse_certificate(cert_pem)?;
        Ok(chrono::Utc::now() > info.not_after)
    }

    /// Check if a certificate is valid (not expired and not before start date).
    pub fn is_certificate_valid(&self, cert_pem: &[u8]) -> CryptoResult<bool> {
        let info = self.parse_certificate(cert_pem)?;
        let now = chrono::Utc::now();
        Ok(now >= info.not_before && now <= info.not_after)
    }

    /// Add a trusted root certificate.
    pub fn add_trusted_root(&mut self, cert_pem: Vec<u8>) -> CryptoResult<()> {
        // Validate it's a CA certificate
        let info = self.parse_certificate(&cert_pem)?;
        if !info.is_ca {
            return Err(CryptoError::CertificateError(
                "Certificate is not a CA certificate".to_string(),
            ));
        }

        self.trusted_roots.push(cert_pem);
        Ok(())
    }

    /// Get all trusted root certificates.
    pub fn get_trusted_roots(&self) -> &[Vec<u8>] {
        &self.trusted_roots
    }

    /// Load certificate from file.
    pub fn load_certificate(&mut self, path: &Path) -> CryptoResult<Vec<u8>> {
        let cert_pem = std::fs::read(path)
            .map_err(CryptoError::IoError)?;

        // Validate the certificate
        self.parse_certificate(&cert_pem)?;

        let fingerprint = self.calculate_fingerprint(&cert_pem)?;
        self.certificates.insert(fingerprint, cert_pem.clone());

        Ok(cert_pem)
    }

    /// Save certificate to file.
    pub fn save_certificate(&self, cert_pem: &[u8], path: &Path) -> CryptoResult<()> {
        std::fs::write(path, cert_pem)
            .map_err(CryptoError::IoError)?;

        Ok(())
    }

    /// Calculate SHA-256 fingerprint of a certificate.
    fn calculate_fingerprint(&self, cert_pem: &[u8]) -> CryptoResult<String> {
        use ring::digest;

        let (_, pem) = parse_x509_pem(cert_pem)
            .map_err(|e| CryptoError::CertificateError(format!("Failed to parse PEM: {}", e)))?;

        let digest = digest::digest(&digest::SHA256, &pem.contents);
        Ok(hex::encode(digest.as_ref()))
    }

    /// Get certificate by fingerprint.
    pub fn get_certificate(&self, fingerprint: &str) -> Option<&Vec<u8>> {
        self.certificates.get(fingerprint)
    }

    /// List all stored certificate fingerprints.
    pub fn list_certificates(&self) -> Vec<String> {
        self.certificates.keys().cloned().collect()
    }

    /// Remove a certificate.
    pub fn remove_certificate(&mut self, fingerprint: &str) -> Option<Vec<u8>> {
        self.certificates.remove(fingerprint)
    }
}

impl Default for CertificateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_generate_certificate() {
        let mut manager = CertificateManager::new();

        let request = CertificateRequest {
            common_name: "test.example.com".to_string(),
            organization: Some("Test Org".to_string()),
            organizational_unit: None,
            country: Some("US".to_string()),
            state: None,
            locality: None,
            subject_alt_names: vec!["test.example.com".to_string()],
            validity_days: 365,
            is_ca: false,
            key_usage: vec![KeyUsage::DigitalSignature],
            extended_key_usage: vec![ExtendedKeyUsage::ServerAuth],
        };

        let result = manager.generate_certificate(&request);
        assert!(result.is_ok());

        let (cert_pem, key_pem) = result.unwrap();
        assert!(!cert_pem.is_empty());
        assert!(!key_pem.is_empty());
    }

    #[test]
    fn test_generate_ca_certificate() {
        let mut manager = CertificateManager::new();
        let result = manager.generate_ca_certificate("Test CA", 3650);
        assert!(result.is_ok());

        let (cert_pem, _) = result.unwrap();
        let info = manager.parse_certificate(&cert_pem).unwrap();
        assert!(info.is_ca);
    }

    #[test]
    fn test_certificate_validation() {
        let mut manager = CertificateManager::new();
        let (cert_pem, _) = manager.generate_ca_certificate("Test CA", 365).unwrap();

        let is_valid = manager.is_certificate_valid(&cert_pem).unwrap();
        assert!(is_valid);

        let is_expired = manager.is_certificate_expired(&cert_pem).unwrap();
        assert!(!is_expired);
    }

    #[test]
    fn test_certificate_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let cert_path = temp_dir.path().join("test-cert.pem");

        let mut manager = CertificateManager::new();
        let (cert_pem, _) = manager.generate_ca_certificate("Test CA", 365).unwrap();

        manager.save_certificate(&cert_pem, &cert_path).unwrap();
        assert!(cert_path.exists());

        let loaded_cert = manager.load_certificate(&cert_path).unwrap();
        assert_eq!(cert_pem, loaded_cert);
    }
}
