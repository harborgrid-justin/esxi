use crate::analysis::{analyze_color_contrast, analyze_heading_structure, check_aria_validity};
use crate::rules;
use crate::types::*;
use rayon::prelude::*;
use scraper::{Html, Selector, ElementRef};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref VALID_LANG_CODES: Regex = Regex::new(r"^[a-z]{2,3}(-[A-Z]{2})?$").unwrap();
    static ref ARIA_ROLES: HashSet<&'static str> = {
        let mut set = HashSet::new();
        set.insert("alert");
        set.insert("alertdialog");
        set.insert("application");
        set.insert("article");
        set.insert("banner");
        set.insert("button");
        set.insert("checkbox");
        set.insert("complementary");
        set.insert("contentinfo");
        set.insert("dialog");
        set.insert("document");
        set.insert("feed");
        set.insert("figure");
        set.insert("form");
        set.insert("grid");
        set.insert("gridcell");
        set.insert("group");
        set.insert("heading");
        set.insert("img");
        set.insert("link");
        set.insert("list");
        set.insert("listbox");
        set.insert("listitem");
        set.insert("main");
        set.insert("menu");
        set.insert("menubar");
        set.insert("menuitem");
        set.insert("navigation");
        set.insert("progressbar");
        set.insert("radio");
        set.insert("radiogroup");
        set.insert("region");
        set.insert("row");
        set.insert("rowgroup");
        set.insert("search");
        set.insert("separator");
        set.insert("slider");
        set.insert("spinbutton");
        set.insert("status");
        set.insert("switch");
        set.insert("tab");
        set.insert("table");
        set.insert("tablist");
        set.insert("tabpanel");
        set.insert("textbox");
        set.insert("timer");
        set.insert("toolbar");
        set.insert("tooltip");
        set.insert("tree");
        set.insert("treegrid");
        set.insert("treeitem");
        set
    };
}

/// HTML Parser and DOM Analyzer
pub struct HTMLParser {
    document: Html,
    url: String,
}

impl HTMLParser {
    pub fn new(html: &str, url: String) -> Result<Self> {
        let document = Html::parse_document(html);
        Ok(Self { document, url })
    }

    pub fn get_document(&self) -> &Html {
        &self.document
    }

    pub fn select(&self, selector_str: &str) -> Result<Vec<ElementRef>> {
        let selector = Selector::parse(selector_str)
            .map_err(|e| ScannerError::ParseError(format!("Invalid selector: {:?}", e)))?;
        Ok(self.document.select(&selector).collect())
    }

    pub fn get_title(&self) -> String {
        if let Ok(elements) = self.select("title") {
            if let Some(title) = elements.first() {
                return title.text().collect::<String>().trim().to_string();
            }
        }
        String::new()
    }

    pub fn count_elements(&self) -> usize {
        if let Ok(elements) = self.select("*") {
            elements.len()
        } else {
            0
        }
    }
}

/// WCAG Rule Engine - executes rules against parsed HTML
pub struct WCAGRuleEngine {
    rules: Vec<Rule>,
    parser: HTMLParser,
}

impl WCAGRuleEngine {
    pub fn new(parser: HTMLParser, levels: Vec<WCAGLevel>) -> Self {
        let rules = rules::get_rules_by_levels(&levels);
        Self { rules, parser }
    }

    /// Execute all rules and collect issues
    pub fn execute(&self) -> Vec<Issue> {
        let issues = Arc::new(Mutex::new(Vec::new()));

        // Execute rules in parallel using rayon
        self.rules.par_iter().for_each(|rule| {
            let rule_issues = self.execute_rule(rule);
            let mut issues_lock = issues.lock().unwrap();
            issues_lock.extend(rule_issues);
        });

        let issues = Arc::try_unwrap(issues).unwrap().into_inner().unwrap();
        issues
    }

