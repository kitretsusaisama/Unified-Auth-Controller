use auth_protocols::OAuthService;

#[tokio::main]
async fn main() {
    println!("Testing OAuth 2.1 Service...");

    let oauth_service = OAuthService::new(
        "client_id".to_string(),
        "client_secret".to_string(),
        "https://provider.com/authorize".to_string(),
        "https://provider.com/token".to_string(),
    );

    let auth_url = oauth_service
        .authorize_url("https://sso.com/callback", "random_state")
        .expect("Failed to generate auth URL");

    println!("Generated Auth URL: {}", auth_url);
    assert!(auth_url.contains("client_id=client_id"));
    assert!(auth_url.contains("response_type=code"));

    let token = oauth_service
        .exchange_token("valid_code", "https://sso.com/callback")
        .await
        .expect("Failed to exchange token");

    println!("Exchanged Token: {}", token);
    assert_eq!(token, "mock_access_token");

    println!("OAuth Test Passed!");
}
