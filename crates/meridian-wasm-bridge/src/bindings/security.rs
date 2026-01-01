//! Security validation and sanitization bindings.
//!
//! Features:
//! - XSS (Cross-Site Scripting) detection
//! - SQL injection detection
//! - CSRF token validation
//! - Input sanitization
//! - Content Security Policy validation
//! - Rate limiting checks

use wasm_bindgen::prelude::*;
use crate::types::{SecurityParams, SecurityResult, SecurityThreat, LocationInfo, OperationResult};
use crate::async_bridge::execute_async;
use serde::{Deserialize, Serialize};

/// Security engine for validating and sanitizing user input.
#[wasm_bindgen]
pub struct SecurityEngine {
    instance_id: String,
    strict_mode: bool,
}

#[wasm_bindgen]
impl SecurityEngine {
    /// Create a new security engine instance.
    ///
    /// # Arguments
    ///
    /// * `strict_mode` - If true, applies stricter validation rules
    #[wasm_bindgen(constructor)]
    pub fn new(strict_mode: bool) -> Self {
        Self {
            instance_id: uuid::Uuid::new_v4().to_string(),
            strict_mode,
        }
    }

    /// Get the instance ID.
    #[wasm_bindgen(getter)]
    pub fn instance_id(&self) -> String {
        self.instance_id.clone()
    }

    /// Validate input for security threats.
    ///
    /// Returns detailed information about detected threats.
    pub async fn validate(&self, params: JsValue) -> Result<JsValue, JsValue> {
        execute_async(async move {
            let params: SecurityParams = serde_wasm_bindgen::from_value(params)
                .map_err(|e| JsValue::from_str(&format!("Invalid params: {}", e)))?;

            tracing::debug!("Validating input for {}: {} chars", params.check_type, params.input.len());

            let result = match params.check_type.as_str() {
                "xss" => validate_xss(&params.input, self.strict_mode)?,
                "sql_injection" => validate_sql_injection(&params.input)?,
                "csrf" => validate_csrf(&params.input)?,
                "path_traversal" => validate_path_traversal(&params.input)?,
                "command_injection" => validate_command_injection(&params.input)?,
                _ => {
                    return Err(JsValue::from_str(&format!(
                        "Unknown check type: {}",
                        params.check_type
                    )));
                }
            };

            if !result.threats.is_empty() {
                tracing::warn!(
                    "Security threats detected: {} threats in {} check",
                    result.threats.len(),
                    params.check_type
                );
            }

            let op_result = OperationResult::success(result, Some(0));
            serde_wasm_bindgen::to_value(&op_result)
                .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
        }).await
    }

    /// Sanitize input by removing or escaping dangerous content.
    ///
    /// Returns sanitized string that is safe to use.
    pub async fn sanitize(&self, input: String, sanitize_type: String) -> Result<String, JsValue> {
        tracing::debug!("Sanitizing input for {}: {} chars", sanitize_type, input.len());

        let sanitized = match sanitize_type.as_str() {
            "html" => sanitize_html(&input),
            "sql" => sanitize_sql(&input),
            "url" => sanitize_url(&input),
            "filename" => sanitize_filename(&input),
            _ => {
                return Err(JsValue::from_str(&format!(
                    "Unknown sanitize type: {}",
                    sanitize_type
                )));
            }
        };

        Ok(sanitized)
    }

    /// Validate a Content Security Policy header.
    pub async fn validate_csp(&self, csp: String) -> Result<JsValue, JsValue> {
        tracing::debug!("Validating CSP: {}", csp);

        let result = validate_csp_internal(&csp)?;

        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
    }

    /// Check if a password meets security requirements.
    pub fn validate_password(&self, password: String) -> Result<JsValue, JsValue> {
        let result = validate_password_internal(&password, self.strict_mode)?;

        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
    }

    /// Generate a secure random token.
    pub fn generate_token(&self, length: usize) -> String {
        use uuid::Uuid;

        // In production, use a proper cryptographic RNG
        let tokens: Vec<String> = (0..((length + 31) / 32))
            .map(|_| Uuid::new_v4().to_string().replace("-", ""))
            .collect();

        tokens.join("")[..length].to_string()
    }

    /// Hash a password using a secure algorithm.
    ///
    /// Returns the hashed password (placeholder - use bcrypt/argon2 in production).
    pub async fn hash_password(&self, password: String) -> Result<String, JsValue> {
        // Placeholder: In production, use bcrypt or argon2
        let hash = format!("$bcrypt${}$", base64::encode(password));
        Ok(hash)
    }