    /// Execute a single rule
    fn execute_rule(&self, rule: &Rule) -> Vec<Issue> {
        match rule.id.as_str() {
            "image-alt" => self.check_image_alt(),
            "input-image-alt" => self.check_input_image_alt(),
            "label" => self.check_form_labels(),
            "document-title" => self.check_document_title(),
            "html-has-lang" => self.check_html_lang(),
            "html-lang-valid" => self.check_html_lang_valid(),
            "link-name" => self.check_link_text(),
            "button-name" => self.check_button_text(),
            "duplicate-id" => self.check_duplicate_ids(),
            "heading-order" => self.check_heading_order(),
            "empty-heading" => self.check_empty_headings(),
            "color-contrast" => self.check_color_contrast(),
            "area-alt" => self.check_area_alt(),
            "object-alt" => self.check_object_alt(),
            "list" => self.check_list_structure(),
            "definition-list" => self.check_definition_list(),
            "table-headers" => self.check_table_headers(),
            "tabindex" => self.check_tabindex(),
            "meta-refresh" => self.check_meta_refresh(),
            "blink" => self.check_blink(),
            "marquee" => self.check_marquee(),
            "aria-required-attr" => self.check_aria_required_attrs(),
            "aria-valid-attr" => self.check_aria_valid_attrs(),
            "aria-valid-attr-value" => self.check_aria_valid_attr_values(),
            _ => Vec::new(),
        }
    }

    fn create_issue(
        &self,
        rule: &Rule,
        severity: Severity,
        message: String,
        element: ElementRef,
        fix_suggestions: Vec<String>,
    ) -> Issue {
        let context = self.create_context(element);
        Issue::new(
            rule,
            severity,
            message,
            format!("See WCAG 2.1 {} - {}", rule.level.as_str(), rule.success_criterion),
            context,
            fix_suggestions,
            format!("This violates WCAG {} principle: {}", rule.level.as_str(), rule.principle.as_str()),
        )
    }

    fn create_context(&self, element: ElementRef) -> IssueContext {
        let html = element.html();
        let mut attributes = HashMap::new();

        for (name, value) in element.value().attrs() {
            attributes.insert(name.to_string(), value.to_string());
        }

        IssueContext {
            html: element.text().collect::<String>(),
            outer_html: html.clone(),
            position: Position {
                line: 0,
                column: 0,
                xpath: Self::get_xpath(element),
                selector: Self::get_selector(element),
            },
            attributes,
            computed_styles: None,
        }
    }

    fn get_xpath(element: ElementRef) -> String {
        let tag = element.value().name();
        format!("//{}", tag)
    }

    fn get_selector(element: ElementRef) -> String {
        let tag = element.value().name();
        if let Some(id) = element.value().attr("id") {
            format!("{}#{}", tag, id)
        } else if let Some(class) = element.value().attr("class") {
            let classes = class.split_whitespace().next().unwrap_or("");
            format!("{}.{}", tag, classes)
        } else {
            tag.to_string()
        }
    }

    // Rule implementations
    fn check_image_alt(&self) -> Vec<Issue> {
        let mut issues = Vec::new();
        if let Ok(images) = self.parser.select("img") {
            let rule = rules::get_rule_by_id("image-alt").unwrap();

            for img in images {
                let has_alt = img.value().attr("alt").is_some();
                let role = img.value().attr("role");
                let aria_hidden = img.value().attr("aria-hidden");

                if !has_alt && role != Some("presentation") && aria_hidden != Some("true") {
                    issues.push(self.create_issue(
                        rule,
                        Severity::Critical,
                        "Image is missing alt attribute".to_string(),
                        img,
                        vec!["Add an alt attribute describing the image".to_string(),
                             "If decorative, use alt=\"\" or role=\"presentation\"".to_string()],
                    ));
                } else if has_alt {
                    let alt_value = img.value().attr("alt").unwrap();
                    if alt_value.is_empty() {
                        let src = img.value().attr("src").unwrap_or("");
                        if !src.is_empty() && role != Some("presentation") {
                            // Empty alt is only acceptable for decorative images
                        }
                    }
                }
            }
        }
        issues
    }

