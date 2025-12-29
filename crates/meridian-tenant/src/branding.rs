//! White-labeling and branding per tenant.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::{TenantError, TenantResult};

/// Color scheme for tenant branding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    pub primary: String,
    pub secondary: String,
    pub accent: String,
    pub background: String,
    pub text: String,
    pub error: String,
    pub success: String,
    pub warning: String,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            primary: "#0066cc".to_string(),
            secondary: "#666666".to_string(),
            accent: "#ff9900".to_string(),
            background: "#ffffff".to_string(),
            text: "#333333".to_string(),
            error: "#cc0000".to_string(),
            success: "#00cc00".to_string(),
            warning: "#ffcc00".to_string(),
        }
    }
}

/// Typography settings for tenant branding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Typography {
    pub font_family: String,
    pub heading_font_family: Option<String>,
    pub font_size_base: String,
    pub font_size_small: String,
    pub font_size_large: String,
}

impl Default for Typography {
    fn default() -> Self {
        Self {
            font_family: "system-ui, -apple-system, sans-serif".to_string(),
            heading_font_family: None,
            font_size_base: "16px".to_string(),
            font_size_small: "14px".to_string(),
            font_size_large: "20px".to_string(),
        }
    }
}

/// Logo configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Logo {
    pub url: String,
    pub alt_text: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub link_url: Option<String>,
}

/// Favicon configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Favicon {
    pub ico_url: String,
    pub png_16_url: Option<String>,
    pub png_32_url: Option<String>,
    pub apple_touch_icon_url: Option<String>,
}

/// Email branding configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailBranding {
    pub from_name: String,
    pub from_email: String,
    pub reply_to_email: Option<String>,
    pub header_color: String,
    pub footer_text: Option<String>,
    pub logo_url: Option<String>,
}

/// Social media links.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SocialLinks {
    pub website: Option<String>,
    pub twitter: Option<String>,
    pub linkedin: Option<String>,
    pub facebook: Option<String>,
    pub github: Option<String>,
}

