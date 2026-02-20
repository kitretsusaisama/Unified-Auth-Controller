#[allow(unused_imports)]
use auth_protocols::oidc::OidcConfig;
#[allow(unused_imports)]
use auth_protocols::OidcService;
use proptest::prelude::*;

#[allow(unused_imports)]
fn main() {
    println!("Running Protocol Property Tests...");
    println!("Please run: cargo test --bin test_protocol_property");
}

proptest! {
    #[test]
    fn test_oidc_url_generation(
        client_id in "[a-zA-Z0-9]{10}",
        client_secret in "[a-zA-Z0-9]{20}",
        redirect_uri in "https://[a-z]{5}\\.com/callback"
    ) {
        let config = OidcConfig {
            issuer_url: "https://issuer.com".to_string(),
            client_id: client_id.clone(),
            client_secret: client_secret.clone(),
            redirect_url: redirect_uri.clone(),
        };

        let service = OidcService::new_manual(
            config,
            "https://issuer.com/auth".to_string(),
            "https://issuer.com/token".to_string(),
        ).unwrap();

        let (url, _, _) = service.get_authorization_url();

        assert!(url.contains(&client_id));
        assert!(url.contains("scope=openid"));
        // Basic URL encoding check - usually library handles this, we verify it happened
        // But the redirect_uri strategy generates safe chars mostly.
        // If we want to test encoding, we should inject special chars.
        // For now, simple containment is enough for property test.
        assert!(url.contains(&redirect_uri.replace("/", "%2F").replace(":", "%3A")));
    }
}
