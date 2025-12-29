//! Business glossary for terms, definitions, and data dictionaries

use crate::error::{GovernanceError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Business glossary manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessGlossary {
    /// Terms by identifier
    terms: HashMap<String, BusinessTerm>,
    /// Categories
    categories: HashMap<String, Category>,
    /// Term relationships
    relationships: Vec<TermRelationship>,
}

/// Business term definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessTerm {
    /// Term identifier
    pub id: String,
    /// Term name
    pub name: String,
    /// Display name
    pub display_name: String,
    /// Definition
    pub definition: String,
    /// Category
    pub category: String,
    /// Owner/steward
    pub owner: String,
    /// Status
    pub status: TermStatus,
    /// Synonyms
    pub synonyms: Vec<String>,
    /// Acronyms
    pub acronyms: Vec<String>,
    /// Examples
    pub examples: Vec<String>,
    /// Usage notes
    pub usage_notes: Option<String>,
    /// Related data assets (fields, tables, etc.)
    pub related_assets: Vec<String>,
    /// Tags
    pub tags: HashSet<String>,
    /// Custom attributes
    pub attributes: HashMap<String, String>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Last reviewed timestamp
    pub last_reviewed: Option<DateTime<Utc>>,
    /// Approved by
    pub approved_by: Option<String>,
}

/// Term status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TermStatus {
    Draft,
    UnderReview,
    Approved,
    Deprecated,
    Retired,
}

/// Business term category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    /// Category identifier
    pub id: String,
    /// Category name
    pub name: String,
    /// Category description
    pub description: String,
    /// Parent category (for hierarchies)
    pub parent: Option<String>,
    /// Category owner
    pub owner: String,
}

/// Relationship between terms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermRelationship {
    /// Relationship identifier
    pub id: String,
    /// Source term
    pub source_term: String,
    /// Target term
    pub target_term: String,
    /// Relationship type
    pub relationship_type: RelationshipType,
    /// Relationship description
    pub description: Option<String>,
}

/// Relationship type between terms
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RelationshipType {
    /// Term is a synonym of another
    SynonymOf,
    /// Term is related to another
    RelatedTo,
    /// Term is a parent/child of another
    IsPartOf,
    /// Term replaces another (deprecation)
    Replaces,
    /// Term is replaced by another
    ReplacedBy,
    /// Custom relationship
    Custom(String),
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Matching terms
    pub terms: Vec<BusinessTerm>,
    /// Total matches
    pub total: usize,
    /// Search query
    pub query: String,
}

impl BusinessGlossary {
    /// Create a new business glossary
    pub fn new() -> Self {
        Self {
            terms: HashMap::new(),
            categories: HashMap::new(),
            relationships: Vec::new(),
        }
    }

    /// Add a business term
    pub fn add_term(&mut self, term: BusinessTerm) -> Result<()> {
        if self.terms.contains_key(&term.id) {
            return Err(GovernanceError::Glossary(format!(
                "Term already exists: {}",
                term.id
            )));
        }

        // Verify category exists
        if !self.categories.contains_key(&term.category) {
            return Err(GovernanceError::Glossary(format!(
                "Category not found: {}",
                term.category
            )));
        }

        self.terms.insert(term.id.clone(), term);
        Ok(())
    }

    /// Get a business term
    pub fn get_term(&self, term_id: &str) -> Result<&BusinessTerm> {
        self.terms
            .get(term_id)
            .ok_or_else(|| GovernanceError::TermNotFound(term_id.to_string()))
    }

    /// Update a business term
    pub fn update_term(&mut self, term: BusinessTerm) -> Result<()> {
        if !self.terms.contains_key(&term.id) {
            return Err(GovernanceError::TermNotFound(term.id.clone()));
        }

        self.terms.insert(term.id.clone(), term);
        Ok(())
    }

    /// Delete a business term
    pub fn delete_term(&mut self, term_id: &str) -> Result<BusinessTerm> {
        // Remove relationships involving this term
        self.relationships
            .retain(|r| r.source_term != term_id && r.target_term != term_id);

        self.terms
            .remove(term_id)
            .ok_or_else(|| GovernanceError::TermNotFound(term_id.to_string()))
    }