    fn check_input_image_alt(&self) -> Vec<Issue> {
        let mut issues = Vec::new();
        if let Ok(inputs) = self.parser.select("input[type='image']") {
            let rule = rules::get_rule_by_id("input-image-alt").unwrap();

            for input in inputs {
                if input.value().attr("alt").is_none() {
                    issues.push(self.create_issue(
                        rule,
                        Severity::Critical,
                        "Image button is missing alt attribute".to_string(),
                        input,
                        vec!["Add an alt attribute describing the button's purpose".to_string()],
                    ));
                }
            }
        }
        issues
    }

    fn check_form_labels(&self) -> Vec<Issue> {
        let mut issues = Vec::new();
        if let Ok(inputs) = self.parser.select("input, select, textarea") {
            let rule = rules::get_rule_by_id("label").unwrap();

            for input in inputs {
                let input_type = input.value().attr("type").unwrap_or("text");

                // Skip hidden and submit/button inputs
                if input_type == "hidden" || input_type == "submit" || input_type == "button" {
                    continue;
                }

                let has_label = input.value().attr("id").and_then(|id| {
                    let selector = format!("label[for='{}']", id);
                    self.parser.select(&selector).ok().and_then(|labels| {
                        if labels.is_empty() { None } else { Some(true) }
                    })
                }).is_some();

                let has_aria_label = input.value().attr("aria-label").is_some() ||
                                    input.value().attr("aria-labelledby").is_some();

                if !has_label && !has_aria_label {
                    issues.push(self.create_issue(
                        rule,
                        Severity::Critical,
                        "Form input is missing a label".to_string(),
                        input,
                        vec!["Add a label element with for attribute matching input id".to_string(),
                             "Or add aria-label or aria-labelledby attribute".to_string()],
                    ));
                }
            }
        }
        issues
    }

    fn check_document_title(&self) -> Vec<Issue> {
        let mut issues = Vec::new();
        let rule = rules::get_rule_by_id("document-title").unwrap();

        if let Ok(titles) = self.parser.select("title") {
            if titles.is_empty() {
                if let Ok(html_elements) = self.parser.select("html") {
                    if let Some(html) = html_elements.first() {
                        issues.push(self.create_issue(
                            rule,
                            Severity::Serious,
                            "Page is missing a title element".to_string(),
                            *html,
                            vec!["Add a <title> element in the <head> section".to_string()],
                        ));
                    }
                }
            } else {
                let title_text: String = titles[0].text().collect::<String>().trim().to_string();
                if title_text.is_empty() {
                    issues.push(self.create_issue(
                        rule,
                        Severity::Serious,
                        "Page title is empty".to_string(),
                        titles[0],
                        vec!["Add descriptive text to the title element".to_string()],
                    ));
                }
            }
        }
        issues
    }

    fn check_html_lang(&self) -> Vec<Issue> {
        let mut issues = Vec::new();
        if let Ok(html_elements) = self.parser.select("html") {
            let rule = rules::get_rule_by_id("html-has-lang").unwrap();

            if let Some(html) = html_elements.first() {
                if html.value().attr("lang").is_none() {
                    issues.push(self.create_issue(
                        rule,
                        Severity::Serious,
                        "HTML element is missing lang attribute".to_string(),
                        *html,
                        vec!["Add lang attribute to html element (e.g., lang=\"en\")".to_string()],
                    ));
                }
            }
        }
        issues
    }

    fn check_html_lang_valid(&self) -> Vec<Issue> {
        let mut issues = Vec::new();
        if let Ok(html_elements) = self.parser.select("html") {
            let rule = rules::get_rule_by_id("html-lang-valid").unwrap();

            if let Some(html) = html_elements.first() {
                if let Some(lang) = html.value().attr("lang") {
                    if !VALID_LANG_CODES.is_match(lang) {
                        issues.push(self.create_issue(
                            rule,
                            Severity::Serious,
                            format!("HTML lang attribute has invalid value: {}", lang),
                            *html,
                            vec!["Use a valid ISO language code (e.g., \"en\", \"es\", \"fr\")".to_string()],
                        ));
                    }
                }
            }
        }
        issues
    }

