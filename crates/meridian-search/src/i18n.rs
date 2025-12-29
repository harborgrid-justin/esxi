//! Multi-language support for search.

use crate::client::SearchClient;
use crate::error::{SearchError, SearchResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};
use whatlang::{detect, Lang};

/// Language detector for automatic language detection.
pub struct LanguageDetector;

impl LanguageDetector {
    /// Detect the language of a text.
    pub fn detect(text: &str) -> SearchResult<Language> {
        if text.is_empty() {
            return Err(SearchError::LanguageError(
                "Cannot detect language of empty text".to_string(),
            ));
        }

        let info = detect(text).ok_or_else(|| {
            SearchError::LanguageError("Could not detect language".to_string())
        })?;

        let language = match info.lang() {
            Lang::Eng => Language::English,
            Lang::Spa => Language::Spanish,
            Lang::Fra => Language::French,
            Lang::Deu => Language::German,
            Lang::Ita => Language::Italian,
            Lang::Por => Language::Portuguese,
            Lang::Rus => Language::Russian,
            Lang::Jpn => Language::Japanese,
            Lang::Kor => Language::Korean,
            Lang::Cmn => Language::Chinese,
            Lang::Ara => Language::Arabic,
            Lang::Hin => Language::Hindi,
            Lang::Nld => Language::Dutch,
            Lang::Swe => Language::Swedish,
            Lang::Pol => Language::Polish,
            Lang::Tur => Language::Turkish,
            _ => Language::Unknown,
        };

        debug!(
            "Detected language: {:?} with confidence: {}",
            language,
            info.confidence()
        );

        Ok(language)
    }

    /// Detect language and return ISO code.
    pub fn detect_iso_code(text: &str) -> SearchResult<String> {
        let language = Self::detect(text)?;
        Ok(language.iso_code().to_string())
    }
}

/// Supported languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    English,
    Spanish,
    French,
    German,
    Italian,
    Portuguese,
    Russian,
    Japanese,
    Korean,
    Chinese,
    Arabic,
    Hindi,
    Dutch,
    Swedish,
    Polish,
    Turkish,
    Unknown,
}

impl Language {
    /// Get the ISO 639-1 language code.
    pub fn iso_code(&self) -> &str {
        match self {
            Self::English => "en",
            Self::Spanish => "es",
            Self::French => "fr",
            Self::German => "de",
            Self::Italian => "it",
            Self::Portuguese => "pt",
            Self::Russian => "ru",
            Self::Japanese => "ja",
            Self::Korean => "ko",
            Self::Chinese => "zh",
            Self::Arabic => "ar",
            Self::Hindi => "hi",
            Self::Dutch => "nl",
            Self::Swedish => "sv",
            Self::Polish => "pl",
            Self::Turkish => "tr",
            Self::Unknown => "unknown",
        }
    }

    /// Get the Elasticsearch analyzer name for this language.
    pub fn analyzer_name(&self) -> &str {
        match self {
            Self::English => "english",
            Self::Spanish => "spanish",
            Self::French => "french",
            Self::German => "german",
            Self::Italian => "italian",
            Self::Portuguese => "portuguese",
            Self::Russian => "russian",
            Self::Japanese => "cjk",
            Self::Korean => "cjk",
            Self::Chinese => "cjk",
            Self::Arabic => "arabic",
            Self::Hindi => "standard",
            Self::Dutch => "dutch",
            Self::Swedish => "swedish",
            Self::Polish => "standard",
            Self::Turkish => "turkish",
            Self::Unknown => "standard",
        }
    }

    /// Get the stemmer name for this language.
    pub fn stemmer(&self) -> Option<&str> {
        match self {
            Self::English => Some("english"),
            Self::Spanish => Some("spanish"),
            Self::French => Some("french"),
            Self::German => Some("german"),
            Self::Italian => Some("italian"),
            Self::Portuguese => Some("portuguese"),
            Self::Russian => Some("russian"),
            Self::Dutch => Some("dutch"),
            Self::Swedish => Some("swedish"),
            Self::Turkish => Some("turkish"),
            _ => None,
        }
    }

    /// Get stopwords for this language.
    pub fn stopwords(&self) -> Vec<&str> {
        match self {
            Self::English => vec![
                "a", "an", "and", "are", "as", "at", "be", "but", "by", "for", "if", "in",
                "into", "is", "it", "no", "not", "of", "on", "or", "such", "that", "the",
                "their", "then", "there", "these", "they", "this", "to", "was", "will", "with",
            ],
            Self::Spanish => vec![
                "el", "la", "de", "que", "y", "a", "en", "un", "ser", "se", "no", "haber",
                "por", "con", "su", "para", "como", "estar", "tener", "le", "lo", "todo",
            ],
            Self::French => vec![
                "le", "de", "un", "être", "et", "à", "il", "avoir", "ne", "je", "son",
                "que", "se", "qui", "ce", "dans", "en", "du", "elle", "au", "pour",
            ],
            Self::German => vec![
                "der", "die", "und", "in", "den", "von", "zu", "das", "mit", "sich", "des",
                "auf", "für", "ist", "im", "dem", "nicht", "ein", "eine", "als", "auch",
            ],
            _ => vec![],
        }
    }
}