    /// Add a category
    pub fn add_category(&mut self, category: Category) -> Result<()> {
        if self.categories.contains_key(&category.id) {
            return Err(GovernanceError::Glossary(format!(
                "Category already exists: {}",
                category.id
            )));
        }

        // Verify parent exists if specified
        if let Some(ref parent) = category.parent {
            if !self.categories.contains_key(parent) {
                return Err(GovernanceError::Glossary(format!(
                    "Parent category not found: {}",
                    parent
                )));
            }
        }

        self.categories.insert(category.id.clone(), category);
        Ok(())
    }

    /// Get a category
    pub fn get_category(&self, category_id: &str) -> Result<&Category> {
        self.categories.get(category_id).ok_or_else(|| {
            GovernanceError::Glossary(format!("Category not found: {}", category_id))
        })
    }

    /// Add a relationship between terms
    pub fn add_relationship(&mut self, relationship: TermRelationship) -> Result<()> {
        // Verify both terms exist
        if !self.terms.contains_key(&relationship.source_term) {
            return Err(GovernanceError::TermNotFound(
                relationship.source_term.clone(),
            ));
        }

        if !self.terms.contains_key(&relationship.target_term) {
            return Err(GovernanceError::TermNotFound(
                relationship.target_term.clone(),
            ));
        }

        self.relationships.push(relationship);
        Ok(())
    }

    /// Get relationships for a term
    pub fn get_term_relationships(&self, term_id: &str) -> Vec<&TermRelationship> {
        self.relationships
            .iter()
            .filter(|r| r.source_term == term_id || r.target_term == term_id)
            .collect()
    }