    fn check_link_text(&self) -> Vec<Issue> {
        let mut issues = Vec::new();
        if let Ok(links) = self.parser.select("a[href]") {
            let rule = rules::get_rule_by_id("link-name").unwrap();

            for link in links {
                let text: String = link.text().collect::<String>().trim().to_string();
                let aria_label = link.value().attr("aria-label");
                let aria_labelledby = link.value().attr("aria-labelledby");

                if text.is_empty() && aria_label.is_none() && aria_labelledby.is_none() {
                    issues.push(self.create_issue(
                        rule,
                        Severity::Serious,
                        "Link has no accessible text".to_string(),
                        link,
                        vec!["Add text content to the link".to_string(),
                             "Or add aria-label attribute".to_string()],
                    ));
                }
            }
        }
        issues
    }

    fn check_button_text(&self) -> Vec<Issue> {
        let mut issues = Vec::new();
        if let Ok(buttons) = self.parser.select("button") {
            let rule = rules::get_rule_by_id("button-name").unwrap();

            for button in buttons {
                let text: String = button.text().collect::<String>().trim().to_string();
                let aria_label = button.value().attr("aria-label");
                let aria_labelledby = button.value().attr("aria-labelledby");

                if text.is_empty() && aria_label.is_none() && aria_labelledby.is_none() {
                    issues.push(self.create_issue(
                        rule,
                        Severity::Critical,
                        "Button has no accessible text".to_string(),
                        button,
                        vec!["Add text content to the button".to_string(),
                             "Or add aria-label attribute".to_string()],
                    ));
                }
            }
        }
        issues
    }

    fn check_duplicate_ids(&self) -> Vec<Issue> {
        let mut issues = Vec::new();
        let mut id_map: HashMap<String, Vec<ElementRef>> = HashMap::new();

        if let Ok(elements) = self.parser.select("[id]") {
            for element in elements {
                if let Some(id) = element.value().attr("id") {
                    id_map.entry(id.to_string()).or_insert_with(Vec::new).push(element);
                }
            }
        }

        let rule = rules::get_rule_by_id("duplicate-id").unwrap();

        for (id, elements) in id_map {
            if elements.len() > 1 {
                for element in elements {
                    issues.push(self.create_issue(
                        rule,
                        Severity::Critical,
                        format!("Duplicate ID: {}", id),
                        element,
                        vec!["Ensure each ID is unique within the page".to_string()],
                    ));
                }
            }
        }
        issues
    }

    fn check_heading_order(&self) -> Vec<Issue> {
        analyze_heading_structure(self.parser.get_document())
    }

    fn check_empty_headings(&self) -> Vec<Issue> {
        let mut issues = Vec::new();
        if let Ok(headings) = self.parser.select("h1, h2, h3, h4, h5, h6") {
            let rule = rules::get_rule_by_id("empty-heading").unwrap();

            for heading in headings {
                let text: String = heading.text().collect::<String>().trim().to_string();
                if text.is_empty() {
                    issues.push(self.create_issue(
                        rule,
                        Severity::Serious,
                        "Heading is empty".to_string(),
                        heading,
                        vec!["Add descriptive text to the heading".to_string()],
                    ));
                }
            }
        }
        issues
    }

    fn check_color_contrast(&self) -> Vec<Issue> {
        analyze_color_contrast(self.parser.get_document())
    }

    fn check_area_alt(&self) -> Vec<Issue> {
        let mut issues = Vec::new();
        if let Ok(areas) = self.parser.select("area[href]") {
            let rule = rules::get_rule_by_id("area-alt").unwrap();

            for area in areas {
                if area.value().attr("alt").is_none() {
                    issues.push(self.create_issue(
                        rule,
                        Severity::Critical,
                        "Image map area is missing alt attribute".to_string(),
                        area,
                        vec!["Add an alt attribute describing the linked area".to_string()],
                    ));
                }
            }
        }
        issues
    }

