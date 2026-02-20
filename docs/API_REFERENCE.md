# API Reference (V1)

## Base URL
`https://api.auth.example.com/v1`

## Authentication

### Login
`POST /auth/login`
- **Body**: `{ "email": "user@example.com", "password": "..." }`
- **Response**: `AuthResponse` (Tokens)

### Register
`POST /auth/register`
- **Body**: `{ "email": "...", "password": "..." }`

## OIDC Endpoints

### Authorization
`GET /auth/authorize`
- Standard OIDC params: `client_id`, `redirect_uri`, `response_type=code`, `scope`.

### Token
`POST /auth/token`
- Standard OIDC token exchange.

## User Management

### Ban User
`POST /users/:id/ban`
- Requires Admin role.

### Activate User
`POST /users/:id/activate`
- Requires Admin role.

## Authorization

### Create Role
`POST /auth/roles`
- Define new roles with permissions.
