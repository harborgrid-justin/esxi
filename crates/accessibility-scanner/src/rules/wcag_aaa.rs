use crate::types::{Rule, WCAGLevel, Principle};

/// Get all WCAG 2.1 Level AAA rules
pub fn get_level_aaa_rules() -> Vec<Rule> {
    vec![
        // 1.2.6 Sign Language (Level AAA)
        Rule {
            id: "sign-language".to_string(),
            name: "Audio must have sign language interpretation".to_string(),
            description: "Prerecorded audio should provide sign language interpretation.".to_string(),
            level: WCAGLevel::AAA,
            principle: Principle::Perceivable,
            guideline: "1.2".to_string(),
            success_criterion: "sign-language-prerecorded".to_string(),
            tags: vec!["audio".to_string(), "sign-language".to_string()],
        },

        // 1.2.7 Extended Audio Description (Level AAA)
        Rule {
            id: "extended-audio-description".to_string(),
            name: "Video must have extended audio description".to_string(),
            description: "When pauses in audio are insufficient, extended audio description should be provided.".to_string(),
            level: WCAGLevel::AAA,
            principle: Principle::Perceivable,
            guideline: "1.2".to_string(),
            success_criterion: "extended-audio-description-prerecorded".to_string(),
            tags: vec!["video".to_string(), "audio-description".to_string()],
        },

        // 1.2.8 Media Alternative (Level AAA)
        Rule {
            id: "media-alternative".to_string(),
            name: "Media must have text alternative".to_string(),
            description: "Synchronized media should have a full text alternative.".to_string(),
            level: WCAGLevel::AAA,
            principle: Principle::Perceivable,
            guideline: "1.2".to_string(),
            success_criterion: "media-alternative-prerecorded".to_string(),
            tags: vec!["multimedia".to_string(), "transcripts".to_string()],
        },

        // 1.2.9 Audio-only (Live) (Level AAA)
        Rule {
            id: "live-audio-alternative".to_string(),
            name: "Live audio must have text alternative".to_string(),
            description: "Live audio-only content should have a text alternative.".to_string(),
            level: WCAGLevel::AAA,
            principle: Principle::Perceivable,
            guideline: "1.2".to_string(),
            success_criterion: "audio-only-live".to_string(),
            tags: vec!["audio".to_string(), "live".to_string()],
        },

        // 1.3.6 Identify Purpose (Level AAA)
        Rule {
            id: "identify-purpose".to_string(),
            name: "Purpose of UI components must be programmatically determined".to_string(),
            description: "The purpose of UI components should be identifiable through markup.".to_string(),
            level: WCAGLevel::AAA,
            principle: Principle::Perceivable,
            guideline: "1.3".to_string(),
            success_criterion: "identify-purpose".to_string(),
            tags: vec!["aria".to_string(), "semantics".to_string()],
        },

        // 1.4.6 Contrast (Enhanced) (Level AAA)
        Rule {
            id: "color-contrast-enhanced".to_string(),
            name: "Text must have enhanced color contrast".to_string(),
            description: "Text and images of text must have a contrast ratio of at least 7:1.".to_string(),
            level: WCAGLevel::AAA,
            principle: Principle::Perceivable,
            guideline: "1.4".to_string(),
            success_criterion: "contrast-enhanced".to_string(),
            tags: vec!["color".to_string(), "contrast".to_string()],
        },
        Rule {
            id: "large-text-contrast-enhanced".to_string(),
            name: "Large text must have enhanced contrast".to_string(),
            description: "Large text must have a contrast ratio of at least 4.5:1.".to_string(),
            level: WCAGLevel::AAA,
            principle: Principle::Perceivable,
            guideline: "1.4".to_string(),
            success_criterion: "contrast-enhanced".to_string(),
            tags: vec!["color".to_string(), "contrast".to_string()],
        },

        // 1.4.7 Low or No Background Audio (Level AAA)
        Rule {
            id: "background-audio".to_string(),
            name: "Background audio must be minimal".to_string(),
            description: "Speech audio should have minimal or no background sounds.".to_string(),
            level: WCAGLevel::AAA,
            principle: Principle::Perceivable,
            guideline: "1.4".to_string(),
            success_criterion: "low-or-no-background-audio".to_string(),
            tags: vec!["audio".to_string()],
        },

        // 1.4.8 Visual Presentation (Level AAA)
        Rule {
            id: "visual-presentation".to_string(),
            name: "Text blocks must be visually presentable".to_string(),
            description: "Users should be able to select colors, line spacing, and width for text blocks.".to_string(),
            level: WCAGLevel::AAA,
            principle: Principle::Perceivable,
            guideline: "1.4".to_string(),
            success_criterion: "visual-presentation".to_string(),
            tags: vec!["text".to_string(), "presentation".to_string()],
        },

        // 1.4.9 Images of Text (No Exception) (Level AAA)
        Rule {
            id: "no-images-of-text".to_string(),
            name: "Images of text must not be used".to_string(),
            description: "Text should be used instead of images of text except for logos.".to_string(),
            level: WCAGLevel::AAA,
            principle: Principle::Perceivable,
            guideline: "1.4".to_string(),
            success_criterion: "images-of-text-no-exception".to_string(),
            tags: vec!["images".to_string(), "text".to_string()],
        },

        // 2.1.3 Keyboard (No Exception) (Level AAA)
        Rule {
            id: "keyboard-no-exception".to_string(),
            name: "All functionality must be keyboard accessible (no exceptions)".to_string(),
            description: "All functionality must be operable through keyboard with no exceptions.".to_string(),
            level: WCAGLevel::AAA,
            principle: Principle::Operable,
            guideline: "2.1".to_string(),
            success_criterion: "keyboard-no-exception".to_string(),
            tags: vec!["keyboard".to_string()],
        },

        // 2.2.3 No Timing (Level AAA)
        Rule {
            id: "no-timing".to_string(),
            name: "Timing should not be essential".to_string(),
            description: "Timing should not be an essential part of content or activity.".to_string(),
            level: WCAGLevel::AAA,
            principle: Principle::Operable,
            guideline: "2.2".to_string(),
            success_criterion: "no-timing".to_string(),
            tags: vec!["timing".to_string()],
        },

        // 2.2.4 Interruptions (Level AAA)
        Rule {
            id: "interruptions".to_string(),
            name: "Interruptions must be postponable".to_string(),
            description: "Users should be able to postpone or suppress interruptions.".to_string(),
            level: WCAGLevel::AAA,
            principle: Principle::Operable,
            guideline: "2.2".to_string(),
            success_criterion: "interruptions".to_string(),
            tags: vec!["interruptions".to_string()],
        },

        // 2.2.5 Re-authenticating (Level AAA)
        Rule {
            id: "re-authenticating".to_string(),
            name: "Re-authentication must not lose data".to_string(),
            description: "When a session expires, users can re-authenticate without losing data.".to_string(),
            level: WCAGLevel::AAA,
            principle: Principle::Operable,
            guideline: "2.2".to_string(),
            success_criterion: "re-authenticating".to_string(),
            tags: vec!["authentication".to_string()],
        },

        // 2.2.6 Timeouts (Level AAA)
        Rule {
            id: "timeouts".to_string(),
            name: "Users must be warned of timeouts".to_string(),
            description: "Users should be warned of duration of inactivity that causes data loss.".to_string(),
            level: WCAGLevel::AAA,
            principle: Principle::Operable,
            guideline: "2.2".to_string(),
            success_criterion: "timeouts".to_string(),
            tags: vec!["timeouts".to_string()],
        },

        // 2.3.2 Three Flashes (Level AAA)
        Rule {
            id: "three-flashes".to_string(),
            name: "Content must not flash more than 3 times".to_string(),
            description: "Web pages must not contain anything that flashes more than three times per second.".to_string(),
            level: WCAGLevel::AAA,
            principle: Principle::Operable,
            guideline: "2.3".to_string(),
            success_criterion: "three-flashes".to_string(),
            tags: vec!["flashing".to_string(), "seizures".to_string()],
        },

        // 2.3.3 Animation from Interactions (Level AAA)
        Rule {
            id: "animation-from-interactions".to_string(),
            name: "Motion animation must be disableable".to_string(),
            description: "Motion animation triggered by interaction can be disabled.".to_string(),
            level: WCAGLevel::AAA,
            principle: Principle::Operable,
            guideline: "2.3".to_string(),
            success_criterion: "animation-from-interactions".to_string(),
            tags: vec!["animation".to_string(), "motion".to_string()],
        },

        // 2.4.8 Location (Level AAA)
        Rule {
            id: "location".to_string(),
            name: "User location must be indicated".to_string(),
            description: "Information about user's location within a set of pages is available.".to_string(),
            level: WCAGLevel::AAA,
            principle: Principle::Operable,
            guideline: "2.4".to_string(),
            success_criterion: "location".to_string(),
            tags: vec!["navigation".to_string(), "breadcrumbs".to_string()],
        },

        // 2.4.9 Link Purpose (Link Only) (Level AAA)
        Rule {
            id: "link-purpose-link-only".to_string(),
            name: "Link purpose must be clear from link text alone".to_string(),
            description: "The purpose of each link can be identified from the link text alone.".to_string(),
            level: WCAGLevel::AAA,
            principle: Principle::Operable,
            guideline: "2.4".to_string(),
            success_criterion: "link-purpose-link-only".to_string(),
            tags: vec!["links".to_string()],
        },

        // 2.4.10 Section Headings (Level AAA)
        Rule {
            id: "section-headings".to_string(),
            name: "Sections must have headings".to_string(),
            description: "Section headings should be used to organize content.".to_string(),
            level: WCAGLevel::AAA,
            principle: Principle::Operable,
            guideline: "2.4".to_string(),
            success_criterion: "section-headings".to_string(),
            tags: vec!["headings".to_string(), "structure".to_string()],
        },

        // 2.5.5 Target Size (Level AAA)
        Rule {
            id: "target-size".to_string(),
            name: "Touch targets must be at least 44x44 pixels".to_string(),
            description: "The size of the target for pointer inputs is at least 44 by 44 CSS pixels.".to_string(),
            level: WCAGLevel::AAA,
            principle: Principle::Operable,
            guideline: "2.5".to_string(),
            success_criterion: "target-size".to_string(),
            tags: vec!["touch".to_string(), "mobile".to_string()],
        },

        // 2.5.6 Concurrent Input Mechanisms (Level AAA)
        Rule {
            id: "concurrent-input".to_string(),
            name: "Multiple input mechanisms must be supported".to_string(),
            description: "Content does not restrict use of input modalities.".to_string(),
            level: WCAGLevel::AAA,
            principle: Principle::Operable,
            guideline: "2.5".to_string(),
            success_criterion: "concurrent-input-mechanisms".to_string(),
            tags: vec!["input".to_string()],
        },

        // 3.1.3 Unusual Words (Level AAA)
        Rule {
            id: "unusual-words".to_string(),
            name: "Unusual words must be defined".to_string(),
            description: "A mechanism is available for identifying specific definitions of words used in an unusual way.".to_string(),
            level: WCAGLevel::AAA,
            principle: Principle::Understandable,
            guideline: "3.1".to_string(),
            success_criterion: "unusual-words".to_string(),
            tags: vec!["language".to_string(), "definitions".to_string()],
        },

        // 3.1.4 Abbreviations (Level AAA)
        Rule {
            id: "abbreviations".to_string(),
            name: "Abbreviations must be expanded".to_string(),
            description: "A mechanism for identifying expanded form of abbreviations is available.".to_string(),
            level: WCAGLevel::AAA,
            principle: Principle::Understandable,
            guideline: "3.1".to_string(),
            success_criterion: "abbreviations".to_string(),
            tags: vec!["language".to_string(), "abbreviations".to_string()],
        },

        // 3.1.5 Reading Level (Level AAA)
        Rule {
            id: "reading-level".to_string(),
            name: "Content should not require advanced reading ability".to_string(),
            description: "When text requires advanced reading ability, supplemental content should be provided.".to_string(),
            level: WCAGLevel::AAA,
            principle: Principle::Understandable,
            guideline: "3.1".to_string(),
            success_criterion: "reading-level".to_string(),
            tags: vec!["language".to_string(), "readability".to_string()],
        },

        // 3.1.6 Pronunciation (Level AAA)
        Rule {
            id: "pronunciation".to_string(),
            name: "Pronunciation must be available for ambiguous words".to_string(),
            description: "A mechanism for identifying pronunciation of words is available where meaning is ambiguous.".to_string(),
            level: WCAGLevel::AAA,
            principle: Principle::Understandable,
            guideline: "3.1".to_string(),
            success_criterion: "pronunciation".to_string(),
            tags: vec!["language".to_string(), "pronunciation".to_string()],
        },

        // 3.2.5 Change on Request (Level AAA)
        Rule {
            id: "change-on-request".to_string(),
            name: "Context changes only occur on user request".to_string(),
            description: "Changes of context are initiated only by user request.".to_string(),
            level: WCAGLevel::AAA,
            principle: Principle::Understandable,
            guideline: "3.2".to_string(),
            success_criterion: "change-on-request".to_string(),
            tags: vec!["context-changes".to_string()],
        },

        // 3.3.5 Help (Level AAA)
        Rule {
            id: "context-sensitive-help".to_string(),
            name: "Context-sensitive help must be available".to_string(),
            description: "Context-sensitive help is available for form inputs.".to_string(),
            level: WCAGLevel::AAA,
            principle: Principle::Understandable,
            guideline: "3.3".to_string(),
            success_criterion: "help".to_string(),
            tags: vec!["forms".to_string(), "help".to_string()],
        },

        // 3.3.6 Error Prevention (All) (Level AAA)
        Rule {
            id: "error-prevention-all".to_string(),
            name: "All submissions must be reversible or confirmed".to_string(),
            description: "All user submissions must be reversible, verifiable, or confirmed.".to_string(),
            level: WCAGLevel::AAA,
            principle: Principle::Understandable,
            guideline: "3.3".to_string(),
            success_criterion: "error-prevention-all".to_string(),
            tags: vec!["forms".to_string(), "submissions".to_string()],
        },
    ]
}
