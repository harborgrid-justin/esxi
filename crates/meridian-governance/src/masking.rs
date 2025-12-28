//! Data masking and anonymization functions

use crate::error::{GovernanceError, Result};
use blake3;
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Data masking manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskingManager {
    /// Masking rules by field identifier
    rules: HashMap<String, MaskingRule>,
    /// Salt for hashing operations (in production, store securely)
    #[serde(skip)]
    salt: Vec<u8>,
}

/// Masking rule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskingRule {
    /// Rule identifier
    pub id: String,
    /// Target field identifier
    pub field_id: String,
    /// Masking strategy
    pub strategy: MaskingStrategy,
    /// Whether masking is reversible
    pub reversible: bool,
    /// Rule description
    pub description: Option<String>,
    /// Rule enabled
    pub enabled: bool,
}

/// Masking strategy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MaskingStrategy {
    /// Complete redaction (replace with placeholder)
    Redact {
        /// Placeholder value
        placeholder: String,
    },

    /// Partial masking (show only first/last N characters)
    Partial {
        /// Show first N characters
        show_first: usize,
        /// Show last N characters
        show_last: usize,
        /// Mask character
        mask_char: char,
    },

    /// One-way hashing (irreversible)
    Hash {
        /// Hash algorithm
        algorithm: HashAlgorithm,
    },

    /// Format-preserving encryption (reversible)
    FormatPreserving {
        /// Format to preserve (e.g., "email", "phone", "ssn")
        format_type: String,
    },

    /// Tokenization (reversible, requires token vault)
    Tokenization {
        /// Token format
        format: String,
    },

    /// Data substitution (replace with synthetic data)
    Substitution {
        /// Substitution type
        sub_type: SubstitutionType,
    },

    /// Generalization (reduce precision)
    Generalization {
        /// Generalization level
        level: GeneralizationLevel,
    },

    /// Shuffling (randomize within dataset)
    Shuffle,

    /// Nullification (set to null/empty)
    Nullify,

    /// Custom masking function
    Custom {
        /// Function name/identifier
        function: String,
        /// Configuration parameters
        config: HashMap<String, String>,
    },
}

/// Hash algorithm for masking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HashAlgorithm {
    Sha256,
    Sha512,
    Blake3,
}

/// Substitution type for synthetic data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SubstitutionType {
    /// Random value from predefined list
    RandomFromList { values: Vec<String> },
    /// Generated fake data
    Fake { data_type: FakeDataType },
    /// Constant value
    Constant { value: String },
}

/// Fake data type for substitution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FakeDataType {
    Name,
    Email,
    Phone,
    Address,
    Company,
    CreditCard,
    Ssn,
    Date,
    Number,
}

/// Generalization level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GeneralizationLevel {
    /// Age ranges (e.g., 25-30)
    AgeRange { bin_size: u8 },
    /// Date to year/month
    DateToYear,
    DateToMonth,
    /// Location to city/state/country
    LocationToCity,
    LocationToState,
    LocationToCountry,
    /// Numeric ranges
    NumericRange { bin_size: f64 },
}

/// Masking result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskingResult {
    /// Original value length/size
    pub original_size: usize,
    /// Masked value
    pub masked_value: String,
    /// Strategy used
    pub strategy: String,
    /// Whether reversible
    pub reversible: bool,
}

impl MaskingManager {
    /// Create a new masking manager
    pub fn new() -> Self {
        let mut salt = vec![0u8; 32];
        let rng = SystemRandom::new();
        rng.fill(&mut salt).expect("Failed to generate salt");

        Self {
            rules: HashMap::new(),
            salt,
        }
    }

    /// Add a masking rule
    pub fn add_rule(&mut self, rule: MaskingRule) -> Result<()> {
        if self.rules.contains_key(&rule.id) {
            return Err(GovernanceError::Masking(format!(
                "Masking rule already exists: {}",
                rule.id
            )));
        }

        self.rules.insert(rule.id.clone(), rule);
        Ok(())
    }

    /// Get a masking rule
    pub fn get_rule(&self, rule_id: &str) -> Result<&MaskingRule> {
        self.rules.get(rule_id).ok_or_else(|| {
            GovernanceError::Masking(format!("Masking rule not found: {}", rule_id))
        })
    }

    /// Remove a masking rule
    pub fn remove_rule(&mut self, rule_id: &str) -> Result<MaskingRule> {
        self.rules.remove(rule_id).ok_or_else(|| {
            GovernanceError::Masking(format!("Masking rule not found: {}", rule_id))
        })
    }

    /// Mask a value using a specific rule
    pub fn mask(&self, rule_id: &str, value: &str) -> Result<MaskingResult> {
        let rule = self.get_rule(rule_id)?;

        if !rule.enabled {
            return Ok(MaskingResult {
                original_size: value.len(),
                masked_value: value.to_string(),
                strategy: "None (disabled)".to_string(),
                reversible: false,
            });
        }

        let masked_value = self.apply_strategy(&rule.strategy, value)?;

        Ok(MaskingResult {
            original_size: value.len(),
            masked_value,
            strategy: format!("{:?}", rule.strategy),
            reversible: rule.reversible,
        })
    }

