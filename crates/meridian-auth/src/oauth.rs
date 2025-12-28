//! OAuth2 provider support for third-party authentication

use crate::error::{AuthError, AuthResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OAuth2 provider type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OAuthProvider {
    Google,
    GitHub,
    Microsoft,
}

impl OAuthProvider {
    /// Get provider name
    pub fn name(&self) -> &str {
        match self {
            OAuthProvider::Google => "google",
            OAuthProvider::GitHub => "github",
            OAuthProvider::Microsoft => "microsoft",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> AuthResult<Self> {
        match s.to_lowercase().as_str() {
            "google" => Ok(OAuthProvider::Google),
            "github" => Ok(OAuthProvider::GitHub),
            "microsoft" => Ok(OAuthProvider::Microsoft),
            _ => Err(AuthError::UnsupportedOAuthProvider(s.to_string())),
        }
    }
}

/// OAuth2 configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    /// Client ID
    pub client_id: String,
    /// Client secret
    pub client_secret: String,
    /// Redirect URI
    pub redirect_uri: String,
    /// Authorization endpoint
    pub authorize_url: String,
    /// Token endpoint
    pub token_url: String,
    /// User info endpoint
    pub user_info_url: String,
    /// Scopes to request
    pub scopes: Vec<String>,
}

impl OAuthConfig {
    /// Create Google OAuth configuration
    pub fn google(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_uri,
            authorize_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token_url: "https://oauth2.googleapis.com/token".to_string(),
            user_info_url: "https://www.googleapis.com/oauth2/v2/userinfo".to_string(),
            scopes: vec![
                "openid".to_string(),
                "email".to_string(),
                "profile".to_string(),
            ],
        }
    }

    /// Create GitHub OAuth configuration
    pub fn github(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_uri,
            authorize_url: "https://github.com/login/oauth/authorize".to_string(),
            token_url: "https://github.com/login/oauth/access_token".to_string(),
            user_info_url: "https://api.github.com/user".to_string(),
            scopes: vec!["user:email".to_string()],
        }
    }

    /// Create Microsoft OAuth configuration
    pub fn microsoft(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_uri,
            authorize_url: "https://login.microsoftonline.com/common/oauth2/v2.0/authorize"
                .to_string(),
            token_url: "https://login.microsoftonline.com/common/oauth2/v2.0/token".to_string(),
            user_info_url: "https://graph.microsoft.com/v1.0/me".to_string(),
            scopes: vec!["openid".to_string(), "email".to_string(), "profile".to_string()],
        }
    }
}

/// OAuth2 authorization URL parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationUrlParams {
    pub client_id: String,
    pub redirect_uri: String,
    pub response_type: String,
    pub scope: String,
    pub state: String,
}

/// OAuth2 token response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

/// OAuth2 user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthUserInfo {
    /// User ID from provider
    pub id: String,
    /// User email
    pub email: String,
    /// Email verification status
    #[serde(default)]
    pub email_verified: bool,
    /// Display name
    pub name: Option<String>,
    /// Profile picture URL
    pub picture: Option<String>,
    /// Additional user data
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// OAuth2 provider trait
#[async_trait]
pub trait OAuth2Provider: Send + Sync {
    /// Get provider type
    fn provider_type(&self) -> OAuthProvider;

    /// Generate authorization URL
    fn authorization_url(&self, state: String) -> AuthResult<String>;

    /// Exchange authorization code for access token
    async fn exchange_code(&self, code: String) -> AuthResult<TokenResponse>;

    /// Get user information using access token
    async fn get_user_info(&self, access_token: &str) -> AuthResult<OAuthUserInfo>;
}

/// Generic OAuth2 provider implementation
pub struct GenericOAuth2Provider {
    provider: OAuthProvider,
    config: OAuthConfig,
}

impl GenericOAuth2Provider {
    /// Create a new OAuth2 provider
    pub fn new(provider: OAuthProvider, config: OAuthConfig) -> Self {
        Self { provider, config }
    }

    /// Create Google provider
    pub fn google(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self::new(
            OAuthProvider::Google,
            OAuthConfig::google(client_id, client_secret, redirect_uri),
        )
    }

