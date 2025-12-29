use crate::types::{Rule, WCAGLevel, Principle};

/// Get all WCAG 2.1 Level A rules
pub fn get_level_a_rules() -> Vec<Rule> {
    vec![
        // 1.1.1 Non-text Content (Level A)
        Rule {
            id: "image-alt".to_string(),
            name: "Images must have alternative text".to_string(),
            description: "All img elements must have an alt attribute. Images convey information that must be available to screen reader users.".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Perceivable,
            guideline: "1.1".to_string(),
            success_criterion: "non-text-content".to_string(),
            tags: vec!["images".to_string(), "alt-text".to_string()],
        },
        Rule {
            id: "input-image-alt".to_string(),
            name: "Image buttons must have alternative text".to_string(),
            description: "Input elements with type='image' must have an alt attribute.".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Perceivable,
            guideline: "1.1".to_string(),
            success_criterion: "non-text-content".to_string(),
            tags: vec!["forms".to_string(), "buttons".to_string()],
        },
        Rule {
            id: "object-alt".to_string(),
            name: "Object elements must have text alternative".to_string(),
            description: "Object elements must have alternative text provided.".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Perceivable,
            guideline: "1.1".to_string(),
            success_criterion: "non-text-content".to_string(),
            tags: vec!["multimedia".to_string()],
        },
        Rule {
            id: "area-alt".to_string(),
            name: "Image map areas must have alternative text".to_string(),
            description: "Area elements must have an alt attribute.".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Perceivable,
            guideline: "1.1".to_string(),
            success_criterion: "non-text-content".to_string(),
            tags: vec!["images".to_string(), "image-maps".to_string()],
        },

        // 1.2.1 Audio-only and Video-only (Level A)
        Rule {
            id: "audio-caption".to_string(),
            name: "Audio elements must have captions or transcript".to_string(),
            description: "Audio-only content must have a text alternative (captions or transcript).".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Perceivable,
            guideline: "1.2".to_string(),
            success_criterion: "audio-only-and-video-only-prerecorded".to_string(),
            tags: vec!["audio".to_string(), "multimedia".to_string()],
        },
        Rule {
            id: "video-caption".to_string(),
            name: "Video elements must have captions".to_string(),
            description: "Video content must have synchronized captions.".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Perceivable,
            guideline: "1.2".to_string(),
            success_criterion: "captions-prerecorded".to_string(),
            tags: vec!["video".to_string(), "multimedia".to_string()],
        },

        // 1.3.1 Info and Relationships (Level A)
        Rule {
            id: "label".to_string(),
            name: "Form inputs must have labels".to_string(),
            description: "Every form input must have a properly associated label element.".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Perceivable,
            guideline: "1.3".to_string(),
            success_criterion: "info-and-relationships".to_string(),
            tags: vec!["forms".to_string(), "labels".to_string()],
        },
        Rule {
            id: "list".to_string(),
            name: "Lists must be properly marked up".to_string(),
            description: "List content must use proper list elements (ul, ol, dl).".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Perceivable,
            guideline: "1.3".to_string(),
            success_criterion: "info-and-relationships".to_string(),
            tags: vec!["structure".to_string(), "lists".to_string()],
        },
        Rule {
            id: "definition-list".to_string(),
            name: "Definition lists must only contain dt and dd elements".to_string(),
            description: "dl elements must only contain properly ordered dt and dd elements.".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Perceivable,
            guideline: "1.3".to_string(),
            success_criterion: "info-and-relationships".to_string(),
            tags: vec!["structure".to_string(), "lists".to_string()],
        },
        Rule {
            id: "table-headers".to_string(),
            name: "Data tables must have headers".to_string(),
            description: "Tables used for data must have proper th elements.".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Perceivable,
            guideline: "1.3".to_string(),
            success_criterion: "info-and-relationships".to_string(),
            tags: vec!["tables".to_string()],
        },
        Rule {
            id: "th-has-data-cells".to_string(),
            name: "Table headers must have associated data cells".to_string(),
            description: "Each th element must have corresponding td elements.".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Perceivable,
            guideline: "1.3".to_string(),
            success_criterion: "info-and-relationships".to_string(),
            tags: vec!["tables".to_string()],
        },
        Rule {
            id: "td-headers-attr".to_string(),
            name: "Complex tables must use headers attribute".to_string(),
            description: "In complex tables, td elements must use headers attribute to reference th elements.".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Perceivable,
            guideline: "1.3".to_string(),
            success_criterion: "info-and-relationships".to_string(),
            tags: vec!["tables".to_string()],
        },

        // 1.3.2 Meaningful Sequence (Level A)
        Rule {
            id: "tabindex".to_string(),
            name: "Tabindex must not be used with positive values".to_string(),
            description: "Positive tabindex values disrupt natural tab order and should not be used.".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Perceivable,
            guideline: "1.3".to_string(),
            success_criterion: "meaningful-sequence".to_string(),
            tags: vec!["keyboard".to_string(), "navigation".to_string()],
        },

        // 1.3.3 Sensory Characteristics (Level A)
        Rule {
            id: "sensory-characteristics".to_string(),
            name: "Instructions must not rely solely on sensory characteristics".to_string(),
            description: "Instructions must not depend solely on shape, size, visual location, orientation, or sound.".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Perceivable,
            guideline: "1.3".to_string(),
            success_criterion: "sensory-characteristics".to_string(),
            tags: vec!["instructions".to_string()],
        },

        // 1.4.1 Use of Color (Level A)
        Rule {
            id: "color-alone".to_string(),
            name: "Color must not be the only visual means of conveying information".to_string(),
            description: "Information conveyed with color must also be available through other visual means.".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Perceivable,
            guideline: "1.4".to_string(),
            success_criterion: "use-of-color".to_string(),
            tags: vec!["color".to_string()],
        },

        // 1.4.2 Audio Control (Level A)
        Rule {
            id: "audio-control".to_string(),
            name: "Auto-playing audio must be controllable".to_string(),
            description: "Audio that plays automatically for more than 3 seconds must have a control to pause or stop it.".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Perceivable,
            guideline: "1.4".to_string(),
            success_criterion: "audio-control".to_string(),
            tags: vec!["audio".to_string(), "controls".to_string()],
        },

        // 2.1.1 Keyboard (Level A)
        Rule {
            id: "keyboard".to_string(),
            name: "All functionality must be keyboard accessible".to_string(),
            description: "All interactive elements must be operable through a keyboard interface.".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Operable,
            guideline: "2.1".to_string(),
            success_criterion: "keyboard".to_string(),
            tags: vec!["keyboard".to_string()],
        },
        Rule {
            id: "onclick-has-keyboard-equivalent".to_string(),
            name: "Click handlers must have keyboard equivalent".to_string(),
            description: "Elements with onclick must also have onkeydown or onkeyup for keyboard access.".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Operable,
            guideline: "2.1".to_string(),
            success_criterion: "keyboard".to_string(),
            tags: vec!["keyboard".to_string(), "javascript".to_string()],
        },

        // 2.1.2 No Keyboard Trap (Level A)
        Rule {
            id: "no-keyboard-trap".to_string(),
            name: "Keyboard focus must not be trapped".to_string(),
            description: "Users must be able to move focus away from any element using only keyboard.".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Operable,
            guideline: "2.1".to_string(),
            success_criterion: "no-keyboard-trap".to_string(),
            tags: vec!["keyboard".to_string(), "focus".to_string()],
        },

        // 2.2.1 Timing Adjustable (Level A)
        Rule {
            id: "meta-refresh".to_string(),
            name: "Meta refresh must not be used".to_string(),
            description: "Pages must not use meta refresh to automatically redirect or refresh.".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Operable,
            guideline: "2.2".to_string(),
            success_criterion: "timing-adjustable".to_string(),
            tags: vec!["timing".to_string(), "redirects".to_string()],
        },

        // 2.2.2 Pause, Stop, Hide (Level A)
        Rule {
            id: "blink".to_string(),
            name: "Blinking content must be avoidable".to_string(),
            description: "Content must not blink or there must be a mechanism to stop it.".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Operable,
            guideline: "2.2".to_string(),
            success_criterion: "pause-stop-hide".to_string(),
            tags: vec!["animation".to_string()],
        },
        Rule {
            id: "marquee".to_string(),
            name: "Marquee elements must not be used".to_string(),
            description: "The marquee element is deprecated and creates accessibility issues.".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Operable,
            guideline: "2.2".to_string(),
            success_criterion: "pause-stop-hide".to_string(),
            tags: vec!["animation".to_string(), "deprecated".to_string()],
        },

        // 2.3.1 Three Flashes or Below Threshold (Level A)
        Rule {
            id: "no-flashing".to_string(),
            name: "Content must not flash more than 3 times per second".to_string(),
            description: "Flashing content can cause seizures and must be avoided or limited.".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Operable,
            guideline: "2.3".to_string(),
            success_criterion: "three-flashes-or-below-threshold".to_string(),
            tags: vec!["seizures".to_string(), "flashing".to_string()],
        },

        // 2.4.1 Bypass Blocks (Level A)
        Rule {
            id: "skip-link".to_string(),
            name: "Pages must have a skip navigation link".to_string(),
            description: "A mechanism must exist to bypass repeated blocks of content.".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Operable,
            guideline: "2.4".to_string(),
            success_criterion: "bypass-blocks".to_string(),
            tags: vec!["navigation".to_string(), "skip-links".to_string()],
        },

        // 2.4.2 Page Titled (Level A)
        Rule {
            id: "document-title".to_string(),
            name: "Pages must have a title".to_string(),
            description: "Every HTML document must have a descriptive title element.".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Operable,
            guideline: "2.4".to_string(),
            success_criterion: "page-titled".to_string(),
            tags: vec!["page-structure".to_string(), "titles".to_string()],
        },

        // 2.4.3 Focus Order (Level A)
        Rule {
            id: "focus-order".to_string(),
            name: "Focus order must be logical".to_string(),
            description: "When navigating by keyboard, focus order must be logical and intuitive.".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Operable,
            guideline: "2.4".to_string(),
            success_criterion: "focus-order".to_string(),
            tags: vec!["keyboard".to_string(), "focus".to_string()],
        },

        // 2.4.4 Link Purpose (Level A)
        Rule {
            id: "link-name".to_string(),
            name: "Links must have discernible text".to_string(),
            description: "Every link must have accessible text that describes its purpose.".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Operable,
            guideline: "2.4".to_string(),
            success_criterion: "link-purpose-in-context".to_string(),
            tags: vec!["links".to_string(), "link-text".to_string()],
        },

        // 3.1.1 Language of Page (Level A)
        Rule {
            id: "html-has-lang".to_string(),
            name: "HTML element must have lang attribute".to_string(),
            description: "The html element must have a lang attribute to identify the page language.".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Understandable,
            guideline: "3.1".to_string(),
            success_criterion: "language-of-page".to_string(),
            tags: vec!["language".to_string()],
        },
        Rule {
            id: "html-lang-valid".to_string(),
            name: "HTML lang attribute must be valid".to_string(),
            description: "The lang attribute must contain a valid language code.".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Understandable,
            guideline: "3.1".to_string(),
            success_criterion: "language-of-page".to_string(),
            tags: vec!["language".to_string()],
        },

        // 3.2.1 On Focus (Level A)
        Rule {
            id: "no-onchange".to_string(),
            name: "Select elements should not change context on change".to_string(),
            description: "Form controls should not automatically submit or change context when receiving focus.".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Understandable,
            guideline: "3.2".to_string(),
            success_criterion: "on-focus".to_string(),
            tags: vec!["forms".to_string(), "context-changes".to_string()],
        },

        // 3.2.2 On Input (Level A)
        Rule {
            id: "no-auto-submit".to_string(),
            name: "Forms should not auto-submit on input".to_string(),
            description: "Changing form values should not automatically cause form submission.".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Understandable,
            guideline: "3.2".to_string(),
            success_criterion: "on-input".to_string(),
            tags: vec!["forms".to_string(), "auto-submit".to_string()],
        },

        // 3.3.1 Error Identification (Level A)
        Rule {
            id: "error-identification".to_string(),
            name: "Form errors must be clearly identified".to_string(),
            description: "When form validation fails, errors must be identified and described to the user.".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Understandable,
            guideline: "3.3".to_string(),
            success_criterion: "error-identification".to_string(),
            tags: vec!["forms".to_string(), "errors".to_string()],
        },

        // 3.3.2 Labels or Instructions (Level A)
        Rule {
            id: "label-title-only".to_string(),
            name: "Labels must not rely solely on title attribute".to_string(),
            description: "Form labels must be visible text, not just title attributes.".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Understandable,
            guideline: "3.3".to_string(),
            success_criterion: "labels-or-instructions".to_string(),
            tags: vec!["forms".to_string(), "labels".to_string()],
        },

        // 4.1.1 Parsing (Level A)
        Rule {
            id: "duplicate-id".to_string(),
            name: "IDs must be unique".to_string(),
            description: "Each id attribute value must be unique within the page.".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Robust,
            guideline: "4.1".to_string(),
            success_criterion: "parsing".to_string(),
            tags: vec!["parsing".to_string(), "html".to_string()],
        },

        // 4.1.2 Name, Role, Value (Level A)
        Rule {
            id: "button-name".to_string(),
            name: "Buttons must have accessible text".to_string(),
            description: "Button elements must have visible text or an aria-label.".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Robust,
            guideline: "4.1".to_string(),
            success_criterion: "name-role-value".to_string(),
            tags: vec!["buttons".to_string(), "aria".to_string()],
        },
        Rule {
            id: "aria-required-attr".to_string(),
            name: "ARIA roles must have required attributes".to_string(),
            description: "Elements with ARIA roles must have all required ARIA attributes.".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Robust,
            guideline: "4.1".to_string(),
            success_criterion: "name-role-value".to_string(),
            tags: vec!["aria".to_string()],
        },
        Rule {
            id: "aria-valid-attr-value".to_string(),
            name: "ARIA attributes must have valid values".to_string(),
            description: "ARIA attribute values must conform to allowed value specifications.".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Robust,
            guideline: "4.1".to_string(),
            success_criterion: "name-role-value".to_string(),
            tags: vec!["aria".to_string()],
        },
        Rule {
            id: "aria-valid-attr".to_string(),
            name: "ARIA attributes must be valid".to_string(),
            description: "Elements must only use valid ARIA attributes.".to_string(),
            level: WCAGLevel::A,
            principle: Principle::Robust,
            guideline: "4.1".to_string(),
            success_criterion: "name-role-value".to_string(),
            tags: vec!["aria".to_string()],
        },
    ]
}