    /// Apply a masking strategy to a value
    fn apply_strategy(&self, strategy: &MaskingStrategy, value: &str) -> Result<String> {
        match strategy {
            MaskingStrategy::Redact { placeholder } => Ok(placeholder.clone()),

            MaskingStrategy::Partial {
                show_first,
                show_last,
                mask_char,
            } => Ok(Self::partial_mask(value, *show_first, *show_last, *mask_char)),

            MaskingStrategy::Hash { algorithm } => self.hash_value(value, algorithm),

            MaskingStrategy::FormatPreserving { format_type } => {
                Self::format_preserving_mask(value, format_type)
            }

            MaskingStrategy::Tokenization { format } => Self::tokenize(value, format),

            MaskingStrategy::Substitution { sub_type } => Self::substitute(value, sub_type),

            MaskingStrategy::Generalization { level } => Self::generalize(value, level),

            MaskingStrategy::Shuffle => {
                // Shuffling requires dataset context, return marker
                Ok("[SHUFFLED]".to_string())
            }

            MaskingStrategy::Nullify => Ok(String::new()),

            MaskingStrategy::Custom { function, config } => {
                Err(GovernanceError::Masking(format!(
                    "Custom masking function not implemented: {}",
                    function
                )))
            }
        }
    }

    /// Partial masking (show first and last N characters)
    fn partial_mask(value: &str, show_first: usize, show_last: usize, mask_char: char) -> String {
        let len = value.len();

        if len <= show_first + show_last {
            return value.to_string();
        }

        let first = &value[..show_first];
        let last = &value[len - show_last..];
        let masked_len = len - show_first - show_last;

        format!("{}{}{}", first, mask_char.to_string().repeat(masked_len), last)
    }

    /// Hash a value
    fn hash_value(&self, value: &str, algorithm: &HashAlgorithm) -> Result<String> {
        let salted = format!("{}{}", value, String::from_utf8_lossy(&self.salt));

        match algorithm {
            HashAlgorithm::Sha256 => {
                let mut hasher = Sha256::new();
                hasher.update(salted.as_bytes());
                Ok(format!("{:x}", hasher.finalize()))
            }
            HashAlgorithm::Sha512 => {
                let mut hasher = sha2::Sha512::new();
                hasher.update(salted.as_bytes());
                Ok(format!("{:x}", hasher.finalize()))
            }
            HashAlgorithm::Blake3 => {
                let hash = blake3::hash(salted.as_bytes());
                Ok(hash.to_hex().to_string())
            }
        }
    }

    /// Format-preserving masking
    fn format_preserving_mask(value: &str, format_type: &str) -> Result<String> {
        match format_type {
            "email" => {
                if let Some(at_pos) = value.find('@') {
                    let (local, domain) = value.split_at(at_pos);
                    let masked_local = if local.len() > 2 {
                        format!("{}***", &local[..2])
                    } else {
                        "***".to_string()
                    };
                    Ok(format!("{}{}", masked_local, domain))
                } else {
                    Ok("***@***.***".to_string())
                }
            }
            "phone" => {
                let digits: String = value.chars().filter(|c| c.is_numeric()).collect();
                if digits.len() >= 10 {
                    Ok(format!(
                        "(XXX) XXX-{}",
                        &digits[digits.len() - 4..]
                    ))
                } else {
                    Ok("XXX-XXX-XXXX".to_string())
                }
            }
            "ssn" => {
                let digits: String = value.chars().filter(|c| c.is_numeric()).collect();
                if digits.len() >= 9 {
                    Ok(format!(
                        "XXX-XX-{}",
                        &digits[digits.len() - 4..]
                    ))
                } else {
                    Ok("XXX-XX-XXXX".to_string())
                }
            }
            "credit_card" => {
                let digits: String = value.chars().filter(|c| c.is_numeric()).collect();
                if digits.len() >= 16 {
                    Ok(format!(
                        "XXXX XXXX XXXX {}",
                        &digits[digits.len() - 4..]
                    ))
                } else {
                    Ok("XXXX XXXX XXXX XXXX".to_string())
                }
            }
            _ => Err(GovernanceError::InvalidMaskingConfig(format!(
                "Unknown format type: {}",
                format_type
            ))),
        }
    }

    /// Tokenization (simplified, would use vault in production)
    fn tokenize(value: &str, format: &str) -> Result<String> {
        // Simplified tokenization - in production would use secure token vault
        let token = uuid::Uuid::new_v4().to_string();
        Ok(format!("TOKEN_{}", &token[..8]))
    }

