use crate::rules;
use crate::types::*;
use scraper::{Html, Selector, ElementRef};
use std::collections::HashMap;

/// Analyze color contrast ratios between text and background
pub fn analyze_color_contrast(document: &Html) -> Vec<Issue> {
    let mut issues = Vec::new();
    let rule = rules::get_rule_by_id("color-contrast");

    if rule.is_none() {
        return issues;
    }

    let rule = rule.unwrap();

    // Select all text elements
    let selector = Selector::parse("p, h1, h2, h3, h4, h5, h6, span, div, a, button, label, li, td, th").unwrap();
    let elements: Vec<ElementRef> = document.select(&selector).collect();

    for element in elements {
        let text: String = element.text().collect::<String>().trim().to_string();

        if text.is_empty() {
            continue;
        }

        // In a real implementation, we would:
        // 1. Extract computed color and background-color from inline styles or style tags
        // 2. Calculate contrast ratio using the formula: (L1 + 0.05) / (L2 + 0.05)
        // 3. Where L is relative luminance
        // For this implementation, we'll check for inline styles

        if let Some(style) = element.value().attr("style") {
            let has_color = style.contains("color:");
            let has_background = style.contains("background-color:") || style.contains("background:");

            if has_color && has_background {
                // Here we would parse the colors and calculate contrast
                // For now, we'll create a placeholder check
                let colors = extract_colors_from_style(style);

                if let (Some(fg), Some(bg)) = colors {
                    let contrast_ratio = calculate_contrast_ratio(&fg, &bg);

                    // WCAG AA requires 4.5:1 for normal text, 3:1 for large text
                    let font_size = extract_font_size(style);
                    let is_large_text = font_size.map_or(false, |size| size >= 18.0 || (size >= 14.0 && style.contains("bold")));

                    let required_ratio = if is_large_text { 3.0 } else { 4.5 };

                    if contrast_ratio < required_ratio {
                        let context = create_element_context(element);
                        let issue = Issue::new(
                            rule,
                            Severity::Serious,
                            format!(
                                "Insufficient color contrast: {:.2}:1 (minimum {:.1}:1 required)",
                                contrast_ratio, required_ratio
                            ),
                            format!("Ensure text has a contrast ratio of at least {:.1}:1 against its background", required_ratio),
                            context,
                            vec![
                                format!("Increase contrast to at least {:.1}:1", required_ratio),
                                "Use a color contrast checker tool to verify colors".to_string(),
                                "Consider using darker text on light backgrounds or lighter text on dark backgrounds".to_string(),
                            ],
                            "Low contrast makes text difficult to read for users with low vision or color blindness".to_string(),
                        );
                        issues.push(issue);
                    }
                }
            }
        }
    }

    issues
}

/// Analyze heading structure and hierarchy
pub fn analyze_heading_structure(document: &Html) -> Vec<Issue> {
    let mut issues = Vec::new();
    let rule = rules::get_rule_by_id("heading-order");

    if rule.is_none() {
        return issues;
    }

    let rule = rule.unwrap();

    let selector = Selector::parse("h1, h2, h3, h4, h5, h6").unwrap();
    let headings: Vec<ElementRef> = document.select(&selector).collect();

    let mut last_level: Option<u8> = None;

    for heading in headings {
        let tag_name = heading.value().name();
        let current_level = tag_name.chars().last().unwrap().to_digit(10).unwrap() as u8;

        if let Some(last) = last_level {
            // Check if heading levels are skipped
            if current_level > last + 1 {
                let context = create_element_context(heading);
                let issue = Issue::new(
                    rule,
                    Severity::Moderate,
                    format!(
                        "Heading levels skipped from h{} to h{}",
                        last, current_level
                    ),
                    "Heading levels should not be skipped to maintain proper document structure".to_string(),
                    context,
                    vec![
                        format!("Use h{} instead of h{} here", last + 1, current_level),
                        "Maintain sequential heading hierarchy".to_string(),
                    ],
                    "Skipped heading levels can confuse screen reader users navigating by headings".to_string(),
                );
                issues.push(issue);
            }
        }

        last_level = Some(current_level);
    }

    issues
}