    fn check_object_alt(&self) -> Vec<Issue> {
        let mut issues = Vec::new();
        if let Ok(objects) = self.parser.select("object") {
            let rule = rules::get_rule_by_id("object-alt").unwrap();

            for object in objects {
                let text: String = object.text().collect::<String>().trim().to_string();
                let aria_label = object.value().attr("aria-label");

                if text.is_empty() && aria_label.is_none() {
                    issues.push(self.create_issue(
                        rule,
                        Severity::Serious,
                        "Object element has no text alternative".to_string(),
                        object,
                        vec!["Add text content inside the object element".to_string(),
                             "Or add aria-label attribute".to_string()],
                    ));
                }
            }
        }
        issues
    }

    fn check_list_structure(&self) -> Vec<Issue> {
        let mut issues = Vec::new();
        if let Ok(lists) = self.parser.select("ul, ol") {
            let rule = rules::get_rule_by_id("list").unwrap();

            for list in lists {
                // Check that list only contains li elements (ignoring whitespace text nodes)
                let children = list.children();
                for child in children {
                    if let Some(element) = child.value().as_element() {
                        if element.name() != "li" {
                            issues.push(self.create_issue(
                                rule,
                                Severity::Moderate,
                                format!("List contains invalid child element: {}", element.name()),
                                list,
                                vec!["Lists should only contain <li> elements as direct children".to_string()],
                            ));
                        }
                    }
                }
            }
        }
        issues
    }

    fn check_definition_list(&self) -> Vec<Issue> {
        let mut issues = Vec::new();
        if let Ok(dlists) = self.parser.select("dl") {
            let rule = rules::get_rule_by_id("definition-list").unwrap();

            for dl in dlists {
                let children = dl.children();
                for child in children {
                    if let Some(element) = child.value().as_element() {
                        let name = element.name();
                        if name != "dt" && name != "dd" {
                            issues.push(self.create_issue(
                                rule,
                                Severity::Moderate,
                                format!("Definition list contains invalid child element: {}", name),
                                dl,
                                vec!["Definition lists should only contain <dt> and <dd> elements".to_string()],
                            ));
                        }
                    }
                }
            }
        }
        issues
    }

    fn check_table_headers(&self) -> Vec<Issue> {
        let mut issues = Vec::new();
        if let Ok(tables) = self.parser.select("table") {
            let rule = rules::get_rule_by_id("table-headers").unwrap();

            for table in tables {
                // Check if table has th elements
                let table_html = table.html();
                if !table_html.contains("<th") && !table_html.contains("role=\"presentation\"") {
                    issues.push(self.create_issue(
                        rule,
                        Severity::Serious,
                        "Data table is missing header cells".to_string(),
                        table,
                        vec!["Add <th> elements to define table headers".to_string(),
                             "Or use role=\"presentation\" if table is used for layout".to_string()],
                    ));
                }
            }
        }
        issues
    }

    fn check_tabindex(&self) -> Vec<Issue> {
        let mut issues = Vec::new();
        if let Ok(elements) = self.parser.select("[tabindex]") {
            let rule = rules::get_rule_by_id("tabindex").unwrap();

            for element in elements {
                if let Some(tabindex) = element.value().attr("tabindex") {
                    if let Ok(value) = tabindex.parse::<i32>() {
                        if value > 0 {
                            issues.push(self.create_issue(
                                rule,
                                Severity::Serious,
                                format!("Positive tabindex value found: {}", value),
                                element,
                                vec!["Use tabindex=\"0\" for focusable elements".to_string(),
                                     "Use tabindex=\"-1\" for programmatically focusable elements".to_string(),
                                     "Avoid positive tabindex values as they disrupt natural tab order".to_string()],
                            ));
                        }
                    }
                }
            }
        }
        issues
    }

