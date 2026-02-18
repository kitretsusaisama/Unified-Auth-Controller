pub mod oidc;
pub mod saml;
pub mod oauth;
pub mod discovery;

pub use oidc::OidcService;
pub use saml::SamlService;
pub use oauth::OAuthService;