impl std::str::FromStr for Language {
    type Err = SearchError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "en" | "english" => Ok(Self::English),
            "es" | "spanish" => Ok(Self::Spanish),
            "fr" | "french" => Ok(Self::French),
            "de" | "german" => Ok(Self::German),
            "it" | "italian" => Ok(Self::Italian),
            "pt" | "portuguese" => Ok(Self::Portuguese),
            "ru" | "russian" => Ok(Self::Russian),
            "ja" | "japanese" => Ok(Self::Japanese),
            "ko" | "korean" => Ok(Self::Korean),
            "zh" | "chinese" => Ok(Self::Chinese),
            "ar" | "arabic" => Ok(Self::Arabic),
            "hi" | "hindi" => Ok(Self::Hindi),
            "nl" | "dutch" => Ok(Self::Dutch),
            "sv" | "swedish" => Ok(Self::Swedish),
            "pl" | "polish" => Ok(Self::Polish),
            "tr" | "turkish" => Ok(Self::Turkish),
            _ => Ok(Self::Unknown),
        }
    }
}

/// Multi-language search builder.
pub struct MultiLanguageSearch {
    client: SearchClient,
    index: String,
    query: String,
    languages: Vec<Language>,
    boost_map: HashMap<Language, f64>,
    size: usize,
}

impl MultiLanguageSearch {
    /// Create a new multi-language search.
    pub fn new(client: SearchClient, index: impl Into<String>, query: impl Into<String>) -> Self {
        Self {
            client,
            index: index.into(),
            query: query.into(),
            languages: Vec::new(),
            boost_map: HashMap::new(),
            size: 10,
        }
    }

    /// Add a language to search.
    pub fn language(mut self, language: Language) -> Self {
        self.languages.push(language);
        self
    }

    /// Set boost for a specific language.
    pub fn boost_language(mut self, language: Language, boost: f64) -> Self {
        self.boost_map.insert(language, boost);
        self
    }

    /// Auto-detect and search in detected language.
    pub fn auto_detect(mut self) -> SearchResult<Self> {
        let language = LanguageDetector::detect(&self.query)?;
        info!("Auto-detected language: {:?}", language);
        self.languages.push(language);
        Ok(self)
    }

    /// Set the number of results.
    pub fn size(mut self, size: usize) -> Self {
        self.size = size;
        self
    }

    /// Execute the multi-language search.
    pub async fn execute<T>(&self) -> SearchResult<MultiLanguageResults<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        info!(
            "Executing multi-language search with {} languages",
            self.languages.len()
        );

        // Build multi-language query
        let query = self.build_query();

        let response = self
            .client
            .client()
            .search(elasticsearch::SearchParts::Index(&[&self.index]))
            .body(query)
            .send()
            .await?;

        if !response.status_code().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SearchError::ElasticsearchError(format!(
                "Multi-language search failed: {}",
                error_text
            )));
        }

        let body: serde_json::Value = response.json().await?;
        self.parse_results(body)
    }

    /// Build the multi-language query.
    fn build_query(&self) -> serde_json::Value {
        use serde_json::json;

        let mut should_clauses = Vec::new();

        for language in &self.languages {
            let field = format!("content.{}", language.iso_code());
            let boost = self.boost_map.get(language).copied().unwrap_or(1.0);

            should_clauses.push(json!({
                "match": {
                    field: {
                        "query": self.query,
                        "boost": boost
                    }
                }
            }));
        }

        // Add a fallback to standard field
        should_clauses.push(json!({
            "match": {
                "content": {
                    "query": self.query,
                    "boost": 0.5
                }
            }
        }));

        json!({
            "query": {
                "bool": {
                    "should": should_clauses
                }
            },
            "size": self.size
        })
    }

    /// Parse search results.
    fn parse_results<T>(&self, body: serde_json::Value) -> SearchResult<MultiLanguageResults<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let hits = body["hits"]["hits"]
            .as_array()
            .ok_or_else(|| SearchError::QueryParseError("Missing hits array".to_string()))?;

        let total = body["hits"]["total"]["value"]
            .as_u64()
            .unwrap_or(hits.len() as u64);

        let results: Vec<MultiLanguageHit<T>> = hits
            .iter()
            .map(|hit| {
                let source: T = serde_json::from_value(hit["_source"].clone())?;
                let score = hit["_score"].as_f64();

                Ok(MultiLanguageHit {
                    id: hit["_id"].as_str().unwrap_or("").to_string(),
                    source,
                    score,
                })
            })
            .collect::<Result<Vec<_>, serde_json::Error>>()?;

        Ok(MultiLanguageResults {
            total,
            hits: results,
        })
    }
}

