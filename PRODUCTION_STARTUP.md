# Production Startup Guide

This guide provides instructions for running the SSO Platform in a production environment.

## Prerequisites

- Rust 1.70+ installed
- MySQL database server (or use SQLite for simpler deployments)
- Proper environment variables configured

## Environment Configuration

Copy the example environment file and configure for production:

```bash
cp .env.example .env
```

Important environment variables for production:

```bash
# Environment
AUTH__ENVIRONMENT=production

# Server configuration
AUTH__SERVER__PORT=8080
AUTH__SERVER__HOST=0.0.0.0

# Database configuration (choose one)
AUTH__DATABASE__MYSQL_URL=mysql://username:password@host:port/database
# OR
AUTH__DATABASE__SQLITE_URL=sqlite:./production.db

# Security configuration
AUTH__SECURITY__JWT_SECRET=very-long-secret-key-change-this-immediately
AUTH__SECURITY__JWT_EXPIRY_MINUTES=30
AUTH__SECURITY__REFRESH_TOKEN_EXPIRY_HOURS=720

# External services
AUTH__EXTERNAL_SERVICES__SMTP__HOST=smtp.gmail.com
AUTH__EXTERNAL_SERVICES__SMTP__PORT=587
AUTH__EXTERNAL_SERVICES__SMTP__USERNAME=your-email@gmail.com
AUTH__EXTERNAL_SERVICES__SMTP__PASSWORD=your-app-password
AUTH__EXTERNAL_SERVICES__SMTP__FROM_ADDRESS=your-email@gmail.com
```

## Build for Production

```bash
# Build release version
cargo build --release

# Run the application
cargo run --release --bin auth-platform
```

## Docker Deployment

Build and run with Docker:

```bash
# Build Docker image
docker build -t sso-platform .

# Run container
docker run -d \
  -p 8080:8080 \
  -e AUTH__ENVIRONMENT=production \
  -e AUTH__DATABASE__MYSQL_URL="mysql://..." \
  -e AUTH__SECURITY__JWT_SECRET="your-secret" \
  sso-platform
```

## Health Checks

The application provides health check endpoints:
- `/health` - Basic health check
- `/ready` - Readiness probe
- `/metrics` - Prometheus metrics

## Monitoring

- Application logs in JSON format
- OpenTelemetry support for distributed tracing
- Prometheus metrics endpoint

## Performance Tuning

For production environments, consider these optimizations:

- Increase database connection pool size: `AUTH__DATABASE__MAX_CONNECTIONS=50`
- Set appropriate timeouts
- Configure Redis cache for sessions
- Enable compression for HTTP responses

## Security Best Practices

- Use strong, randomly generated JWT secrets
- Enable HTTPS in production
- Regular security audits
- Keep dependencies updated
- Implement rate limiting
- Monitor authentication attempts