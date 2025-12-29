use crate::types::{Rule, WCAGLevel, Principle};

/// Get all WCAG 2.1 Level AA rules
pub fn get_level_aa_rules() -> Vec<Rule> {
    vec![
        // 1.2.4 Captions (Live) (Level AA)
        Rule {
            id: "live-caption".to_string(),
            name: "Live audio must have captions".to_string(),
            description: "Live audio content in synchronized media must have captions.".to_string(),
            level: WCAGLevel::AA,
            principle: Principle::Perceivable,
            guideline: "1.2".to_string(),
            success_criterion: "captions-live".to_string(),
            tags: vec!["audio".to_string(), "live".to_string(), "captions".to_string()],
        },

        // 1.2.5 Audio Description (Level AA)
        Rule {
            id: "audio-description".to_string(),
            name: "Video must have audio description".to_string(),
            description: "Prerecorded video must have audio description of visual content.".to_string(),
            level: WCAGLevel::AA,
            principle: Principle::Perceivable,
            guideline: "1.2".to_string(),
            success_criterion: "audio-description-prerecorded".to_string(),
            tags: vec!["video".to_string(), "audio-description".to_string()],
        },

        // 1.3.4 Orientation (Level AA)
        Rule {
            id: "orientation".to_string(),
            name: "Content must not restrict orientation".to_string(),
            description: "Content must not be restricted to a single display orientation (portrait or landscape).".to_string(),
            level: WCAGLevel::AA,
            principle: Principle::Perceivable,
            guideline: "1.3".to_string(),
            success_criterion: "orientation".to_string(),
            tags: vec!["mobile".to_string(), "orientation".to_string()],
        },

        // 1.3.5 Identify Input Purpose (Level AA)
        Rule {
            id: "autocomplete".to_string(),
            name: "Input fields must have autocomplete attribute".to_string(),
            description: "Form inputs for personal information should have appropriate autocomplete values.".to_string(),
            level: WCAGLevel::AA,
            principle: Principle::Perceivable,
            guideline: "1.3".to_string(),
            success_criterion: "identify-input-purpose".to_string(),
            tags: vec!["forms".to_string(), "autocomplete".to_string()],
        },

        // 1.4.3 Contrast (Minimum) (Level AA)
        Rule {
            id: "color-contrast".to_string(),
            name: "Text must have sufficient color contrast".to_string(),
            description: "Text and images of text must have a contrast ratio of at least 4.5:1.".to_string(),
            level: WCAGLevel::AA,
            principle: Principle::Perceivable,
            guideline: "1.4".to_string(),
            success_criterion: "contrast-minimum".to_string(),
            tags: vec!["color".to_string(), "contrast".to_string()],
        },
        Rule {
            id: "large-text-contrast".to_string(),
            name: "Large text must have sufficient contrast".to_string(),
            description: "Large text (18pt+ or 14pt+ bold) must have a contrast ratio of at least 3:1.".to_string(),
            level: WCAGLevel::AA,
            principle: Principle::Perceivable,
            guideline: "1.4".to_string(),
            success_criterion: "contrast-minimum".to_string(),
            tags: vec!["color".to_string(), "contrast".to_string()],
        },

        // 1.4.4 Resize Text (Level AA)
        Rule {
            id: "resize-text".to_string(),
            name: "Text must be resizable to 200%".to_string(),
            description: "Text must be resizable up to 200% without loss of content or functionality.".to_string(),
            level: WCAGLevel::AA,
            principle: Principle::Perceivable,
            guideline: "1.4".to_string(),
            success_criterion: "resize-text".to_string(),
            tags: vec!["text".to_string(), "zoom".to_string()],
        },

        // 1.4.5 Images of Text (Level AA)
        Rule {
            id: "image-of-text".to_string(),
            name: "Images of text should be avoided".to_string(),
            description: "Text should be used instead of images of text unless essential.".to_string(),
            level: WCAGLevel::AA,
            principle: Principle::Perceivable,
            guideline: "1.4".to_string(),
            success_criterion: "images-of-text".to_string(),
            tags: vec!["images".to_string(), "text".to_string()],
        },

        // 1.4.10 Reflow (Level AA)
        Rule {
            id: "reflow".to_string(),
            name: "Content must reflow without horizontal scrolling".to_string(),
            description: "Content must be viewable at 320px width without horizontal scrolling.".to_string(),
            level: WCAGLevel::AA,
            principle: Principle::Perceivable,
            guideline: "1.4".to_string(),
            success_criterion: "reflow".to_string(),
            tags: vec!["responsive".to_string(), "mobile".to_string()],
        },

        // 1.4.11 Non-text Contrast (Level AA)
        Rule {
            id: "non-text-contrast".to_string(),
            name: "UI components must have sufficient contrast".to_string(),
            description: "User interface components and graphical objects must have a contrast ratio of at least 3:1.".to_string(),
            level: WCAGLevel::AA,
            principle: Principle::Perceivable,
            guideline: "1.4".to_string(),
            success_criterion: "non-text-contrast".to_string(),
            tags: vec!["color".to_string(), "contrast".to_string(), "ui".to_string()],
        },

        // 1.4.12 Text Spacing (Level AA)
        Rule {
            id: "text-spacing".to_string(),
            name: "Text must be readable with increased spacing".to_string(),
            description: "Content must remain readable when text spacing is increased.".to_string(),
            level: WCAGLevel::AA,
            principle: Principle::Perceivable,
            guideline: "1.4".to_string(),
            success_criterion: "text-spacing".to_string(),
            tags: vec!["text".to_string(), "spacing".to_string()],
        },

        // 1.4.13 Content on Hover or Focus (Level AA)
        Rule {
            id: "hover-focus-content".to_string(),
            name: "Hover/focus content must be dismissible and hoverable".to_string(),
            description: "Content that appears on hover or focus must be dismissible, hoverable, and persistent.".to_string(),
            level: WCAGLevel::AA,
            principle: Principle::Perceivable,
            guideline: "1.4".to_string(),
            success_criterion: "content-on-hover-or-focus".to_string(),
            tags: vec!["hover".to_string(), "focus".to_string(), "tooltips".to_string()],
        },

        // 2.4.5 Multiple Ways (Level AA)
        Rule {
            id: "multiple-ways".to_string(),
            name: "Multiple ways to find pages must exist".to_string(),
            description: "More than one way must be available to locate pages (e.g., navigation, search, sitemap).".to_string(),
            level: WCAGLevel::AA,
            principle: Principle::Operable,
            guideline: "2.4".to_string(),
            success_criterion: "multiple-ways".to_string(),
            tags: vec!["navigation".to_string(), "search".to_string()],
        },

        // 2.4.6 Headings and Labels (Level AA)
        Rule {
            id: "heading-order".to_string(),
            name: "Headings must be in correct order".to_string(),
            description: "Heading levels should not be skipped (e.g., h1 to h3 without h2).".to_string(),
            level: WCAGLevel::AA,
            principle: Principle::Operable,
            guideline: "2.4".to_string(),
            success_criterion: "headings-and-labels".to_string(),
            tags: vec!["headings".to_string(), "structure".to_string()],
        },
        Rule {
            id: "empty-heading".to_string(),
            name: "Headings must not be empty".to_string(),
            description: "Heading elements must contain text content.".to_string(),
            level: WCAGLevel::AA,
            principle: Principle::Operable,
            guideline: "2.4".to_string(),
            success_criterion: "headings-and-labels".to_string(),
            tags: vec!["headings".to_string()],
        },

        // 2.4.7 Focus Visible (Level AA)
        Rule {
            id: "focus-visible".to_string(),
            name: "Keyboard focus must be visible".to_string(),
            description: "Any keyboard operable interface must have a visible focus indicator.".to_string(),
            level: WCAGLevel::AA,
            principle: Principle::Operable,
            guideline: "2.4".to_string(),
            success_criterion: "focus-visible".to_string(),
            tags: vec!["keyboard".to_string(), "focus".to_string()],
        },

        // 3.1.2 Language of Parts (Level AA)
        Rule {
            id: "lang-change".to_string(),
            name: "Language changes must be marked".to_string(),
            description: "When content language changes, it must be indicated with lang attribute.".to_string(),
            level: WCAGLevel::AA,
            principle: Principle::Understandable,
            guideline: "3.1".to_string(),
            success_criterion: "language-of-parts".to_string(),
            tags: vec!["language".to_string()],
        },

        // 3.2.3 Consistent Navigation (Level AA)
        Rule {
            id: "consistent-navigation".to_string(),
            name: "Navigation must be consistent".to_string(),
            description: "Navigational mechanisms must be in the same relative order across pages.".to_string(),
            level: WCAGLevel::AA,
            principle: Principle::Understandable,
            guideline: "3.2".to_string(),
            success_criterion: "consistent-navigation".to_string(),
            tags: vec!["navigation".to_string(), "consistency".to_string()],
        },

        // 3.2.4 Consistent Identification (Level AA)
        Rule {
            id: "consistent-identification".to_string(),
            name: "Components must be identified consistently".to_string(),
            description: "Components with the same functionality must be identified consistently.".to_string(),
            level: WCAGLevel::AA,
            principle: Principle::Understandable,
            guideline: "3.2".to_string(),
            success_criterion: "consistent-identification".to_string(),
            tags: vec!["consistency".to_string()],
        },

        // 3.3.3 Error Suggestion (Level AA)
        Rule {
            id: "error-suggestion".to_string(),
            name: "Error messages must provide suggestions".to_string(),
            description: "When errors are detected, suggestions for correction should be provided.".to_string(),
            level: WCAGLevel::AA,
            principle: Principle::Understandable,
            guideline: "3.3".to_string(),
            success_criterion: "error-suggestion".to_string(),
            tags: vec!["forms".to_string(), "errors".to_string()],
        },

        // 3.3.4 Error Prevention (Legal, Financial, Data) (Level AA)
        Rule {
            id: "error-prevention".to_string(),
            name: "Important submissions must be reversible or confirmed".to_string(),
            description: "Legal, financial, or data submissions must be reversible, verifiable, or confirmed.".to_string(),
            level: WCAGLevel::AA,
            principle: Principle::Understandable,
            guideline: "3.3".to_string(),
            success_criterion: "error-prevention-legal-financial-data".to_string(),
            tags: vec!["forms".to_string(), "transactions".to_string()],
        },

        // 4.1.3 Status Messages (Level AA)
        Rule {
            id: "status-messages".to_string(),
            name: "Status messages must be programmatically determinable".to_string(),
            description: "Status messages must be conveyed to assistive technologies using ARIA live regions.".to_string(),
            level: WCAGLevel::AA,
            principle: Principle::Robust,
            guideline: "4.1".to_string(),
            success_criterion: "status-messages".to_string(),
            tags: vec!["aria".to_string(), "live-regions".to_string()],
        },
    ]
}
