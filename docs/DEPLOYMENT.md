# Deployment Guide

## Docker Deployment

The platform provides a production-optimized `Dockerfile`.

### Build
```bash
docker build -t sso-platform:latest .
```

### Run
```bash
docker run -d -p 8080:8080 \
  -e AUTH__DATABASE__MYSQL_URL="mysql://..." \
  -e AUTH__SECURITY__JWT_SECRET="correct-horse-battery-staple" \
  sso-platform:latest
```

## Kubernetes Deployment

Manifests are located in the `k8s/` directory.

### Prerequisites
- Kubernetes cluster (v1.24+)
- Ingress Controller (Nginx recommended)
- Cert-Manager (for TLS)

### 1. Secrets Management
Create a secret for sensitive configuration:
```bash
kubectl create secret generic sso-secrets \
  --from-literal=db-url='mysql://...' \
  --from-literal=jwt-secret='...'
```

### 2. Deploy
```bash
kubectl apply -f k8s/configmap.yaml
kubectl apply -f k8s/deployment.yaml
kubectl apply -f k8s/service.yaml
```

### 3. Expose
```bash
kubectl apply -f k8s/ingress.yaml
```

## Health Checks

- **Liveness Probe**: `/health/live` - Returns 200 OK if the process is running.
- **Readiness Probe**: `/health/ready` - Returns 200 OK if DB and Redis dependencies are reachable.