/// Check ARIA validity
pub fn check_aria_validity(document: &Html, check_type: &str) -> Vec<Issue> {
    let mut issues = Vec::new();

    match check_type {
        "required-attr" => {
            let rule = rules::get_rule_by_id("aria-required-attr");
            if let Some(rule) = rule {
                let selector = Selector::parse("[role]").unwrap();
                let elements: Vec<ElementRef> = document.select(&selector).collect();

                for element in elements {
                    if let Some(role) = element.value().attr("role") {
                        let required_attrs = get_required_aria_attrs(role);

                        for required_attr in required_attrs {
                            if element.value().attr(required_attr).is_none() {
                                let context = create_element_context(element);
                                let issue = Issue::new(
                                    rule,
                                    Severity::Serious,
                                    format!("Element with role='{}' is missing required attribute: {}", role, required_attr),
                                    format!("Add the {} attribute to this element", required_attr),
                                    context,
                                    vec![format!("Add {}=\"...\" to the element", required_attr)],
                                    format!("The {} role requires the {} attribute for assistive technologies", role, required_attr),
                                );
                                issues.push(issue);
                            }
                        }
                    }
                }
            }
        }
        "valid-attr" => {
            let rule = rules::get_rule_by_id("aria-valid-attr");
            if let Some(rule) = rule {
                let selector = Selector::parse("[aria-*]").unwrap();
                let all_elements: Vec<ElementRef> = document.select(&Selector::parse("*").unwrap()).collect();

                let valid_aria_attrs = get_valid_aria_attributes();

                for element in all_elements {
                    for (attr_name, _) in element.value().attrs() {
                        if attr_name.starts_with("aria-") && !valid_aria_attrs.contains(&attr_name) {
                            let context = create_element_context(element);
                            let issue = Issue::new(
                                rule,
                                Severity::Serious,
                                format!("Invalid ARIA attribute: {}", attr_name),
                                "Use only valid ARIA attributes".to_string(),
                                context,
                                vec![
                                    format!("Remove the invalid {} attribute", attr_name),
                                    "Check ARIA specification for valid attributes".to_string(),
                                ],
                                "Invalid ARIA attributes may be ignored by assistive technologies".to_string(),
                            );
                            issues.push(issue);
                        }
                    }
                }
            }
        }
        "valid-attr-value" => {
            let rule = rules::get_rule_by_id("aria-valid-attr-value");
            if let Some(rule) = rule {
                let all_elements: Vec<ElementRef> = document.select(&Selector::parse("*").unwrap()).collect();

                for element in all_elements {
                    // Check aria-hidden
                    if let Some(aria_hidden) = element.value().attr("aria-hidden") {
                        if aria_hidden != "true" && aria_hidden != "false" {
                            let context = create_element_context(element);
                            let issue = Issue::new(
                                rule,
                                Severity::Serious,
                                format!("Invalid value for aria-hidden: '{}'", aria_hidden),
                                "aria-hidden must be either 'true' or 'false'".to_string(),
                                context,
                                vec!["Use aria-hidden=\"true\" or aria-hidden=\"false\"".to_string()],
                                "Invalid ARIA attribute values may not work correctly with assistive technologies".to_string(),
                            );
                            issues.push(issue);
                        }
                    }

                    // Check aria-checked
                    if let Some(aria_checked) = element.value().attr("aria-checked") {
                        if aria_checked != "true" && aria_checked != "false" && aria_checked != "mixed" {
                            let context = create_element_context(element);
                            let issue = Issue::new(
                                rule,
                                Severity::Serious,
                                format!("Invalid value for aria-checked: '{}'", aria_checked),
                                "aria-checked must be 'true', 'false', or 'mixed'".to_string(),
                                context,
                                vec!["Use aria-checked=\"true\", \"false\", or \"mixed\"".to_string()],
                                "Invalid ARIA attribute values may not work correctly with assistive technologies".to_string(),
                            );
                            issues.push(issue);
                        }
                    }

                    // Check aria-expanded
                    if let Some(aria_expanded) = element.value().attr("aria-expanded") {
                        if aria_expanded != "true" && aria_expanded != "false" {
                            let context = create_element_context(element);
                            let issue = Issue::new(
                                rule,
                                Severity::Serious,
                                format!("Invalid value for aria-expanded: '{}'", aria_expanded),
                                "aria-expanded must be either 'true' or 'false'".to_string(),
                                context,
                                vec!["Use aria-expanded=\"true\" or aria-expanded=\"false\"".to_string()],
                                "Invalid ARIA attribute values may not work correctly with assistive technologies".to_string(),
                            );
                            issues.push(issue);
                        }
                    }
                }
            }
        }
        _ => {}
    }

    issues
}