/// Complete tenant branding configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantBranding {
    pub tenant_id: Uuid,
    pub company_name: String,
    pub tagline: Option<String>,
    pub color_scheme: ColorScheme,
    pub typography: Typography,
    pub logo: Option<Logo>,
    pub logo_dark: Option<Logo>,
    pub favicon: Option<Favicon>,
    pub email_branding: Option<EmailBranding>,
    pub social_links: SocialLinks,
    pub custom_css: Option<String>,
    pub custom_domain: Option<String>,
    pub meta_tags: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TenantBranding {
    pub fn new(tenant_id: Uuid, company_name: impl Into<String>) -> Self {
        Self {
            tenant_id,
            company_name: company_name.into(),
            tagline: None,
            color_scheme: ColorScheme::default(),
            typography: Typography::default(),
            logo: None,
            logo_dark: None,
            favicon: None,
            email_branding: None,
            social_links: SocialLinks::default(),
            custom_css: None,
            custom_domain: None,
            meta_tags: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Generates CSS variables for the color scheme.
    pub fn generate_css_variables(&self) -> String {
        format!(
            r#":root {{
  --color-primary: {};
  --color-secondary: {};
  --color-accent: {};
  --color-background: {};
  --color-text: {};
  --color-error: {};
  --color-success: {};
  --color-warning: {};
  --font-family: {};
  --font-size-base: {};
  --font-size-small: {};
  --font-size-large: {};
}}"#,
            self.color_scheme.primary,
            self.color_scheme.secondary,
            self.color_scheme.accent,
            self.color_scheme.background,
            self.color_scheme.text,
            self.color_scheme.error,
            self.color_scheme.success,
            self.color_scheme.warning,
            self.typography.font_family,
            self.typography.font_size_base,
            self.typography.font_size_small,
            self.typography.font_size_large,
        )
    }

    /// Generates HTML meta tags.
    pub fn generate_meta_tags(&self) -> String {
        let mut tags = Vec::new();

        tags.push(format!(r#"<meta name="application-name" content="{}">"#, self.company_name));

        if let Some(tagline) = &self.tagline {
            tags.push(format!(r#"<meta name="description" content="{}">"#, tagline));
        }

        tags.push(format!(r#"<meta name="theme-color" content="{}">"#, self.color_scheme.primary));

        for (key, value) in &self.meta_tags {
            tags.push(format!(r#"<meta name="{}" content="{}">"#, key, value));
        }

        tags.join("\n")
    }

    /// Validates branding configuration.
    pub fn validate(&self) -> TenantResult<()> {
        // Validate color hex codes
        for color in [
            &self.color_scheme.primary,
            &self.color_scheme.secondary,
            &self.color_scheme.accent,
            &self.color_scheme.background,
            &self.color_scheme.text,
        ] {
            if !color.starts_with('#') || (color.len() != 7 && color.len() != 4) {
                return Err(TenantError::ValidationError(
                    format!("Invalid color code: {}", color)
                ));
            }
        }

        // Validate URLs if present
        if let Some(logo) = &self.logo {
            if !logo.url.starts_with("http://") && !logo.url.starts_with("https://") {
                return Err(TenantError::ValidationError(
                    "Logo URL must be HTTP or HTTPS".to_string()
                ));
            }
        }

        Ok(())
    }
}

/// Manager for tenant branding configurations.
pub struct BrandingManager {
    brandings: HashMap<Uuid, TenantBranding>,
}

impl BrandingManager {
    pub fn new() -> Self {
        Self {
            brandings: HashMap::new(),
        }
    }

    /// Gets branding for a tenant.
    pub fn get(&self, tenant_id: Uuid) -> Option<&TenantBranding> {
        self.brandings.get(&tenant_id)
    }

    /// Gets branding for a tenant or returns default.
    pub fn get_or_default(&self, tenant_id: Uuid, company_name: &str) -> TenantBranding {
        self.brandings
            .get(&tenant_id)
            .cloned()
            .unwrap_or_else(|| TenantBranding::new(tenant_id, company_name))
    }

    /// Sets branding for a tenant.
    pub fn set(&mut self, branding: TenantBranding) -> TenantResult<()> {
        branding.validate()?;
        let tenant_id = branding.tenant_id;
        self.brandings.insert(tenant_id, branding);
        Ok(())
    }

    /// Updates specific branding fields.
    pub fn update(
        &mut self,
        tenant_id: Uuid,
        update_fn: impl FnOnce(&mut TenantBranding),
    ) -> TenantResult<()> {
        let branding = self.brandings.get_mut(&tenant_id).ok_or_else(|| {
            TenantError::TenantNotFound(tenant_id.to_string())
        })?;

        update_fn(branding);
        branding.updated_at = Utc::now();
        branding.validate()?;

        Ok(())
    }

    /// Deletes branding for a tenant.
    pub fn delete(&mut self, tenant_id: Uuid) {
        self.brandings.remove(&tenant_id);
    }

    /// Exports branding as JSON.
    pub fn export(&self, tenant_id: Uuid) -> TenantResult<String> {
        let branding = self.brandings.get(&tenant_id).ok_or_else(|| {
            TenantError::TenantNotFound(tenant_id.to_string())
        })?;

        serde_json::to_string_pretty(branding)
            .map_err(|e| TenantError::ConfigError(e.to_string()))
    }

    /// Imports branding from JSON.
    pub fn import(&mut self, json: &str) -> TenantResult<()> {
        let branding: TenantBranding = serde_json::from_str(json)
            .map_err(|e| TenantError::ConfigError(e.to_string()))?;

        branding.validate()?;
        self.brandings.insert(branding.tenant_id, branding);

        Ok(())
    }
}

impl Default for BrandingManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Branding template for quick setup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandingTemplate {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub color_scheme: ColorScheme,
    pub typography: Typography,
}

impl BrandingTemplate {
    pub fn professional_blue() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Professional Blue".to_string(),
            description: Some("Clean professional blue theme".to_string()),
            color_scheme: ColorScheme {
                primary: "#0066cc".to_string(),
                secondary: "#004080".to_string(),
                accent: "#ff9900".to_string(),
                background: "#ffffff".to_string(),
                text: "#333333".to_string(),
                error: "#d32f2f".to_string(),
                success: "#388e3c".to_string(),
                warning: "#f57c00".to_string(),
            },
            typography: Typography::default(),
        }
    }

    pub fn modern_green() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Modern Green".to_string(),
            description: Some("Fresh modern green theme".to_string()),
            color_scheme: ColorScheme {
                primary: "#00a86b".to_string(),
                secondary: "#006644".to_string(),
                accent: "#ffb300".to_string(),
                background: "#ffffff".to_string(),
                text: "#2c3e50".to_string(),
                error: "#e74c3c".to_string(),
                success: "#27ae60".to_string(),
                warning: "#f39c12".to_string(),
            },
            typography: Typography::default(),
        }
    }

    pub fn dark_mode() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Dark Mode".to_string(),
            description: Some("Dark theme for reduced eye strain".to_string()),
            color_scheme: ColorScheme {
                primary: "#3f51b5".to_string(),
                secondary: "#7986cb".to_string(),
                accent: "#ffc107".to_string(),
                background: "#121212".to_string(),
                text: "#e0e0e0".to_string(),
                error: "#cf6679".to_string(),
                success: "#81c784".to_string(),
                warning: "#ffb74d".to_string(),
            },
            typography: Typography::default(),
        }
    }

    /// Applies this template to tenant branding.
    pub fn apply_to(&self, branding: &mut TenantBranding) {
        branding.color_scheme = self.color_scheme.clone();
        branding.typography = self.typography.clone();
        branding.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_branding() {
        let branding = TenantBranding::new(Uuid::new_v4(), "ACME Corp");
        assert_eq!(branding.company_name, "ACME Corp");
        assert!(branding.validate().is_ok());
    }

    #[test]
    fn test_css_generation() {
        let branding = TenantBranding::new(Uuid::new_v4(), "Test");
        let css = branding.generate_css_variables();
        assert!(css.contains("--color-primary"));
        assert!(css.contains("--font-family"));
    }

    #[test]
    fn test_branding_manager() {
        let mut manager = BrandingManager::new();
        let tenant_id = Uuid::new_v4();
        let branding = TenantBranding::new(tenant_id, "Test Company");

        assert!(manager.set(branding).is_ok());
        assert!(manager.get(tenant_id).is_some());
    }

    #[test]
    fn test_branding_templates() {
        let template = BrandingTemplate::professional_blue();
        assert_eq!(template.name, "Professional Blue");

        let mut branding = TenantBranding::new(Uuid::new_v4(), "Test");
        template.apply_to(&mut branding);
        assert_eq!(branding.color_scheme.primary, "#0066cc");
    }
}
