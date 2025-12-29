//! WCAG 2.1 Success Criteria definitions

use serde::{Deserialize, Serialize};

/// WCAG conformance level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WCAGLevel {
    /// Level A (minimum)
    A,
    /// Level AA (mid-range)
    AA,
    /// Level AAA (highest)
    AAA,
}

impl std::fmt::Display for WCAGLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WCAGLevel::A => write!(f, "A"),
            WCAGLevel::AA => write!(f, "AA"),
            WCAGLevel::AAA => write!(f, "AAA"),
        }
    }
}

/// WCAG principle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WCAGPrinciple {
    /// Perceivable - Information and user interface components must be presentable to users
    Perceivable,
    /// Operable - User interface components and navigation must be operable
    Operable,
    /// Understandable - Information and operation of user interface must be understandable
    Understandable,
    /// Robust - Content must be robust enough to be interpreted by a wide variety of user agents
    Robust,
}

impl std::fmt::Display for WCAGPrinciple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WCAGPrinciple::Perceivable => write!(f, "Perceivable"),
            WCAGPrinciple::Operable => write!(f, "Operable"),
            WCAGPrinciple::Understandable => write!(f, "Understandable"),
            WCAGPrinciple::Robust => write!(f, "Robust"),
        }
    }
}

/// WCAG success criterion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WCAGCriterion {
    /// Criterion identifier (e.g., "1.1.1")
    pub id: String,

    /// Criterion name
    pub name: String,

    /// WCAG level
    pub level: WCAGLevel,

    /// WCAG principle
    pub principle: WCAGPrinciple,

    /// Description of the criterion
    pub description: String,

    /// URL to WCAG documentation
    pub url: String,
}

