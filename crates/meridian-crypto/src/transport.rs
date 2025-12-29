//! Transport layer security and encryption in transit.
//!
//! This module provides TLS configuration and management for secure communication.

use crate::error::{CryptoError, CryptoResult};
use rustls::{ClientConfig, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;

/// TLS configuration builder.
pub struct TlsConfigBuilder {
    /// Server certificate path.
    cert_path: Option<String>,

    /// Private key path.
    key_path: Option<String>,

    /// CA certificate path for client verification.
    ca_cert_path: Option<String>,

    /// Require client authentication.
    require_client_auth: bool,

    /// Supported TLS versions.
    min_tls_version: TlsVersion,

    /// Supported cipher suites.
    cipher_suites: Vec<String>,
}

/// TLS version enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsVersion {
    Tls12,
    Tls13,
}

impl TlsConfigBuilder {
    /// Create a new TLS configuration builder.
    pub fn new() -> Self {
        Self {
            cert_path: None,
            key_path: None,
            ca_cert_path: None,
            require_client_auth: false,
            min_tls_version: TlsVersion::Tls12,
            cipher_suites: Vec::new(),
        }
    }

    /// Set the server certificate path.
    pub fn with_cert(mut self, path: impl Into<String>) -> Self {
        self.cert_path = Some(path.into());
        self
    }

    /// Set the private key path.
    pub fn with_key(mut self, path: impl Into<String>) -> Self {
        self.key_path = Some(path.into());
        self
    }

    /// Set the CA certificate path for client verification.
    pub fn with_ca_cert(mut self, path: impl Into<String>) -> Self {
        self.ca_cert_path = Some(path.into());
        self
    }

    /// Require client authentication.
    pub fn require_client_auth(mut self, require: bool) -> Self {
        self.require_client_auth = require;
        self
    }

    /// Set minimum TLS version.
    pub fn with_min_tls_version(mut self, version: TlsVersion) -> Self {
        self.min_tls_version = version;
        self
    }

    /// Build server TLS configuration.
    pub fn build_server_config(self) -> CryptoResult<Arc<ServerConfig>> {
        let cert_path = self
            .cert_path
            .ok_or_else(|| CryptoError::InvalidConfiguration("Certificate path required".to_string()))?;

        let key_path = self
            .key_path
            .ok_or_else(|| CryptoError::InvalidConfiguration("Private key path required".to_string()))?;

        // Load certificates
        let cert_file = File::open(&cert_path)
            .map_err(|e| CryptoError::CertificateError(format!("Failed to open cert file: {}", e)))?;
        let mut cert_reader = BufReader::new(cert_file);
        let cert_chain: Vec<_> = certs(&mut cert_reader)
            .filter_map(|cert| cert.ok())
            .collect();

        if cert_chain.is_empty() {
            return Err(CryptoError::CertificateError(
                "No valid certificates found".to_string(),
            ));
        }

        // Load private key
        let key_file = File::open(&key_path)
            .map_err(|e| CryptoError::CertificateError(format!("Failed to open key file: {}", e)))?;
        let mut key_reader = BufReader::new(key_file);
        let mut keys = pkcs8_private_keys(&mut key_reader)
            .filter_map(|key| key.ok())
            .collect::<Vec<_>>();

        if keys.is_empty() {
            return Err(CryptoError::CertificateError(
                "No valid private keys found".to_string(),
            ));
        }

        let private_key = keys.remove(0);

        // Build server config
        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert_chain, private_key.into())
            .map_err(|e| CryptoError::TransportError(format!("Failed to build server config: {}", e)))?;

        Ok(Arc::new(config))
    }

    /// Build client TLS configuration.
    pub fn build_client_config(self) -> CryptoResult<Arc<ClientConfig>> {
        // Build client config with system root certificates
        let mut root_store = rustls::RootCertStore::empty();

        // Add webpki roots
        root_store.extend(
            webpki_roots::TLS_SERVER_ROOTS
                .iter()
                .cloned()
        );

        // If custom CA cert is provided, add it
        if let Some(ca_path) = &self.ca_cert_path {
            let ca_file = File::open(ca_path)
                .map_err(|e| CryptoError::CertificateError(format!("Failed to open CA cert: {}", e)))?;
            let mut ca_reader = BufReader::new(ca_file);
            let ca_certs: Vec<_> = certs(&mut ca_reader)
                .filter_map(|cert| cert.ok())
                .collect();

            for cert in ca_certs {
                root_store.add(cert).map_err(|e| {
                    CryptoError::CertificateError(format!("Failed to add CA cert: {}", e))
                })?;
            }
        }

        let config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        Ok(Arc::new(config))
    }
}