    /// Create GitHub provider
    pub fn github(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self::new(
            OAuthProvider::GitHub,
            OAuthConfig::github(client_id, client_secret, redirect_uri),
        )
    }

    /// Create Microsoft provider
    pub fn microsoft(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self::new(
            OAuthProvider::Microsoft,
            OAuthConfig::microsoft(client_id, client_secret, redirect_uri),
        )
    }
}

#[async_trait]
impl OAuth2Provider for GenericOAuth2Provider {
    fn provider_type(&self) -> OAuthProvider {
        self.provider
    }

    fn authorization_url(&self, state: String) -> AuthResult<String> {
        let scope = self.config.scopes.join(" ");
        let params = [
            ("client_id", self.config.client_id.as_str()),
            ("redirect_uri", self.config.redirect_uri.as_str()),
            ("response_type", "code"),
            ("scope", &scope),
            ("state", &state),
        ];

        let query = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        Ok(format!("{}?{}", self.config.authorize_url, query))
    }

    async fn exchange_code(&self, _code: String) -> AuthResult<TokenResponse> {
        // In a real implementation, this would make an HTTP request to the token endpoint
        // For this example, we'll return a simulated response

        // This is where you would use an HTTP client like reqwest:
        // let client = reqwest::Client::new();
        // let params = [
        //     ("client_id", &self.config.client_id),
        //     ("client_secret", &self.config.client_secret),
        //     ("code", &code),
        //     ("redirect_uri", &self.config.redirect_uri),
        //     ("grant_type", &"authorization_code".to_string()),
        // ];
        // let response = client
        //     .post(&self.config.token_url)
        //     .form(&params)
        //     .send()
        //     .await
        //     .map_err(|e| AuthError::OAuthTokenExchangeFailed(e.to_string()))?;
        // let token: TokenResponse = response
        //     .json()
        //     .await
        //     .map_err(|e| AuthError::OAuthTokenExchangeFailed(e.to_string()))?;
        // Ok(token)

        // Simulated response for demonstration
        Err(AuthError::OAuthTokenExchangeFailed(
            "Not implemented: requires HTTP client".to_string(),
        ))
    }

    async fn get_user_info(&self, _access_token: &str) -> AuthResult<OAuthUserInfo> {
        // In a real implementation, this would make an HTTP request to the user info endpoint
        // For this example, we'll return an error indicating it needs implementation

        // This is where you would use an HTTP client like reqwest:
        // let client = reqwest::Client::new();
        // let response = client
        //     .get(&self.config.user_info_url)
        //     .bearer_auth(access_token)
        //     .send()
        //     .await
        //     .map_err(|e| AuthError::OAuthProviderError(e.to_string()))?;
        //
        // let user_info: serde_json::Value = response
        //     .json()
        //     .await
        //     .map_err(|e| AuthError::OAuthProviderError(e.to_string()))?;
        //
        // // Parse provider-specific response format
        // let oauth_user = match self.provider {
        //     OAuthProvider::Google => parse_google_user_info(user_info)?,
        //     OAuthProvider::GitHub => parse_github_user_info(user_info)?,
        //     OAuthProvider::Microsoft => parse_microsoft_user_info(user_info)?,
        // };
        //
        // Ok(oauth_user)

        // Simulated response for demonstration
        Err(AuthError::OAuthProviderError(
            "Not implemented: requires HTTP client".to_string(),
        ))
    }
}

/// OAuth2 manager for handling multiple providers
pub struct OAuthManager {
    providers: HashMap<OAuthProvider, Box<dyn OAuth2Provider>>,
}

impl OAuthManager {
    /// Create a new OAuth manager
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Add a provider
    pub fn add_provider(&mut self, provider: Box<dyn OAuth2Provider>) {
        self.providers.insert(provider.provider_type(), provider);
    }

    /// Get provider by type
    pub fn get_provider(&self, provider_type: OAuthProvider) -> Option<&dyn OAuth2Provider> {
        self.providers.get(&provider_type).map(|p| p.as_ref())
    }