/// Get all WCAG 2.1 success criteria
pub fn get_all_criteria() -> Vec<WCAGCriterion> {
    vec![
        // Perceivable - Level A
        WCAGCriterion {
            id: "1.1.1".to_string(),
            name: "Non-text Content".to_string(),
            level: WCAGLevel::A,
            principle: WCAGPrinciple::Perceivable,
            description: "All non-text content has a text alternative that serves the equivalent purpose.".to_string(),
            url: "https://www.w3.org/WAI/WCAG21/Understanding/non-text-content.html".to_string(),
        },
        WCAGCriterion {
            id: "1.3.1".to_string(),
            name: "Info and Relationships".to_string(),
            level: WCAGLevel::A,
            principle: WCAGPrinciple::Perceivable,
            description: "Information, structure, and relationships can be programmatically determined.".to_string(),
            url: "https://www.w3.org/WAI/WCAG21/Understanding/info-and-relationships.html".to_string(),
        },
        WCAGCriterion {
            id: "1.3.2".to_string(),
            name: "Meaningful Sequence".to_string(),
            level: WCAGLevel::A,
            principle: WCAGPrinciple::Perceivable,
            description: "The correct reading sequence can be programmatically determined.".to_string(),
            url: "https://www.w3.org/WAI/WCAG21/Understanding/meaningful-sequence.html".to_string(),
        },
        WCAGCriterion {
            id: "1.3.3".to_string(),
            name: "Sensory Characteristics".to_string(),
            level: WCAGLevel::A,
            principle: WCAGPrinciple::Perceivable,
            description: "Instructions do not rely solely on sensory characteristics.".to_string(),
            url: "https://www.w3.org/WAI/WCAG21/Understanding/sensory-characteristics.html".to_string(),
        },
        WCAGCriterion {
            id: "1.4.1".to_string(),
            name: "Use of Color".to_string(),
            level: WCAGLevel::A,
            principle: WCAGPrinciple::Perceivable,
            description: "Color is not the only visual means of conveying information.".to_string(),
            url: "https://www.w3.org/WAI/WCAG21/Understanding/use-of-color.html".to_string(),
        },
        WCAGCriterion {
            id: "1.4.2".to_string(),
            name: "Audio Control".to_string(),
            level: WCAGLevel::A,
            principle: WCAGPrinciple::Perceivable,
            description: "A mechanism is available to pause or stop audio.".to_string(),
            url: "https://www.w3.org/WAI/WCAG21/Understanding/audio-control.html".to_string(),
        },

        // Perceivable - Level AA
        WCAGCriterion {
            id: "1.3.4".to_string(),
            name: "Orientation".to_string(),
            level: WCAGLevel::AA,
            principle: WCAGPrinciple::Perceivable,
            description: "Content does not restrict its view and operation to a single display orientation.".to_string(),
            url: "https://www.w3.org/WAI/WCAG21/Understanding/orientation.html".to_string(),
        },
        WCAGCriterion {
            id: "1.4.3".to_string(),
            name: "Contrast (Minimum)".to_string(),
            level: WCAGLevel::AA,
            principle: WCAGPrinciple::Perceivable,
            description: "Text has a contrast ratio of at least 4.5:1.".to_string(),
            url: "https://www.w3.org/WAI/WCAG21/Understanding/contrast-minimum.html".to_string(),
        },
        WCAGCriterion {
            id: "1.4.4".to_string(),
            name: "Resize Text".to_string(),
            level: WCAGLevel::AA,
            principle: WCAGPrinciple::Perceivable,
            description: "Text can be resized up to 200% without loss of content or functionality.".to_string(),
            url: "https://www.w3.org/WAI/WCAG21/Understanding/resize-text.html".to_string(),
        },
        WCAGCriterion {
            id: "1.4.5".to_string(),
            name: "Images of Text".to_string(),
            level: WCAGLevel::AA,
            principle: WCAGPrinciple::Perceivable,
            description: "Text is used rather than images of text.".to_string(),
            url: "https://www.w3.org/WAI/WCAG21/Understanding/images-of-text.html".to_string(),
        },
        WCAGCriterion {
            id: "1.4.10".to_string(),
            name: "Reflow".to_string(),
            level: WCAGLevel::AA,
            principle: WCAGPrinciple::Perceivable,
            description: "Content can be presented without loss of information or functionality, and without requiring scrolling in two dimensions.".to_string(),
            url: "https://www.w3.org/WAI/WCAG21/Understanding/reflow.html".to_string(),
        },
        WCAGCriterion {
            id: "1.4.11".to_string(),
            name: "Non-text Contrast".to_string(),
            level: WCAGLevel::AA,
            principle: WCAGPrinciple::Perceivable,
            description: "Visual presentation of UI components has a contrast ratio of at least 3:1.".to_string(),
            url: "https://www.w3.org/WAI/WCAG21/Understanding/non-text-contrast.html".to_string(),
        },
        WCAGCriterion {
            id: "1.4.12".to_string(),
            name: "Text Spacing".to_string(),
            level: WCAGLevel::AA,
            principle: WCAGPrinciple::Perceivable,
            description: "No loss of content or functionality occurs when text spacing is adjusted.".to_string(),
            url: "https://www.w3.org/WAI/WCAG21/Understanding/text-spacing.html".to_string(),
        },
        WCAGCriterion {
            id: "1.4.13".to_string(),
            name: "Content on Hover or Focus".to_string(),
            level: WCAGLevel::AA,
            principle: WCAGPrinciple::Perceivable,
            description: "Additional content triggered by hover or focus is dismissible, hoverable, and persistent.".to_string(),
            url: "https://www.w3.org/WAI/WCAG21/Understanding/content-on-hover-or-focus.html".to_string(),
        },

        // Operable - Level A
        WCAGCriterion {
            id: "2.1.1".to_string(),
            name: "Keyboard".to_string(),
            level: WCAGLevel::A,
            principle: WCAGPrinciple::Operable,
            description: "All functionality is available from a keyboard.".to_string(),
            url: "https://www.w3.org/WAI/WCAG21/Understanding/keyboard.html".to_string(),
        },
        WCAGCriterion {
            id: "2.1.2".to_string(),
            name: "No Keyboard Trap".to_string(),
            level: WCAGLevel::A,
            principle: WCAGPrinciple::Operable,
            description: "Keyboard focus is not trapped.".to_string(),
            url: "https://www.w3.org/WAI/WCAG21/Understanding/no-keyboard-trap.html".to_string(),
        },
        WCAGCriterion {
            id: "2.4.1".to_string(),
            name: "Bypass Blocks".to_string(),
            level: WCAGLevel::A,
            principle: WCAGPrinciple::Operable,
            description: "A mechanism is available to bypass blocks of content.".to_string(),
            url: "https://www.w3.org/WAI/WCAG21/Understanding/bypass-blocks.html".to_string(),
        },
        WCAGCriterion {
            id: "2.4.2".to_string(),
            name: "Page Titled".to_string(),
            level: WCAGLevel::A,
            principle: WCAGPrinciple::Operable,
            description: "Web pages have titles that describe topic or purpose.".to_string(),
            url: "https://www.w3.org/WAI/WCAG21/Understanding/page-titled.html".to_string(),
        },
        WCAGCriterion {
            id: "2.4.3".to_string(),
            name: "Focus Order".to_string(),
            level: WCAGLevel::A,
            principle: WCAGPrinciple::Operable,
            description: "Focus order preserves meaning and operability.".to_string(),
            url: "https://www.w3.org/WAI/WCAG21/Understanding/focus-order.html".to_string(),
        },
        WCAGCriterion {
            id: "2.4.4".to_string(),
            name: "Link Purpose (In Context)".to_string(),
            level: WCAGLevel::A,
            principle: WCAGPrinciple::Operable,
            description: "The purpose of each link can be determined from the link text or context.".to_string(),
            url: "https://www.w3.org/WAI/WCAG21/Understanding/link-purpose-in-context.html".to_string(),
        },

        // Operable - Level AA
        WCAGCriterion {
            id: "2.4.5".to_string(),
            name: "Multiple Ways".to_string(),
            level: WCAGLevel::AA,
            principle: WCAGPrinciple::Operable,
            description: "More than one way is available to locate a page.".to_string(),
            url: "https://www.w3.org/WAI/WCAG21/Understanding/multiple-ways.html".to_string(),
        },
        WCAGCriterion {
            id: "2.4.6".to_string(),
            name: "Headings and Labels".to_string(),
            level: WCAGLevel::AA,
            principle: WCAGPrinciple::Operable,
            description: "Headings and labels describe topic or purpose.".to_string(),
            url: "https://www.w3.org/WAI/WCAG21/Understanding/headings-and-labels.html".to_string(),
        },
        WCAGCriterion {
            id: "2.4.7".to_string(),
            name: "Focus Visible".to_string(),
            level: WCAGLevel::AA,
            principle: WCAGPrinciple::Operable,
            description: "Keyboard focus indicator is visible.".to_string(),
            url: "https://www.w3.org/WAI/WCAG21/Understanding/focus-visible.html".to_string(),
        },

        // Understandable - Level A
        WCAGCriterion {
            id: "3.1.1".to_string(),
            name: "Language of Page".to_string(),
            level: WCAGLevel::A,
            principle: WCAGPrinciple::Understandable,
            description: "The default human language of each page can be programmatically determined.".to_string(),
            url: "https://www.w3.org/WAI/WCAG21/Understanding/language-of-page.html".to_string(),
        },
        WCAGCriterion {
            id: "3.2.1".to_string(),
            name: "On Focus".to_string(),
            level: WCAGLevel::A,
            principle: WCAGPrinciple::Understandable,
            description: "Receiving focus does not initiate a change of context.".to_string(),
            url: "https://www.w3.org/WAI/WCAG21/Understanding/on-focus.html".to_string(),
        },
        WCAGCriterion {
            id: "3.2.2".to_string(),
            name: "On Input".to_string(),
            level: WCAGLevel::A,
            principle: WCAGPrinciple::Understandable,
            description: "Changing the setting of a UI component does not automatically cause a change of context.".to_string(),
            url: "https://www.w3.org/WAI/WCAG21/Understanding/on-input.html".to_string(),
        },
        WCAGCriterion {
            id: "3.3.1".to_string(),
            name: "Error Identification".to_string(),
            level: WCAGLevel::A,
            principle: WCAGPrinciple::Understandable,
            description: "Input errors are identified and described to the user.".to_string(),
            url: "https://www.w3.org/WAI/WCAG21/Understanding/error-identification.html".to_string(),
        },
        WCAGCriterion {
            id: "3.3.2".to_string(),
            name: "Labels or Instructions".to_string(),
            level: WCAGLevel::A,
            principle: WCAGPrinciple::Understandable,
            description: "Labels or instructions are provided when content requires user input.".to_string(),
            url: "https://www.w3.org/WAI/WCAG21/Understanding/labels-or-instructions.html".to_string(),
        },

        // Understandable - Level AA
        WCAGCriterion {
            id: "3.1.2".to_string(),
            name: "Language of Parts".to_string(),
            level: WCAGLevel::AA,
            principle: WCAGPrinciple::Understandable,
            description: "The human language of each passage can be programmatically determined.".to_string(),
            url: "https://www.w3.org/WAI/WCAG21/Understanding/language-of-parts.html".to_string(),
        },
        WCAGCriterion {
            id: "3.2.3".to_string(),
            name: "Consistent Navigation".to_string(),
            level: WCAGLevel::AA,
            principle: WCAGPrinciple::Understandable,
            description: "Navigational mechanisms are consistent.".to_string(),
            url: "https://www.w3.org/WAI/WCAG21/Understanding/consistent-navigation.html".to_string(),
        },
        WCAGCriterion {
            id: "3.2.4".to_string(),
            name: "Consistent Identification".to_string(),
            level: WCAGLevel::AA,
            principle: WCAGPrinciple::Understandable,
            description: "Components with the same functionality are identified consistently.".to_string(),
            url: "https://www.w3.org/WAI/WCAG21/Understanding/consistent-identification.html".to_string(),
        },
        WCAGCriterion {
            id: "3.3.3".to_string(),
            name: "Error Suggestion".to_string(),
            level: WCAGLevel::AA,
            principle: WCAGPrinciple::Understandable,
            description: "Suggestions are provided for fixing input errors.".to_string(),
            url: "https://www.w3.org/WAI/WCAG21/Understanding/error-suggestion.html".to_string(),
        },
        WCAGCriterion {
            id: "3.3.4".to_string(),
            name: "Error Prevention (Legal, Financial, Data)".to_string(),
            level: WCAGLevel::AA,
            principle: WCAGPrinciple::Understandable,
            description: "Submissions are reversible, checked, or confirmed.".to_string(),
            url: "https://www.w3.org/WAI/WCAG21/Understanding/error-prevention-legal-financial-data.html".to_string(),
        },

        // Robust - Level A
        WCAGCriterion {
            id: "4.1.1".to_string(),
            name: "Parsing".to_string(),
            level: WCAGLevel::A,
            principle: WCAGPrinciple::Robust,
            description: "Content can be parsed unambiguously.".to_string(),
            url: "https://www.w3.org/WAI/WCAG21/Understanding/parsing.html".to_string(),
        },
        WCAGCriterion {
            id: "4.1.2".to_string(),
            name: "Name, Role, Value".to_string(),
            level: WCAGLevel::A,
            principle: WCAGPrinciple::Robust,
            description: "Name, role, and value can be programmatically determined for all UI components.".to_string(),
            url: "https://www.w3.org/WAI/WCAG21/Understanding/name-role-value.html".to_string(),
        },

        // Robust - Level AA
        WCAGCriterion {
            id: "4.1.3".to_string(),
            name: "Status Messages".to_string(),
            level: WCAGLevel::AA,
            principle: WCAGPrinciple::Robust,
            description: "Status messages can be programmatically determined.".to_string(),
            url: "https://www.w3.org/WAI/WCAG21/Understanding/status-messages.html".to_string(),
        },
    ]
}