impl Default for TlsConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// TLS session manager.
pub struct TlsSessionManager {
    server_config: Option<Arc<ServerConfig>>,
    client_config: Option<Arc<ClientConfig>>,
}

impl TlsSessionManager {
    /// Create a new TLS session manager.
    pub fn new() -> Self {
        Self {
            server_config: None,
            client_config: None,
        }
    }

    /// Set server configuration.
    pub fn with_server_config(mut self, config: Arc<ServerConfig>) -> Self {
        self.server_config = Some(config);
        self
    }

    /// Set client configuration.
    pub fn with_client_config(mut self, config: Arc<ClientConfig>) -> Self {
        self.client_config = Some(config);
        self
    }

    /// Get server configuration.
    pub fn server_config(&self) -> Option<&Arc<ServerConfig>> {
        self.server_config.as_ref()
    }

    /// Get client configuration.
    pub fn client_config(&self) -> Option<&Arc<ClientConfig>> {
        self.client_config.as_ref()
    }

    /// Validate TLS configuration.
    pub fn validate(&self) -> CryptoResult<()> {
        if self.server_config.is_none() && self.client_config.is_none() {
            return Err(CryptoError::InvalidConfiguration(
                "No TLS configuration set".to_string(),
            ));
        }
        Ok(())
    }
}

impl Default for TlsSessionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a self-signed certificate for testing.
pub fn generate_self_signed_cert(
    subject_alt_names: Vec<String>,
) -> CryptoResult<(Vec<u8>, Vec<u8>)> {
    use rcgen::{Certificate, CertificateParams, DistinguishedName};

    let mut params = CertificateParams::new(subject_alt_names);

    let mut dn = DistinguishedName::new();
    dn.push(rcgen::DnType::CommonName, "Meridian GIS");
    dn.push(rcgen::DnType::OrganizationName, "Meridian");
    params.distinguished_name = dn;

    let cert = Certificate::from_params(params)
        .map_err(|e| CryptoError::CertificateError(format!("Failed to generate certificate: {}", e)))?;

    let cert_pem = cert.serialize_pem()
        .map_err(|e| CryptoError::CertificateError(format!("Failed to serialize cert: {}", e)))?;

    let key_pem = cert.serialize_private_key_pem();

    Ok((cert_pem.into_bytes(), key_pem.into_bytes()))
}

/// Write certificate and key to files.
pub fn write_cert_and_key(
    cert_path: &Path,
    key_path: &Path,
    cert_pem: &[u8],
    key_pem: &[u8],
) -> CryptoResult<()> {
    std::fs::write(cert_path, cert_pem)
        .map_err(|e| CryptoError::IoError(e))?;

    std::fs::write(key_path, key_pem)
        .map_err(|e| CryptoError::IoError(e))?;

    // Set restrictive permissions on private key
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(key_path, perms)
            .map_err(|e| CryptoError::IoError(e))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_self_signed_cert_generation() {
        let subject_alt_names = vec!["localhost".to_string(), "127.0.0.1".to_string()];
        let result = generate_self_signed_cert(subject_alt_names);
        assert!(result.is_ok());

        let (cert_pem, key_pem) = result.unwrap();
        assert!(!cert_pem.is_empty());
        assert!(!key_pem.is_empty());
    }

    #[test]
    fn test_cert_and_key_writing() {
        let temp_dir = TempDir::new().unwrap();
        let cert_path = temp_dir.path().join("cert.pem");
        let key_path = temp_dir.path().join("key.pem");

        let (cert_pem, key_pem) = generate_self_signed_cert(vec!["localhost".to_string()]).unwrap();

        let result = write_cert_and_key(&cert_path, &key_path, &cert_pem, &key_pem);
        assert!(result.is_ok());

        assert!(cert_path.exists());
        assert!(key_path.exists());
    }

    #[test]
    fn test_client_config_builder() {
        let builder = TlsConfigBuilder::new();
        let result = builder.build_client_config();
        assert!(result.is_ok());
    }
}