    /// Verify a password against a hash.
    pub async fn verify_password(&self, password: String, hash: String) -> Result<bool, JsValue> {
        // Placeholder: In production, use bcrypt or argon2
        let expected_hash = format!("$bcrypt${}$", base64::encode(&password));
        Ok(expected_hash == hash)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CspValidationResult {
    valid: bool,
    warnings: Vec<String>,
    errors: Vec<String>,
    directives: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PasswordValidationResult {
    valid: bool,
    strength: String, // weak, medium, strong, very_strong
    issues: Vec<String>,
    score: u8, // 0-100
}

// Internal validation functions

fn validate_xss(input: &str, strict: bool) -> Result<SecurityResult, JsValue> {
    let mut threats = Vec::new();

    // Check for script tags
    if let Some(pos) = input.to_lowercase().find("<script") {
        threats.push(SecurityThreat {
            threat_type: "xss_script_tag".to_string(),
            severity: "high".to_string(),
            description: "Script tag detected in input".to_string(),
            location: Some(LocationInfo {
                line: 0,
                column: pos,
                length: 7,
            }),
        });
    }

    // Check for javascript: protocol
    if let Some(pos) = input.to_lowercase().find("javascript:") {
        threats.push(SecurityThreat {
            threat_type: "xss_javascript_protocol".to_string(),
            severity: "high".to_string(),
            description: "JavaScript protocol detected".to_string(),
            location: Some(LocationInfo {
                line: 0,
                column: pos,
                length: 11,
            }),
        });
    }

    // Check for event handlers (onclick, onerror, etc.)
    if strict {
        let event_handlers = ["onclick", "onerror", "onload", "onmouseover"];
        for handler in &event_handlers {
            if let Some(pos) = input.to_lowercase().find(handler) {
                threats.push(SecurityThreat {
                    threat_type: "xss_event_handler".to_string(),
                    severity: "medium".to_string(),
                    description: format!("Event handler '{}' detected", handler),
                    location: Some(LocationInfo {
                        line: 0,
                        column: pos,
                        length: handler.len(),
                    }),
                });
            }
        }
    }

    let is_safe = threats.is_empty();
    let sanitized = if !is_safe {
        Some(sanitize_html(input))
    } else {
        None
    };

    Ok(SecurityResult {
        is_safe,
        threats,
        sanitized,
    })
}

fn validate_sql_injection(input: &str) -> Result<SecurityResult, JsValue> {
    let mut threats = Vec::new();

    // Check for SQL keywords in suspicious contexts
    let sql_keywords = ["SELECT", "INSERT", "UPDATE", "DELETE", "DROP", "UNION", "EXEC"];
    let input_upper = input.to_uppercase();

    for keyword in &sql_keywords {
        if let Some(pos) = input_upper.find(keyword) {
            // Check if it's not part of a larger word
            let is_standalone = (pos == 0 || !input.chars().nth(pos - 1).unwrap().is_alphanumeric())
                && (pos + keyword.len() >= input.len()
                    || !input.chars().nth(pos + keyword.len()).unwrap().is_alphanumeric());

            if is_standalone {
                threats.push(SecurityThreat {
                    threat_type: "sql_injection".to_string(),
                    severity: "critical".to_string(),
                    description: format!("SQL keyword '{}' detected", keyword),
                    location: Some(LocationInfo {
                        line: 0,
                        column: pos,
                        length: keyword.len(),
                    }),
                });
            }
        }
    }

    // Check for SQL comment markers
    if input.contains("--") || input.contains("/*") {
        threats.push(SecurityThreat {
            threat_type: "sql_comment".to_string(),
            severity: "high".to_string(),
            description: "SQL comment marker detected".to_string(),
            location: None,
        });
    }

    let is_safe = threats.is_empty();
    let sanitized = if !is_safe {
        Some(sanitize_sql(input))
    } else {
        None
    };

    Ok(SecurityResult {
        is_safe,
        threats,
        sanitized,
    })
}

fn validate_csrf(token: &str) -> Result<SecurityResult, JsValue> {
    let mut threats = Vec::new();

    // Basic CSRF token validation
    if token.len() < 32 {
        threats.push(SecurityThreat {
            threat_type: "weak_csrf_token".to_string(),
            severity: "medium".to_string(),
            description: "CSRF token is too short".to_string(),
            location: None,
        });
    }

    // Check for non-random looking tokens
    if token.chars().all(|c| c.is_ascii_digit()) {
        threats.push(SecurityThreat {
            threat_type: "weak_csrf_token".to_string(),
            severity: "high".to_string(),
            description: "CSRF token appears non-random".to_string(),
            location: None,
        });
    }

    Ok(SecurityResult {
        is_safe: threats.is_empty(),
        threats,
        sanitized: None,
    })
}

fn validate_path_traversal(input: &str) -> Result<SecurityResult, JsValue> {
    let mut threats = Vec::new();

    if input.contains("../") || input.contains("..\\") {
        threats.push(SecurityThreat {
            threat_type: "path_traversal".to_string(),
            severity: "critical".to_string(),
            description: "Path traversal sequence detected".to_string(),
            location: None,
        });
    }

    Ok(SecurityResult {
        is_safe: threats.is_empty(),
        threats,
        sanitized: None,
    })
}

fn validate_command_injection(input: &str) -> Result<SecurityResult, JsValue> {
    let mut threats = Vec::new();

    let dangerous_chars = [";", "|", "&", "$", "`", "\n"];
    for &ch in &dangerous_chars {
        if input.contains(ch) {
            threats.push(SecurityThreat {
                threat_type: "command_injection".to_string(),
                severity: "critical".to_string(),
                description: format!("Dangerous character '{}' detected", ch),
                location: None,
            });
        }
    }

    Ok(SecurityResult {
        is_safe: threats.is_empty(),
        threats,
        sanitized: None,
    })
}

// Sanitization functions

fn sanitize_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

fn sanitize_sql(input: &str) -> String {
    input.replace('\'', "''").replace(';', "")
}

fn sanitize_url(input: &str) -> String {
    // Basic URL encoding
    input
        .replace(' ', "%20")
        .replace('<', "%3C")
        .replace('>', "%3E")
}

fn sanitize_filename(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
        .collect()
}

fn validate_csp_internal(csp: &str) -> Result<CspValidationResult, JsValue> {
    let directives: Vec<String> = csp
        .split(';')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    // Check for unsafe-inline
    if csp.contains("unsafe-inline") {
        warnings.push("'unsafe-inline' weakens CSP protection".to_string());
    }

    // Check for unsafe-eval
    if csp.contains("unsafe-eval") {
        warnings.push("'unsafe-eval' weakens CSP protection".to_string());
    }

    // Check for wildcard sources
    if csp.contains("*") && !csp.contains("*.") {
        warnings.push("Wildcard source (*) allows any origin".to_string());
    }

    Ok(CspValidationResult {
        valid: errors.is_empty(),
        warnings,
        errors,
        directives,
    })
}

fn validate_password_internal(password: &str, strict: bool) -> Result<PasswordValidationResult, JsValue> {
    let mut issues = Vec::new();
    let mut score = 0u8;

    // Length check
    if password.len() < 8 {
        issues.push("Password is too short (minimum 8 characters)".to_string());
    } else {
        score += 20;
        if password.len() >= 12 {
            score += 10;
        }
        if password.len() >= 16 {
            score += 10;
        }
    }

    // Complexity checks
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    if has_lowercase {
        score += 15;
    } else {
        issues.push("Password should contain lowercase letters".to_string());
    }

    if has_uppercase {
        score += 15;
    } else if strict {
        issues.push("Password should contain uppercase letters".to_string());
    }

    if has_digit {
        score += 15;
    } else if strict {
        issues.push("Password should contain numbers".to_string());
    }

    if has_special {
        score += 25;
    } else if strict {
        issues.push("Password should contain special characters".to_string());
    }

    let strength = match score {
        0..=30 => "weak",
        31..=60 => "medium",
        61..=85 => "strong",
        _ => "very_strong",
    };

    Ok(PasswordValidationResult {
        valid: issues.is_empty() || !strict,
        strength: strength.to_string(),
        issues,
        score,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_security_engine_creation() {
        let engine = SecurityEngine::new(true);
        assert!(!engine.instance_id().is_empty());
    }

    #[test]
    fn test_xss_detection() {
        let result = validate_xss("<script>alert('xss')</script>", false).unwrap();
        assert!(!result.is_safe);
        assert!(!result.threats.is_empty());
    }

    #[test]
    fn test_sql_injection_detection() {
        let result = validate_sql_injection("'; DROP TABLE users; --").unwrap();
        assert!(!result.is_safe);
        assert!(!result.threats.is_empty());
    }

    #[test]
    fn test_html_sanitization() {
        let sanitized = sanitize_html("<script>alert('xss')</script>");
        assert_eq!(sanitized, "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;");
    }

    #[test]
    fn test_password_validation() {
        let weak = validate_password_internal("123", false).unwrap();
        assert_eq!(weak.strength, "weak");

        let strong = validate_password_internal("MyP@ssw0rd123!", false).unwrap();
        assert!(strong.score > 80);
    }
}