    /// Data substitution
    fn substitute(value: &str, sub_type: &SubstitutionType) -> Result<String> {
        match sub_type {
            SubstitutionType::Constant { value } => Ok(value.clone()),

            SubstitutionType::RandomFromList { values } => {
                if values.is_empty() {
                    return Err(GovernanceError::InvalidMaskingConfig(
                        "Empty substitution list".to_string(),
                    ));
                }
                // Simplified: use hash to pick consistent value
                let hash = blake3::hash(value.as_bytes());
                let index = (hash.as_bytes()[0] as usize) % values.len();
                Ok(values[index].clone())
            }

            SubstitutionType::Fake { data_type } => {
                // Simplified fake data generation
                match data_type {
                    FakeDataType::Name => Ok("John Doe".to_string()),
                    FakeDataType::Email => Ok("example@example.com".to_string()),
                    FakeDataType::Phone => Ok("(555) 555-5555".to_string()),
                    FakeDataType::Address => Ok("123 Main St".to_string()),
                    FakeDataType::Company => Ok("Acme Corp".to_string()),
                    FakeDataType::CreditCard => Ok("4111 1111 1111 1111".to_string()),
                    FakeDataType::Ssn => Ok("123-45-6789".to_string()),
                    FakeDataType::Date => Ok("2024-01-01".to_string()),
                    FakeDataType::Number => Ok("12345".to_string()),
                }
            }
        }
    }

    /// Generalization
    fn generalize(value: &str, level: &GeneralizationLevel) -> Result<String> {
        match level {
            GeneralizationLevel::AgeRange { bin_size } => {
                if let Ok(age) = value.parse::<u8>() {
                    let lower = (age / bin_size) * bin_size;
                    let upper = lower + bin_size;
                    Ok(format!("{}-{}", lower, upper))
                } else {
                    Ok(value.to_string())
                }
            }

            GeneralizationLevel::DateToYear => {
                // Simplified: extract year
                if value.len() >= 4 {
                    Ok(value[..4].to_string())
                } else {
                    Ok(value.to_string())
                }
            }

            GeneralizationLevel::DateToMonth => {
                // Simplified: extract year-month
                if value.len() >= 7 {
                    Ok(value[..7].to_string())
                } else {
                    Ok(value.to_string())
                }
            }

            GeneralizationLevel::NumericRange { bin_size } => {
                if let Ok(num) = value.parse::<f64>() {
                    let lower = (num / bin_size).floor() * bin_size;
                    let upper = lower + bin_size;
                    Ok(format!("{:.2}-{:.2}", lower, upper))
                } else {
                    Ok(value.to_string())
                }
            }

            _ => Ok(value.to_string()),
        }
    }

    /// Mask multiple values using rules
    pub fn mask_batch(
        &self,
        data: HashMap<String, String>,
        field_rules: &HashMap<String, String>,
    ) -> Result<HashMap<String, MaskingResult>> {
        let mut results = HashMap::new();

        for (field, value) in data {
            if let Some(rule_id) = field_rules.get(&field) {
                let result = self.mask(rule_id, &value)?;
                results.insert(field, result);
            }
        }

        Ok(results)
    }

    /// List all masking rules
    pub fn list_rules(&self) -> Vec<&MaskingRule> {
        self.rules.values().collect()
    }

    /// Get masking statistics
    pub fn get_statistics(&self) -> MaskingStatistics {
        let mut strategies = HashMap::new();
        let mut reversible_count = 0;

        for rule in self.rules.values() {
            let strategy_name = format!("{:?}", rule.strategy).split('{').next().unwrap().to_string();
            *strategies.entry(strategy_name).or_insert(0) += 1;

            if rule.reversible {
                reversible_count += 1;
            }
        }

        MaskingStatistics {
            total_rules: self.rules.len(),
            enabled_rules: self.rules.values().filter(|r| r.enabled).count(),
            reversible_rules: reversible_count,
            strategies_used: strategies,
        }
    }
}

/// Masking statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskingStatistics {
    pub total_rules: usize,
    pub enabled_rules: usize,
    pub reversible_rules: usize,
    pub strategies_used: HashMap<String, usize>,
}

impl Default for MaskingManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_masking_manager_creation() {
        let manager = MaskingManager::new();
        assert_eq!(manager.list_rules().len(), 0);
    }

    #[test]
    fn test_partial_masking() {
        let result = MaskingManager::partial_mask("1234567890", 2, 2, '*');
        assert_eq!(result, "12******90");
    }

    #[test]
    fn test_email_masking() {
        let result = MaskingManager::format_preserving_mask("test@example.com", "email").unwrap();
        assert!(result.contains("@example.com"));
        assert!(result.starts_with("te***"));
    }

    #[test]
    fn test_redaction() {
        let mut manager = MaskingManager::new();
        let rule = MaskingRule {
            id: "redact_ssn".to_string(),
            field_id: "ssn".to_string(),
            strategy: MaskingStrategy::Redact {
                placeholder: "[REDACTED]".to_string(),
            },
            reversible: false,
            description: Some("Redact SSN".to_string()),
            enabled: true,
        };

        manager.add_rule(rule).unwrap();
        let result = manager.mask("redact_ssn", "123-45-6789").unwrap();
        assert_eq!(result.masked_value, "[REDACTED]");
    }
}