/// Extract colors from inline style
fn extract_colors_from_style(style: &str) -> (Option<Color>, Option<Color>) {
    let mut fg_color = None;
    let mut bg_color = None;

    for declaration in style.split(';') {
        let parts: Vec<&str> = declaration.split(':').collect();
        if parts.len() != 2 {
            continue;
        }

        let property = parts[0].trim();
        let value = parts[1].trim();

        match property {
            "color" => fg_color = parse_color(value),
            "background-color" | "background" => bg_color = parse_color(value),
            _ => {}
        }
    }

    (fg_color, bg_color)
}

/// Simple color structure
#[derive(Debug, Clone)]
struct Color {
    r: f64,
    g: f64,
    b: f64,
}

/// Parse color from CSS value
fn parse_color(value: &str) -> Option<Color> {
    // Handle hex colors
    if value.starts_with('#') {
        let hex = value.trim_start_matches('#');
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f64 / 255.0;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f64 / 255.0;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f64 / 255.0;
            return Some(Color { r, g, b });
        }
    }

    // Handle rgb() colors
    if value.starts_with("rgb(") {
        let rgb = value.trim_start_matches("rgb(").trim_end_matches(')');
        let parts: Vec<&str> = rgb.split(',').collect();
        if parts.len() == 3 {
            let r = parts[0].trim().parse::<u8>().ok()? as f64 / 255.0;
            let g = parts[1].trim().parse::<u8>().ok()? as f64 / 255.0;
            let b = parts[2].trim().parse::<u8>().ok()? as f64 / 255.0;
            return Some(Color { r, g, b });
        }
    }

    // Handle named colors (simplified)
    match value {
        "black" => Some(Color { r: 0.0, g: 0.0, b: 0.0 }),
        "white" => Some(Color { r: 1.0, g: 1.0, b: 1.0 }),
        "red" => Some(Color { r: 1.0, g: 0.0, b: 0.0 }),
        "green" => Some(Color { r: 0.0, g: 0.5, b: 0.0 }),
        "blue" => Some(Color { r: 0.0, g: 0.0, b: 1.0 }),
        _ => None,
    }
}

/// Calculate relative luminance
fn relative_luminance(color: &Color) -> f64 {
    let r = if color.r <= 0.03928 {
        color.r / 12.92
    } else {
        ((color.r + 0.055) / 1.055).powf(2.4)
    };

    let g = if color.g <= 0.03928 {
        color.g / 12.92
    } else {
        ((color.g + 0.055) / 1.055).powf(2.4)
    };

    let b = if color.b <= 0.03928 {
        color.b / 12.92
    } else {
        ((color.b + 0.055) / 1.055).powf(2.4)
    };

    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Calculate contrast ratio between two colors
fn calculate_contrast_ratio(color1: &Color, color2: &Color) -> f64 {
    let l1 = relative_luminance(color1);
    let l2 = relative_luminance(color2);

    let lighter = l1.max(l2);
    let darker = l1.min(l2);

    (lighter + 0.05) / (darker + 0.05)
}

/// Extract font size from style
fn extract_font_size(style: &str) -> Option<f64> {
    for declaration in style.split(';') {
        let parts: Vec<&str> = declaration.split(':').collect();
        if parts.len() == 2 {
            let property = parts[0].trim();
            let value = parts[1].trim();

            if property == "font-size" {
                // Parse pt or px values
                if value.ends_with("pt") {
                    return value.trim_end_matches("pt").parse().ok();
                } else if value.ends_with("px") {
                    let px: f64 = value.trim_end_matches("px").parse().ok()?;
                    return Some(px * 0.75); // Convert px to pt
                }
            }
        }
    }
    None
}

/// Get required ARIA attributes for a role
fn get_required_aria_attrs(role: &str) -> Vec<&'static str> {
    match role {
        "checkbox" => vec!["aria-checked"],
        "combobox" => vec!["aria-expanded"],
        "radio" => vec!["aria-checked"],
        "scrollbar" => vec!["aria-controls", "aria-valuenow", "aria-valuemin", "aria-valuemax"],
        "slider" => vec!["aria-valuenow", "aria-valuemin", "aria-valuemax"],
        "spinbutton" => vec!["aria-valuenow", "aria-valuemin", "aria-valuemax"],
        "switch" => vec!["aria-checked"],
        _ => vec![],
    }
}