    /// Search terms by name or definition
    pub fn search(&self, query: &str) -> SearchResult {
        let query_lower = query.to_lowercase();

        let matching_terms: Vec<BusinessTerm> = self
            .terms
            .values()
            .filter(|term| {
                term.name.to_lowercase().contains(&query_lower)
                    || term.display_name.to_lowercase().contains(&query_lower)
                    || term.definition.to_lowercase().contains(&query_lower)
                    || term.synonyms.iter().any(|s| s.to_lowercase().contains(&query_lower))
                    || term.acronyms.iter().any(|a| a.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect();

        let total = matching_terms.len();

        SearchResult {
            terms: matching_terms,
            total,
            query: query.to_string(),
        }
    }

    /// Find term by exact name
    pub fn find_by_name(&self, name: &str) -> Option<&BusinessTerm> {
        self.terms
            .values()
            .find(|term| term.name.eq_ignore_ascii_case(name))
    }

    /// Find terms by category
    pub fn find_by_category(&self, category_id: &str) -> Vec<&BusinessTerm> {
        self.terms
            .values()
            .filter(|term| term.category == category_id)
            .collect()
    }

    /// Find terms by status
    pub fn find_by_status(&self, status: &TermStatus) -> Vec<&BusinessTerm> {
        self.terms
            .values()
            .filter(|term| &term.status == status)
            .collect()
    }

    /// Find terms by owner
    pub fn find_by_owner(&self, owner: &str) -> Vec<&BusinessTerm> {
        self.terms
            .values()
            .filter(|term| term.owner == owner)
            .collect()
    }

    /// Find terms linked to a data asset
    pub fn find_by_asset(&self, asset_id: &str) -> Vec<&BusinessTerm> {
        self.terms
            .values()
            .filter(|term| term.related_assets.contains(&asset_id.to_string()))
            .collect()
    }

    /// Get synonyms for a term
    pub fn get_synonyms(&self, term_id: &str) -> Result<Vec<&BusinessTerm>> {
        let relationships = self.get_term_relationships(term_id);

        let synonym_ids: Vec<&str> = relationships
            .iter()
            .filter(|r| r.relationship_type == RelationshipType::SynonymOf)
            .filter_map(|r| {
                if r.source_term == term_id {
                    Some(r.target_term.as_str())
                } else {
                    Some(r.source_term.as_str())
                }
            })
            .collect();

        let mut synonyms = Vec::new();
        for id in synonym_ids {
            if let Ok(term) = self.get_term(id) {
                synonyms.push(term);
            }
        }

        Ok(synonyms)
    }

    /// Get related terms
    pub fn get_related_terms(&self, term_id: &str) -> Result<Vec<&BusinessTerm>> {
        let relationships = self.get_term_relationships(term_id);

        let related_ids: Vec<&str> = relationships
            .iter()
            .filter(|r| r.relationship_type == RelationshipType::RelatedTo)
            .filter_map(|r| {
                if r.source_term == term_id {
                    Some(r.target_term.as_str())
                } else {
                    Some(r.source_term.as_str())
                }
            })
            .collect();

        let mut related = Vec::new();
        for id in related_ids {
            if let Ok(term) = self.get_term(id) {
                related.push(term);
            }
        }

        Ok(related)
    }

    /// Approve a term
    pub fn approve_term(&mut self, term_id: &str, approved_by: String) -> Result<()> {
        let term = self.terms.get_mut(term_id).ok_or_else(|| {
            GovernanceError::TermNotFound(term_id.to_string())
        })?;

        term.status = TermStatus::Approved;
        term.approved_by = Some(approved_by);
        term.updated_at = Utc::now();

        Ok(())
    }

    /// Deprecate a term
    pub fn deprecate_term(&mut self, term_id: &str, replaced_by: Option<String>) -> Result<()> {
        let term = self.terms.get_mut(term_id).ok_or_else(|| {
            GovernanceError::TermNotFound(term_id.to_string())
        })?;

        term.status = TermStatus::Deprecated;
        term.updated_at = Utc::now();

        // Add replacement relationship if specified
        if let Some(replacement_id) = replaced_by {
            let relationship = TermRelationship {
                id: uuid::Uuid::new_v4().to_string(),
                source_term: term_id.to_string(),
                target_term: replacement_id,
                relationship_type: RelationshipType::ReplacedBy,
                description: Some("Term deprecated and replaced".to_string()),
            };

            self.add_relationship(relationship)?;
        }

        Ok(())
    }

    /// List all terms
    pub fn list_terms(&self) -> Vec<&BusinessTerm> {
        self.terms.values().collect()
    }

    /// List all categories
    pub fn list_categories(&self) -> Vec<&Category> {
        self.categories.values().collect()
    }

    /// Get glossary statistics
    pub fn get_statistics(&self) -> GlossaryStatistics {
        let mut terms_by_status = HashMap::new();
        let mut terms_by_category = HashMap::new();

        for term in self.terms.values() {
            *terms_by_status
                .entry(format!("{:?}", term.status))
                .or_insert(0) += 1;

            *terms_by_category
                .entry(term.category.clone())
                .or_insert(0) += 1;
        }

        GlossaryStatistics {
            total_terms: self.terms.len(),
            total_categories: self.categories.len(),
            total_relationships: self.relationships.len(),
            terms_by_status,
            terms_by_category,
        }
    }
}

/// Glossary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryStatistics {
    pub total_terms: usize,
    pub total_categories: usize,
    pub total_relationships: usize,
    pub terms_by_status: HashMap<String, usize>,
    pub terms_by_category: HashMap<String, usize>,
}

impl Default for BusinessGlossary {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glossary_creation() {
        let glossary = BusinessGlossary::new();
        assert_eq!(glossary.list_terms().len(), 0);
    }

    #[test]
    fn test_add_category_and_term() {
        let mut glossary = BusinessGlossary::new();

        let category = Category {
            id: "customer".to_string(),
            name: "Customer".to_string(),
            description: "Customer-related terms".to_string(),
            parent: None,
            owner: "business_team".to_string(),
        };

        glossary.add_category(category).unwrap();

        let term = BusinessTerm {
            id: "customer_id".to_string(),
            name: "Customer ID".to_string(),
            display_name: "Customer Identifier".to_string(),
            definition: "Unique identifier for a customer".to_string(),
            category: "customer".to_string(),
            owner: "business_team".to_string(),
            status: TermStatus::Draft,
            synonyms: vec!["CID".to_string()],
            acronyms: vec![],
            examples: vec!["CUST-12345".to_string()],
            usage_notes: None,
            related_assets: vec![],
            tags: HashSet::new(),
            attributes: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_reviewed: None,
            approved_by: None,
        };

        glossary.add_term(term).unwrap();
        assert_eq!(glossary.list_terms().len(), 1);
    }
}