    fn check_meta_refresh(&self) -> Vec<Issue> {
        let mut issues = Vec::new();
        if let Ok(meta_elements) = self.parser.select("meta[http-equiv='refresh']") {
            let rule = rules::get_rule_by_id("meta-refresh").unwrap();

            for meta in meta_elements {
                issues.push(self.create_issue(
                    rule,
                    Severity::Serious,
                    "Page uses meta refresh".to_string(),
                    meta,
                    vec!["Remove meta refresh and use server-side redirects instead".to_string(),
                         "Or use JavaScript with user control".to_string()],
                ));
            }
        }
        issues
    }

    fn check_blink(&self) -> Vec<Issue> {
        let mut issues = Vec::new();
        if let Ok(blinks) = self.parser.select("blink") {
            let rule = rules::get_rule_by_id("blink").unwrap();

            for blink in blinks {
                issues.push(self.create_issue(
                    rule,
                    Severity::Serious,
                    "Page uses deprecated blink element".to_string(),
                    blink,
                    vec!["Remove blink element".to_string()],
                ));
            }
        }
        issues
    }

    fn check_marquee(&self) -> Vec<Issue> {
        let mut issues = Vec::new();
        if let Ok(marquees) = self.parser.select("marquee") {
            let rule = rules::get_rule_by_id("marquee").unwrap();

            for marquee in marquees {
                issues.push(self.create_issue(
                    rule,
                    Severity::Serious,
                    "Page uses deprecated marquee element".to_string(),
                    marquee,
                    vec!["Remove marquee element and use CSS animations with user controls".to_string()],
                ));
            }
        }
        issues
    }

    fn check_aria_required_attrs(&self) -> Vec<Issue> {
        check_aria_validity(self.parser.get_document(), "required-attr")
    }

    fn check_aria_valid_attrs(&self) -> Vec<Issue> {
        check_aria_validity(self.parser.get_document(), "valid-attr")
    }

    fn check_aria_valid_attr_values(&self) -> Vec<Issue> {
        check_aria_validity(self.parser.get_document(), "valid-attr-value")
    }
}

/// Accessibility Scanner - main entry point
pub struct AccessibilityScanner {
    config: ScanConfig,
    http_client: reqwest::Client,
}

impl AccessibilityScanner {
    pub fn new(config: ScanConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .unwrap();

        Self { config, http_client }
    }

    /// Scan a single URL
    pub async fn scan_url(&self, url: &str) -> Result<PageResult> {
        let start = Instant::now();

        let response = self.http_client.get(url).send().await?;
        let status = response.status().as_u16();
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("text/html")
            .to_string();

        let html = response.text().await?;

        let parser = HTMLParser::new(&html, url.to_string())?;
        let title = parser.get_title();
        let elements_count = parser.count_elements();

        let engine = WCAGRuleEngine::new(parser, self.config.levels.clone());
        let issues = engine.execute();

        let scan_time_ms = start.elapsed().as_millis() as u64;

        Ok(PageResult {
            url: url.to_string(),
            title,
            issues,
            elements_count,
            scan_time_ms,
            http_status: status,
            content_type,
        })
    }

    /// Scan a website (multiple pages)
    pub async fn scan(&self) -> Result<ScanResult> {
        let mut result = ScanResult::new(self.config.clone());

        // For now, just scan the target URL
        // Full crawling would be implemented here
        let page = self.scan_url(&self.config.target_url).await?;
        result.add_page(page);

        result.finalize();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_parser() {
        let html = r#"
            <!DOCTYPE html>
            <html lang="en">
            <head><title>Test Page</title></head>
            <body><h1>Hello World</h1></body>
            </html>
        "#;

        let parser = HTMLParser::new(html, "http://example.com".to_string()).unwrap();
        assert_eq!(parser.get_title(), "Test Page");
        assert!(parser.count_elements() > 0);
    }

    #[test]
    fn test_image_alt_check() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <body>
                <img src="test.jpg">
            </body>
            </html>
        "#;

        let parser = HTMLParser::new(html, "http://example.com".to_string()).unwrap();
        let engine = WCAGRuleEngine::new(parser, vec![WCAGLevel::A]);
        let issues = engine.check_image_alt();

        assert!(!issues.is_empty());
        assert_eq!(issues[0].rule_id, "image-alt");
    }
}
