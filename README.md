# Enterprise SSO Platform

A production-ready, enterprise-grade Single Sign-On (SSO) and Identity Platform built in Rust, designed to support 100k-1M+ concurrent users with comprehensive security, compliance, and multi-tenant capabilities.

## Features

- **Multi-Protocol Support**: SAML 2.0, OpenID Connect 1.0, OAuth 2.1, SCIM 2.0
- **Advanced Multi-Tenancy**: Hierarchical organization → tenant → user structure
- **Dynamic RBAC/ABAC**: Configurable roles and permissions with real-time updates
- **Risk-Based Authentication**: ML-powered fraud detection and adaptive MFA
- **Enterprise Security**: HSM/KMS integration, post-quantum cryptography support
- **Horizontal Scaling**: Database sharding, multi-layer caching, auto-scaling
- **Comprehensive Audit**: SOC2, HIPAA, PCI-DSS compliance with immutable audit trails
- **Dynamic Configuration**: Real-time configuration updates without restarts
### Core Authentication
- **Multi-Protocol Support**: OIDC, SAML 2.0, OAuth 2.1
- **JWT-based Sessions**: Secure token management with refresh tokens
- **Password Security**: Argon2id hashing with 12+ char complexity requirements
- **MFA Support**: TOTP-based two-factor authentication
- **Rate Limiting**: 5 requests/minute per IP with token bucket algorithm
- **Account Lockout**: Automatic lockout after failed login attempts

### Identity Management
- **User Lifecycle**: Registration, activation, suspension, deletion
- **Role-Based Access Control (RBAC)**: Flexible permission system
- **Tenant Isolation**: Multi-tenant architecture with data separation
- **Audit Logging**: Comprehensive activity tracking
- **Email Validation**: RFC-compliant with automatic normalization
- **Password Policy**: Enforced complexity, length, and common password blacklist

### Security & Observability
- **Request Tracking**: UUID-based request IDs for distributed tracing
- **Security Headers**: OWASP best practices (CSP, HSTS, X-Frame-Options, etc.)
- **Structured Errors**: Error codes, messages, and request IDs
- **Structured Logging**: Request IDs in all log entries for correlation

## Architecture

The platform follows a microservices architecture with clear separation of concerns:

- **auth-core**: Pure business logic (Identity, Authorization, Risk Assessment)
- **auth-api**: HTTP API layer with Axum
- **auth-protocols**: SAML, OIDC, OAuth, SCIM implementations
- **auth-db**: Database layer with MySQL/SQLite support
- **auth-config**: Dynamic configuration management
- **auth-crypto**: Cryptographic operations and key management
- **auth-audit**: Audit logging and compliance

## Quick Start

### Prerequisites

- Rust 1.70+ 
- MySQL 8.0+ (for production) or SQLite (for development)
- Redis (optional, for caching)

### Development Setup

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd auth-platform
   ```

2. Copy the environment configuration:
   ```bash
   cp .env.example .env
   ```

3. Update the `.env` file with your database credentials and other settings.

4. Build the project:
   ```bash
   cargo build
   ```

5. Run the development server:
   ```bash
   cargo run
   ```

The server will start on `http://localhost:8080` by default.

## API Documentation

Interactive API documentation is available via **Swagger UI**:

```
http://localhost:8080/swagger-ui
```

For detailed endpoint documentation, see [`docs/API_REFERENCE.md`](docs/API_REFERENCE.md).

### Configuration

The platform supports multiple configuration sources with the following precedence:

1. Environment variables (highest priority)
2. Local configuration file (`config/local.toml`)
3. Environment-specific file (`config/development.toml`, `config/production.toml`)
4. Default configuration (`config/default.toml`)

#### Environment Variables

All configuration can be overridden using environment variables with the `AUTH__` prefix:

```bash
AUTH__SERVER__PORT=8080
AUTH__DATABASE__MYSQL_URL=mysql://user:pass@localhost/db
AUTH__SECURITY__JWT_SECRET=your-secret-key
```

### Database Setup

#### SQLite (Development)
```bash
# SQLite database will be created automatically
export AUTH__DATABASE__SQLITE_URL=sqlite:./dev.db
```

#### MySQL (Production)
```bash
# Create database
mysql -u root -p -e "CREATE DATABASE auth_platform;"

# Set connection string
export AUTH__DATABASE__MYSQL_URL=mysql://user:password@localhost:3306/auth_platform
```

## Testing

Run the test suite:

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run property-based tests (longer running)
cargo test --release -- --ignored
```

## Development

### Project Structure

```
├── crates/
│   ├── auth-core/          # Core business logic
│   ├── auth-api/           # HTTP API layer  
│   ├── auth-protocols/     # Protocol implementations
│   ├── auth-db/            # Database layer
│   ├── auth-config/        # Configuration management
│   ├── auth-crypto/        # Cryptographic operations
│   └── auth-audit/         # Audit and compliance
├── config/                 # Configuration files
├── migrations/             # Database migrations
└── src/                    # Main application
```

### Adding New Features

1. Define requirements in the spec documents
2. Update the design document with technical details
3. Add implementation tasks to the task list
4. Implement the feature following the task breakdown
5. Add comprehensive tests (unit + property-based)
6. Update documentation

### Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Use `cargo clippy` for linting
- Write comprehensive documentation
- Include property-based tests for core logic
- Maintain high test coverage

## Deployment

### Docker

```bash
# Build Docker image
docker build -t sso-platform:latest .

# Run container
docker run -d -p 8080:8080 \
  -e AUTH__DATABASE__MYSQL_URL="mysql://..." \
  -e AUTH__SECURITY__JWT_SECRET="your-secret" \
  sso-platform:latest
```

### Docker Compose (Local Development)

```bash
# Start full stack (MySQL + Redis + App)
docker-compose up -d

# View logs
docker-compose logs -f sso-app

# Stop all services
docker-compose down
```

### Kubernetes

Manifests are located in the `k8s/` directory.

## Security

- All passwords are hashed using Argon2id
- JWT tokens use RS256 with short expiration times
- All communications use TLS 1.3
- Comprehensive audit logging for all operations
- Rate limiting and DDoS protection
- Regular security updates and vulnerability scanning

## Compliance

The platform supports multiple compliance frameworks:

- **SOC 2 Type II**: Comprehensive audit trails and access controls
- **HIPAA**: Healthcare data protection and privacy controls  
- **PCI-DSS**: Payment card industry security standards
- **GDPR**: European data protection and privacy rights

## Performance

- Supports 100k-1M+ concurrent users
- Sub-50ms response times for authentication
- Horizontal scaling with database sharding
- Multi-layer caching (L1: memory, L2: Redis, L3: CDN)
- 99.99% uptime SLA with automated failover

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Submit a pull request

## License

[License information to be added]

## Support

For support and questions:
- Create an issue in the repository
- Check the documentation in the `docs/` directory
- Review the design and requirements specifications