/// Get WCAG criterion by ID
pub fn get_criterion_by_id(id: &str) -> Option<WCAGCriterion> {
    get_all_criteria().into_iter().find(|c| c.id == id)
}

/// Get WCAG criteria for a specific level (includes all lower levels)
pub fn get_criteria_for_level(level: WCAGLevel) -> Vec<WCAGCriterion> {
    get_all_criteria()
        .into_iter()
        .filter(|c| match level {
            WCAGLevel::A => c.level == WCAGLevel::A,
            WCAGLevel::AA => c.level == WCAGLevel::A || c.level == WCAGLevel::AA,
            WCAGLevel::AAA => true,
        })
        .collect()
}

/// Get WCAG criteria for a specific principle
pub fn get_criteria_for_principle(principle: WCAGPrinciple) -> Vec<WCAGCriterion> {
    get_all_criteria()
        .into_iter()
        .filter(|c| c.principle == principle)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_criteria() {
        let criteria = get_all_criteria();
        assert!(!criteria.is_empty());
    }

    #[test]
    fn test_get_criterion_by_id() {
        let criterion = get_criterion_by_id("1.1.1");
        assert!(criterion.is_some());
        assert_eq!(criterion.unwrap().name, "Non-text Content");
    }

    #[test]
    fn test_get_criteria_for_level() {
        let criteria_a = get_criteria_for_level(WCAGLevel::A);
        let criteria_aa = get_criteria_for_level(WCAGLevel::AA);

        assert!(!criteria_a.is_empty());
        assert!(criteria_aa.len() > criteria_a.len());
    }
}
