---
title: Deployment & Environment Strategy
version: 1.0.0
status: Active
last_updated: 2026-01-12
owner: DevOps Team
category: Operations
---

# Deployment & Environment Strategy

> [!NOTE]
> **Purpose**: Document how UPFlame UAC is deployed across different environments.

---

## 1. Environment Strategy

### 1.1 Environments

| Environment | Purpose | URL | Database |
|-------------|---------|-----|----------|
| **Development** | Local development | localhost:8080 | SQLite |
| **Staging** | Pre-production testing | auth-staging.upflame.com | MySQL (staging) |
| **Production** | Live system | auth.upflame.com | MySQL (production) |

---

## 2. Docker Deployment

### 2.1 Dockerfile

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/auth-platform /usr/local/bin/
EXPOSE 8080
CMD ["auth-platform"]
```

### 2.2 Docker Compose

```yaml
version: '3.8'

services:
  auth-api:
    build: .
    ports:
      - "8080:8080"
    environment:
      - AUTH__DATABASE__MYSQL_URL=mysql://user:pass@db:3306/auth
      - AUTH__SECURITY__JWT_SECRET=${JWT_SECRET}
    depends_on:
      - db
      - redis

  db:
    image: mysql:8.0
    environment:
      - MYSQL_ROOT_PASSWORD=${DB_ROOT_PASSWORD}
      - MYSQL_DATABASE=auth
    volumes:
      - db-data:/var/lib/mysql

  redis:
    image: redis:7-alpine
    volumes:
      - redis-data:/data

volumes:
  db-data:
  redis-data:
```

---

## 3. Kubernetes Deployment

### 3.1 Deployment Manifest

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: auth-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: auth-api
  template:
    metadata:
      labels:
        app: auth-api
    spec:
      containers:
      - name: auth-api
        image: upflame/auth-platform:latest
        ports:
        - containerPort: 8080
        env:
        - name: AUTH__DATABASE__MYSQL_URL
          valueFrom:
            secretKeyRef:
              name: auth-secrets
              key: database-url
        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "256Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
```

---

## 4. Database Migrations

### 4.1 Migration Process

```bash
# Run migrations
sqlx migrate run --database-url $DATABASE_URL

# Revert last migration (if needed)
sqlx migrate revert --database-url $DATABASE_URL
```

### 4.2 Migration Strategy

1. **Test on staging first**
2. **Backup production database**
3. **Run migration during maintenance window**
4. **Verify migration success**
5. **Monitor for errors**

---

## 5. Secrets Management

### 5.1 Environment Variables

**Development** (`.env`):
```bash
AUTH__DATABASE__SQLITE_URL=sqlite:./dev.db
AUTH__SECURITY__JWT_SECRET=dev-secret-key
```

**Production** (Kubernetes Secrets):
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: auth-secrets
type: Opaque
data:
  database-url: <base64-encoded>
  jwt-secret: <base64-encoded>
```

---

## 6. Configuration Separation

### 6.1 Config Files

```
config/
├── default.toml       # Base config
├── development.toml   # Dev overrides
├── staging.toml       # Staging overrides
└── production.toml    # Production overrides
```

### 6.2 Loading Order

1. Load `default.toml`
2. Load environment-specific file (`$ENV.toml`)
3. Override with environment variables (`AUTH__*`)

---

## 7. Blue-Green Deployment

### 7.1 Strategy

1. **Deploy to green environment** (new version)
2. **Run smoke tests** on green
3. **Switch traffic** from blue to green
4. **Monitor** for errors
5. **Keep blue** as rollback option for 24 hours

### 7.2 Rollback

If issues detected:
1. **Switch traffic** back to blue
2. **Investigate** issue in green
3. **Fix** and redeploy

---

## 8. Monitoring Setup

### 8.1 Prometheus

```yaml
scrape_configs:
  - job_name: 'auth-api'
    static_configs:
      - targets: ['auth-api:8080']
```

### 8.2 Grafana Dashboards

- **Authentication Metrics**: Login success rate, latency
- **System Metrics**: CPU, memory, disk
- **Database Metrics**: Connection pool, query latency

---

**Document Status**: Active  
**Owner**: DevOps Team
