pub mod wcag_a;
pub mod wcag_aa;
pub mod wcag_aaa;

use crate::types::{Rule, WCAGLevel};
use std::collections::HashMap;
use lazy_static::lazy_static;

lazy_static! {
    static ref ALL_RULES: Vec<Rule> = {
        let mut rules = Vec::new();
        rules.extend(wcag_a::get_level_a_rules());
        rules.extend(wcag_aa::get_level_aa_rules());
        rules.extend(wcag_aaa::get_level_aaa_rules());
        rules
    };

    static ref RULES_BY_ID: HashMap<String, Rule> = {
        ALL_RULES.iter().map(|rule| (rule.id.clone(), rule.clone())).collect()
    };
}

/// Get all WCAG rules
pub fn get_all_rules() -> &'static Vec<Rule> {
    &ALL_RULES
}

/// Get rules filtered by WCAG levels
pub fn get_rules_by_levels(levels: &[WCAGLevel]) -> Vec<Rule> {
    ALL_RULES
        .iter()
        .filter(|rule| levels.contains(&rule.level))
        .cloned()
        .collect()
}

/// Get a rule by its ID
pub fn get_rule_by_id(id: &str) -> Option<&'static Rule> {
    RULES_BY_ID.get(id)
}

/// Get rules by principle
pub fn get_rules_by_principle(principle: crate::types::Principle) -> Vec<Rule> {
    ALL_RULES
        .iter()
        .filter(|rule| rule.principle == principle)
        .cloned()
        .collect()
}

/// Get rules by tag
pub fn get_rules_by_tag(tag: &str) -> Vec<Rule> {
    ALL_RULES
        .iter()
        .filter(|rule| rule.tags.contains(&tag.to_string()))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_rules_loaded() {
        let rules = get_all_rules();
        assert!(rules.len() >= 50, "Should have at least 50 WCAG rules");
    }

    #[test]
    fn test_get_rule_by_id() {
        let rule = get_rule_by_id("image-alt");
        assert!(rule.is_some());
        assert_eq!(rule.unwrap().id, "image-alt");
    }

    #[test]
    fn test_get_rules_by_levels() {
        let a_rules = get_rules_by_levels(&[WCAGLevel::A]);
        assert!(!a_rules.is_empty());

        let aa_rules = get_rules_by_levels(&[WCAGLevel::AA]);
        assert!(!aa_rules.is_empty());

        let aaa_rules = get_rules_by_levels(&[WCAGLevel::AAA]);
        assert!(!aaa_rules.is_empty());
    }

    #[test]
    fn test_unique_rule_ids() {
        let rules = get_all_rules();
        let mut ids = std::collections::HashSet::new();

        for rule in rules {
            assert!(ids.insert(&rule.id), "Duplicate rule ID: {}", rule.id);
        }
    }
}