    /// Generate authorization URL for a provider
    pub fn authorization_url(
        &self,
        provider_type: OAuthProvider,
        state: String,
    ) -> AuthResult<String> {
        let provider = self
            .get_provider(provider_type)
            .ok_or_else(|| AuthError::UnsupportedOAuthProvider(provider_type.name().to_string()))?;

        provider.authorization_url(state)
    }

    /// Exchange code for token
    pub async fn exchange_code(
        &self,
        provider_type: OAuthProvider,
        code: String,
    ) -> AuthResult<TokenResponse> {
        let provider = self
            .get_provider(provider_type)
            .ok_or_else(|| AuthError::UnsupportedOAuthProvider(provider_type.name().to_string()))?;

        provider.exchange_code(code).await
    }

    /// Get user info
    pub async fn get_user_info(
        &self,
        provider_type: OAuthProvider,
        access_token: &str,
    ) -> AuthResult<OAuthUserInfo> {
        let provider = self
            .get_provider(provider_type)
            .ok_or_else(|| AuthError::UnsupportedOAuthProvider(provider_type.name().to_string()))?;

        provider.get_user_info(access_token).await
    }
}

impl Default for OAuthManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper module for URL encoding (simple implementation)
mod urlencoding {
    pub fn encode(s: &str) -> String {
        s.chars()
            .map(|c| match c {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
                _ => format!("%{:02X}", c as u8),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_from_string() {
        assert_eq!(
            OAuthProvider::from_str("google").unwrap(),
            OAuthProvider::Google
        );
        assert_eq!(
            OAuthProvider::from_str("github").unwrap(),
            OAuthProvider::GitHub
        );
        assert_eq!(
            OAuthProvider::from_str("microsoft").unwrap(),
            OAuthProvider::Microsoft
        );
        assert!(OAuthProvider::from_str("unknown").is_err());
    }

    #[test]
    fn test_google_config() {
        let config = OAuthConfig::google(
            "client_id".to_string(),
            "client_secret".to_string(),
            "http://localhost/callback".to_string(),
        );

        assert_eq!(config.client_id, "client_id");
        assert!(config.authorize_url.contains("google"));
        assert!(config.scopes.contains(&"email".to_string()));
    }

    #[test]
    fn test_github_config() {
        let config = OAuthConfig::github(
            "client_id".to_string(),
            "client_secret".to_string(),
            "http://localhost/callback".to_string(),
        );

        assert!(config.authorize_url.contains("github"));
        assert!(config.scopes.contains(&"user:email".to_string()));
    }

    #[test]
    fn test_microsoft_config() {
        let config = OAuthConfig::microsoft(
            "client_id".to_string(),
            "client_secret".to_string(),
            "http://localhost/callback".to_string(),
        );

        assert!(config.authorize_url.contains("microsoft"));
        assert!(config.scopes.contains(&"email".to_string()));
    }

    #[test]
    fn test_authorization_url() {
        let provider = GenericOAuth2Provider::google(
            "test_client".to_string(),
            "test_secret".to_string(),
            "http://localhost/callback".to_string(),
        );

        let url = provider.authorization_url("state123".to_string()).unwrap();

        assert!(url.contains("client_id=test_client"));
        assert!(url.contains("state=state123"));
        assert!(url.contains("response_type=code"));
    }

    #[test]
    fn test_oauth_manager() {
        let mut manager = OAuthManager::new();

        let google = Box::new(GenericOAuth2Provider::google(
            "client_id".to_string(),
            "client_secret".to_string(),
            "http://localhost/callback".to_string(),
        ));

        manager.add_provider(google);

        assert!(manager.get_provider(OAuthProvider::Google).is_some());
        assert!(manager.get_provider(OAuthProvider::GitHub).is_none());
    }

    #[test]
    fn test_url_encoding() {
        let encoded = urlencoding::encode("hello world");
        assert_eq!(encoded, "hello%20world");

        let encoded = urlencoding::encode("test@example.com");
        assert!(encoded.contains("%40")); // @ encoded
    }
}
