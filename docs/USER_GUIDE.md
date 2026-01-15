# User Guide & Operational Flows

This guide outlines common usage patterns and operational procedures for the Enterprise SSO Platform.

## Authentication Flows

### 1. Standard Login (OIDC Authorization Code)

This is the recommended flow for server-side web applications.

1.  **Initiate Login**: Redirect user to `/oauth/authorize`.
    ```http
    GET /oauth/authorize?response_type=code&client_id=...&redirect_uri=...&scope=openid profile
    ```
2.  **User Authentication**: User enters credentials.
3.  **Consent**: User grants permission (if applicable).
4.  **Code Exchange**: App receives code and exchanges it for tokens.
    ```http
    POST /oauth/token
    grant_type=authorization_code&code=...
    ```

### 2. Machine-to-Machine (Client Credentials)

For service-to-service communication.

```http
POST /oauth/token
grant_type=client_credentials&client_id=...&client_secret=...
```

## Operational Procedures

### Key Rotation

To rotate signing keys without downtime:

1.  **Generate New Key**: Use the CLI or admin API to generate a new key pair.
    ```bash
    # Example CLI command (fictional)
    ./sso-cli keys generate --alg RS256 --use signing
    ```
2.  **Propagate**: The new public key is immediately available at `/.well-known/jwks.json`.
3.  **Retire Old Key**: After the token TTL (e.g., 1 hour), mark the old key as inactive.

### Disaster Recovery

In case of primary database failure:

1.  **Failover**: Automated failover to the standby replica is handled by the cloud provider or orchestrator.
2.  **Reconnect**: The application will automatically attempt to reconnect.
3.  **Data Integrity**: Check audit logs in `auth-audit` for any interrupted transactions.