/// Multi-language search results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiLanguageResults<T> {
    pub total: u64,
    pub hits: Vec<MultiLanguageHit<T>>,
}

/// Individual multi-language search hit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiLanguageHit<T> {
    pub id: String,
    pub source: T,
    pub score: Option<f64>,
}

/// Text analyzer helper for different languages.
pub struct TextAnalyzer;

impl TextAnalyzer {
    /// Normalize text for a specific language.
    pub fn normalize(text: &str, language: Language) -> String {
        use unicode_segmentation::UnicodeSegmentation;

        let text = text.to_lowercase();

        // Remove stopwords
        let stopwords = language.stopwords();
        let words: Vec<&str> = text
            .unicode_words()
            .filter(|word| !stopwords.contains(word))
            .collect();

        words.join(" ")
    }

    /// Tokenize text respecting language-specific rules.
    pub fn tokenize(text: &str, language: Language) -> Vec<String> {
        use unicode_segmentation::UnicodeSegmentation;

        match language {
            Language::Japanese | Language::Korean | Language::Chinese => {
                // For CJK languages, split by character
                text.graphemes(true).map(String::from).collect()
            }
            _ => {
                // For other languages, split by words
                text.unicode_words().map(String::from).collect()
            }
        }
    }
}

/// Translation helper (placeholder for integration with translation services).
pub struct TranslationHelper {
    supported_languages: Vec<Language>,
}

impl TranslationHelper {
    /// Create a new translation helper.
    pub fn new() -> Self {
        Self {
            supported_languages: vec![
                Language::English,
                Language::Spanish,
                Language::French,
                Language::German,
                Language::Italian,
                Language::Portuguese,
            ],
        }
    }

    /// Check if translation is supported.
    pub fn supports_language(&self, language: Language) -> bool {
        self.supported_languages.contains(&language)
    }

    /// Placeholder for query translation (would integrate with external service).
    pub async fn translate_query(
        &self,
        query: &str,
        from: Language,
        to: Language,
    ) -> SearchResult<String> {
        if !self.supports_language(from) || !self.supports_language(to) {
            return Err(SearchError::LanguageError(
                "Unsupported language for translation".to_string(),
            ));
        }

        // This would call an external translation API
        info!(
            "Translating query from {:?} to {:?}: {}",
            from, to, query
        );

        // For now, return the original query
        Ok(query.to_string())
    }
}

impl Default for TranslationHelper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_iso_code() {
        assert_eq!(Language::English.iso_code(), "en");
        assert_eq!(Language::Spanish.iso_code(), "es");
        assert_eq!(Language::Japanese.iso_code(), "ja");
    }

    #[test]
    fn test_language_analyzer() {
        assert_eq!(Language::English.analyzer_name(), "english");
        assert_eq!(Language::Japanese.analyzer_name(), "cjk");
        assert_eq!(Language::Unknown.analyzer_name(), "standard");
    }

    #[test]
    fn test_language_stemmer() {
        assert_eq!(Language::English.stemmer(), Some("english"));
        assert_eq!(Language::Spanish.stemmer(), Some("spanish"));
        assert_eq!(Language::Japanese.stemmer(), None);
    }

    #[test]
    fn test_language_stopwords() {
        let stopwords = Language::English.stopwords();
        assert!(stopwords.contains(&"the"));
        assert!(stopwords.contains(&"and"));
        assert!(!stopwords.is_empty());
    }

    #[test]
    fn test_language_from_str() {
        assert_eq!("en".parse::<Language>().unwrap(), Language::English);
        assert_eq!("es".parse::<Language>().unwrap(), Language::Spanish);
        assert_eq!("unknown".parse::<Language>().unwrap(), Language::Unknown);
    }

    #[test]
    fn test_text_analyzer_normalize() {
        let text = "The quick brown fox";
        let normalized = TextAnalyzer::normalize(text, Language::English);
        assert!(!normalized.contains("the")); // "the" is a stopword
        assert!(normalized.contains("quick"));
    }

    #[test]
    fn test_text_analyzer_tokenize() {
        let text = "Hello world";
        let tokens = TextAnalyzer::tokenize(text, Language::English);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], "Hello");
        assert_eq!(tokens[1], "world");
    }

    #[test]
    fn test_language_detector_empty_text() {
        let result = LanguageDetector::detect("");
        assert!(result.is_err());
    }

    #[test]
    fn test_translation_helper_supports() {
        let helper = TranslationHelper::new();
        assert!(helper.supports_language(Language::English));
        assert!(helper.supports_language(Language::Spanish));
        assert!(!helper.supports_language(Language::Japanese));
    }
}