/// Get valid ARIA attributes
fn get_valid_aria_attributes() -> Vec<&'static str> {
    vec![
        "aria-activedescendant",
        "aria-atomic",
        "aria-autocomplete",
        "aria-busy",
        "aria-checked",
        "aria-controls",
        "aria-current",
        "aria-describedby",
        "aria-details",
        "aria-disabled",
        "aria-dropeffect",
        "aria-errormessage",
        "aria-expanded",
        "aria-flowto",
        "aria-grabbed",
        "aria-haspopup",
        "aria-hidden",
        "aria-invalid",
        "aria-keyshortcuts",
        "aria-label",
        "aria-labelledby",
        "aria-level",
        "aria-live",
        "aria-modal",
        "aria-multiline",
        "aria-multiselectable",
        "aria-orientation",
        "aria-owns",
        "aria-placeholder",
        "aria-posinset",
        "aria-pressed",
        "aria-readonly",
        "aria-relevant",
        "aria-required",
        "aria-roledescription",
        "aria-rowcount",
        "aria-rowindex",
        "aria-rowspan",
        "aria-selected",
        "aria-setsize",
        "aria-sort",
        "aria-valuemax",
        "aria-valuemin",
        "aria-valuenow",
        "aria-valuetext",
    ]
}

/// Create issue context from element
fn create_element_context(element: ElementRef) -> IssueContext {
    let html = element.html();
    let mut attributes = HashMap::new();

    for (name, value) in element.value().attrs() {
        attributes.insert(name.to_string(), value.to_string());
    }

    let tag = element.value().name();
    let selector = if let Some(id) = element.value().attr("id") {
        format!("{}#{}", tag, id)
    } else if let Some(class) = element.value().attr("class") {
        let classes = class.split_whitespace().next().unwrap_or("");
        format!("{}.{}", tag, classes)
    } else {
        tag.to_string()
    };

    IssueContext {
        html: element.text().collect::<String>(),
        outer_html: html.clone(),
        position: Position {
            line: 0,
            column: 0,
            xpath: format!("//{}", tag),
            selector,
        },
        attributes,
        computed_styles: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_parsing() {
        let black = parse_color("#000000").unwrap();
        assert_eq!(black.r, 0.0);
        assert_eq!(black.g, 0.0);
        assert_eq!(black.b, 0.0);

        let white = parse_color("#ffffff").unwrap();
        assert_eq!(white.r, 1.0);
        assert_eq!(white.g, 1.0);
        assert_eq!(white.b, 1.0);
    }

    #[test]
    fn test_contrast_ratio() {
        let black = Color { r: 0.0, g: 0.0, b: 0.0 };
        let white = Color { r: 1.0, g: 1.0, b: 1.0 };

        let ratio = calculate_contrast_ratio(&black, &white);
        assert!(ratio > 20.0); // Black on white should be ~21:1
    }

    #[test]
    fn test_relative_luminance() {
        let white = Color { r: 1.0, g: 1.0, b: 1.0 };
        let lum = relative_luminance(&white);
        assert!(lum > 0.99); // White luminance should be close to 1.0
    }
}
