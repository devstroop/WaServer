# Deployment Guide 🚀

Complete guide for deploying WhatsApp Engine in production environments.

## 📋 Overview

WhatsApp Engine supports multiple deployment strategies:
- **Docker Compose** - Simple single-server deployment
- **Kubernetes** - Scalable container orchestration
- **Systemd Service** - Traditional Linux service deployment
- **Cloud Platforms** - AWS, GCP, Azure deployment

## 🐳 Docker Deployment

### Prerequisites

- Docker 20.10+
- Docker Compose 2.0+
- 4GB+ RAM
- Chrome/Chromium dependencies

### Quick Start with Docker Compose

1. **Clone and Setup**:
```bash
git clone https://github.com/devstroop/whatsapp-engine-rust.git
cd whatsapp-engine-rust
cp docker/.env.example docker/.env.production
```

2. **Configure Environment**:
```bash
# Edit production environment
nano docker/.env.production
```

```bash
# docker/.env.production
# Server Configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# Security
AUTH_API_TOKEN=your-super-secure-random-token-here

# Browser Configuration
BROWSER_HEADLESS=true
BROWSER_TIMEOUT_MS=30000

# Logging
LOGGING_LEVEL=info
RUST_LOG=whatsapp_engine=info

# Database (if using PostgreSQL)
DATABASE_URL=postgresql://username:password@postgres:5432/whatsapp_engine

# Redis (for session storage)
REDIS_URL=redis://redis:6379

# Monitoring
METRICS_ENABLED=true
PROMETHEUS_PORT=9090
```

3. **Deploy with Production Compose**:
```bash
# Using the production script
./scripts/deploy-production.sh

# Or manually
docker-compose -f docker/docker-compose.production.yml up -d
```

4. **Verify Deployment**:
```bash
# Check service health
curl http://localhost:3000/health

# Check metrics
curl http://localhost:3000/metrics

# View logs
docker-compose -f docker/docker-compose.production.yml logs -f whatsapp-engine
```

### Production Docker Compose

```yaml
# docker/docker-compose.production.yml
version: '3.8'

services:
  whatsapp-engine:
    build:
      context: ..
      dockerfile: docker/Dockerfile.production
    container_name: whatsapp-engine
    restart: unless-stopped
    environment:
      - SERVER_HOST=0.0.0.0
      - SERVER_PORT=3000
      - AUTH_API_TOKEN=${AUTH_API_TOKEN}
      - BROWSER_HEADLESS=true
      - LOGGING_LEVEL=info
    ports:
      - "3000:3000"
    volumes:
      - whatsapp_sessions:/app/sessions
      - whatsapp_logs:/app/logs
    networks:
      - whatsapp_network
    depends_on:
      - redis
      - postgres
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s

  nginx:
    image: nginx:alpine
    container_name: whatsapp-nginx
    restart: unless-stopped
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.prod.conf:/etc/nginx/nginx.conf:ro
      - ./ssl:/etc/ssl/certs:ro
    networks:
      - whatsapp_network
    depends_on:
      - whatsapp-engine

  redis:
    image: redis:7-alpine
    container_name: whatsapp-redis
    restart: unless-stopped
    command: redis-server --appendonly yes --requirepass ${REDIS_PASSWORD}
    volumes:
      - redis_data:/data
    networks:
      - whatsapp_network

  postgres:
    image: postgres:15-alpine
    container_name: whatsapp-postgres
    restart: unless-stopped
    environment:
      - POSTGRES_DB=whatsapp_engine
      - POSTGRES_USER=${POSTGRES_USER}
      - POSTGRES_PASSWORD=${POSTGRES_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data
    networks:
      - whatsapp_network

  prometheus:
    image: prom/prometheus
    container_name: whatsapp-prometheus
    restart: unless-stopped
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus_data:/prometheus
    networks:
      - whatsapp_network

  grafana:
    image: grafana/grafana
    container_name: whatsapp-grafana
    restart: unless-stopped
    ports:
      - "3001:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_PASSWORD}
    volumes:
      - grafana_data:/var/lib/grafana
      - ./grafana/provisioning:/etc/grafana/provisioning:ro
    networks:
      - whatsapp_network

volumes:
  whatsapp_sessions:
  whatsapp_logs:
  redis_data:
  postgres_data:
  prometheus_data:
  grafana_data:

networks:
  whatsapp_network:
    driver: bridge
```

### SSL/TLS Setup

1. **Generate SSL Certificate**:
```bash
# Using Let's Encrypt
certbot certonly --standalone -d whatsapp.yourdomain.com

# Or self-signed for testing
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout docker/ssl/private.key \
  -out docker/ssl/certificate.crt
```

2. **Nginx SSL Configuration**:
```nginx
# docker/nginx.prod.conf
events {
    worker_connections 1024;
}

http {
    upstream backend {
        server whatsapp-engine:3000;
    }

    server {
        listen 80;
        server_name whatsapp.yourdomain.com;
        return 301 https://$server_name$request_uri;
    }

    server {
        listen 443 ssl http2;
        server_name whatsapp.yourdomain.com;

        ssl_certificate /etc/ssl/certs/certificate.crt;
        ssl_certificate_key /etc/ssl/certs/private.key;
        
        # Security headers
        add_header Strict-Transport-Security "max-age=63072000" always;
        add_header X-Frame-Options DENY always;
        add_header X-Content-Type-Options nosniff always;

        location / {
            proxy_pass http://backend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }
    }
}
```

## ☸️ Kubernetes Deployment

### Prerequisites

- Kubernetes 1.20+
- kubectl configured
- Ingress controller (nginx, traefik, etc.)
- Persistent storage support

### Namespace and Resources

```yaml
# k8s/namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: whatsapp-engine
---
apiVersion: v1
kind: Secret
metadata:
  name: whatsapp-secrets
  namespace: whatsapp-engine
type: Opaque
stringData:
  api-token: "your-super-secure-api-token"
  postgres-password: "secure-postgres-password"
  redis-password: "secure-redis-password"
```

### ConfigMap

```yaml
# k8s/configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: whatsapp-config
  namespace: whatsapp-engine
data:
  config.toml: |
    [server]
    host = "0.0.0.0"
    port = 3000

    [browser]
    headless = true
    timeout_ms = 30000

    [logging]
    level = "info"
```

### Deployment

```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: whatsapp-engine
  namespace: whatsapp-engine
spec:
  replicas: 2
  selector:
    matchLabels:
      app: whatsapp-engine
  template:
    metadata:
      labels:
        app: whatsapp-engine
    spec:
      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
        fsGroup: 1000
      containers:
      - name: whatsapp-engine
        image: whatsapp-engine:latest
        ports:
        - containerPort: 3000
        env:
        - name: AUTH_API_TOKEN
          valueFrom:
            secretKeyRef:
              name: whatsapp-secrets
              key: api-token
        - name: DATABASE_URL
          value: "postgresql://postgres:$(POSTGRES_PASSWORD)@postgres:5432/whatsapp_engine"
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: whatsapp-secrets
              key: postgres-password
        volumeMounts:
        - name: config
          mountPath: /app/config
        - name: sessions
          mountPath: /app/sessions
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /live
            port: 3000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 5
        securityContext:
          allowPrivilegeEscalation: false
          readOnlyRootFilesystem: true
          capabilities:
            drop:
            - ALL
      volumes:
      - name: config
        configMap:
          name: whatsapp-config
      - name: sessions
        persistentVolumeClaim:
          claimName: whatsapp-sessions-pvc
```

### Service and Ingress

```yaml
# k8s/service.yaml
apiVersion: v1
kind: Service
metadata:
  name: whatsapp-engine-service
  namespace: whatsapp-engine
spec:
  selector:
    app: whatsapp-engine
  ports:
  - port: 80
    targetPort: 3000
  type: ClusterIP
---
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: whatsapp-engine-ingress
  namespace: whatsapp-engine
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/rate-limit: "30"
spec:
  tls:
  - hosts:
    - whatsapp.yourdomain.com
    secretName: whatsapp-tls
  rules:
  - host: whatsapp.yourdomain.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: whatsapp-engine-service
            port:
              number: 80
```

### Persistent Storage

```yaml
# k8s/storage.yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: whatsapp-sessions-pvc
  namespace: whatsapp-engine
spec:
  accessModes:
  - ReadWriteOnce
  resources:
    requests:
      storage: 10Gi
  storageClassName: fast-ssd
```

### Deploy to Kubernetes

```bash
# Apply all configurations
kubectl apply -f k8s/

# Check deployment status
kubectl get pods -n whatsapp-engine
kubectl get services -n whatsapp-engine
kubectl get ingress -n whatsapp-engine

# View logs
kubectl logs -f deployment/whatsapp-engine -n whatsapp-engine

# Scale deployment
kubectl scale deployment whatsapp-engine --replicas=3 -n whatsapp-engine
```

## 🖥️ Systemd Service Deployment

### Build and Install

```bash
# Build release binary
cargo build --release --features api-server

# Install binary
sudo cp target/release/whatsapp-server /usr/local/bin/
sudo chmod +x /usr/local/bin/whatsapp-server

# Create service user
sudo useradd --system --shell /bin/false whatsapp-engine

# Create directories
sudo mkdir -p /etc/whatsapp-engine
sudo mkdir -p /var/lib/whatsapp-engine/sessions
sudo mkdir -p /var/log/whatsapp-engine

# Set permissions
sudo chown -R whatsapp-engine:whatsapp-engine /var/lib/whatsapp-engine
sudo chown -R whatsapp-engine:whatsapp-engine /var/log/whatsapp-engine
```

### Configuration

```toml
# /etc/whatsapp-engine/config.toml
[server]
host = "127.0.0.1"
port = 3000

[auth]
api_token = "your-secure-api-token"

[browser]
headless = true
timeout_ms = 30000

[logging]
level = "info"

[limits]
max_file_size_bytes = 16777216
```

### Systemd Service

```ini
# /etc/systemd/system/whatsapp-engine.service
[Unit]
Description=WhatsApp Engine API Server
After=network.target
Wants=network.target

[Service]
Type=exec
User=whatsapp-engine
Group=whatsapp-engine
WorkingDirectory=/var/lib/whatsapp-engine
ExecStart=/usr/local/bin/whatsapp-server
ExecReload=/bin/kill -HUP $MAINPID
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal
SyslogIdentifier=whatsapp-engine

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/whatsapp-engine /var/log/whatsapp-engine
CapabilityBoundingSet=
AmbientCapabilities=
PrivateDevices=true
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true

# Resource limits
LimitNOFILE=65536
MemoryMax=1G

[Install]
WantedBy=multi-user.target
```

### Manage Service

```bash
# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable whatsapp-engine
sudo systemctl start whatsapp-engine

# Check status
sudo systemctl status whatsapp-engine

# View logs
sudo journalctl -u whatsapp-engine -f

# Restart service
sudo systemctl restart whatsapp-engine
```

## ☁️ Cloud Platform Deployment

### AWS Deployment

#### ECS with Fargate

```yaml
# aws/task-definition.json
{
  "family": "whatsapp-engine",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "512",
  "memory": "1024",
  "executionRoleArn": "arn:aws:iam::account:role/ecsTaskExecutionRole",
  "taskRoleArn": "arn:aws:iam::account:role/ecsTaskRole",
  "containerDefinitions": [
    {
      "name": "whatsapp-engine",
      "image": "your-account.dkr.ecr.region.amazonaws.com/whatsapp-engine:latest",
      "portMappings": [
        {
          "containerPort": 3000,
          "protocol": "tcp"
        }
      ],
      "environment": [
        {
          "name": "SERVER_HOST",
          "value": "0.0.0.0"
        },
        {
          "name": "BROWSER_HEADLESS",
          "value": "true"
        }
      ],
      "secrets": [
        {
          "name": "AUTH_API_TOKEN",
          "valueFrom": "arn:aws:secretsmanager:region:account:secret:whatsapp-api-token"
        }
      ],
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/whatsapp-engine",
          "awslogs-region": "us-west-2",
          "awslogs-stream-prefix": "ecs"
        }
      }
    }
  ]
}
```

#### Application Load Balancer

```bash
# Create ALB
aws elbv2 create-load-balancer \
  --name whatsapp-engine-alb \
  --subnets subnet-12345 subnet-67890 \
  --security-groups sg-12345

# Create target group
aws elbv2 create-target-group \
  --name whatsapp-engine-targets \
  --protocol HTTP \
  --port 3000 \
  --vpc-id vpc-12345 \
  --target-type ip \
  --health-check-path /health
```

### Google Cloud Platform

#### Cloud Run Deployment

```yaml
# gcp/service.yaml
apiVersion: serving.knative.dev/v1
kind: Service
metadata:
  name: whatsapp-engine
  annotations:
    run.googleapis.com/ingress: all
spec:
  template:
    metadata:
      annotations:
        autoscaling.knative.dev/maxScale: "10"
        run.googleapis.com/memory: "1Gi"
        run.googleapis.com/cpu: "1"
    spec:
      containerConcurrency: 80
      containers:
      - image: gcr.io/your-project/whatsapp-engine:latest
        ports:
        - containerPort: 3000
        env:
        - name: AUTH_API_TOKEN
          valueFrom:
            secretKeyRef:
              name: whatsapp-secrets
              key: api-token
        resources:
          limits:
            memory: 1Gi
            cpu: 1000m
```

```bash
# Deploy to Cloud Run
gcloud run deploy whatsapp-engine \
  --image gcr.io/your-project/whatsapp-engine:latest \
  --platform managed \
  --region us-central1 \
  --allow-unauthenticated \
  --memory 1Gi \
  --cpu 1 \
  --max-instances 10
```

### Azure Container Instances

```yaml
# azure/container-group.yaml
apiVersion: 2021-03-01
location: eastus
name: whatsapp-engine
properties:
  containers:
  - name: whatsapp-engine
    properties:
      image: your-registry.azurecr.io/whatsapp-engine:latest
      resources:
        requests:
          cpu: 0.5
          memoryInGb: 1
      ports:
      - port: 3000
      environmentVariables:
      - name: SERVER_HOST
        value: 0.0.0.0
      - name: AUTH_API_TOKEN
        secureValue: your-secure-token
  osType: Linux
  restartPolicy: Always
  ipAddress:
    type: Public
    ports:
    - protocol: tcp
      port: 3000
```

## 📊 Monitoring & Observability

### Prometheus Configuration

```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'whatsapp-engine'
    static_configs:
      - targets: ['whatsapp-engine:3000']
    metrics_path: /metrics
    scrape_interval: 30s

  - job_name: 'node-exporter'
    static_configs:
      - targets: ['node-exporter:9100']
```

### Grafana Dashboards

```json
{
  "dashboard": {
    "title": "WhatsApp Engine Metrics",
    "panels": [
      {
        "title": "Request Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(http_requests_total[5m])",
            "legendFormat": "{{method}} {{endpoint}}"
          }
        ]
      },
      {
        "title": "Message Success Rate",
        "type": "stat",
        "targets": [
          {
            "expr": "rate(whatsapp_messages_sent_total[5m]) / rate(whatsapp_message_attempts_total[5m]) * 100"
          }
        ]
      }
    ]
  }
}
```

### Log Aggregation

```yaml
# fluentd/fluent.conf
<source>
  @type forward
  port 24224
  bind 0.0.0.0
</source>

<match whatsapp.engine.**>
  @type elasticsearch
  host elasticsearch
  port 9200
  index_name whatsapp-engine
  type_name _doc
</match>
```

## 🔄 CI/CD Pipeline

### GitHub Actions

```yaml
# .github/workflows/deploy.yml
name: Deploy to Production

on:
  push:
    branches: [main]

jobs:
  build-and-deploy:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Build Docker image
      run: |
        docker build -f docker/Dockerfile.production -t whatsapp-engine:${{ github.sha }} .
        docker tag whatsapp-engine:${{ github.sha }} whatsapp-engine:latest
    
    - name: Deploy to staging
      run: |
        docker-compose -f docker/docker-compose.staging.yml up -d
        
    - name: Run tests
      run: |
        cargo test
        ./scripts/test-production.sh
        
    - name: Deploy to production
      if: success()
      run: |
        ./scripts/deploy-production.sh
```

### GitLab CI

```yaml
# .gitlab-ci.yml
stages:
  - build
  - test
  - deploy

build:
  stage: build
  script:
    - docker build -f docker/Dockerfile.production -t $CI_REGISTRY_IMAGE:$CI_COMMIT_SHA .
    - docker push $CI_REGISTRY_IMAGE:$CI_COMMIT_SHA

test:
  stage: test
  script:
    - cargo test
    - ./scripts/test-production.sh

deploy_production:
  stage: deploy
  script:
    - kubectl set image deployment/whatsapp-engine whatsapp-engine=$CI_REGISTRY_IMAGE:$CI_COMMIT_SHA
  only:
    - main
```

## 🔧 Scaling & Performance

### Horizontal Scaling

```bash
# Docker Swarm
docker service scale whatsapp-engine=5

# Kubernetes
kubectl scale deployment whatsapp-engine --replicas=5

# Auto-scaling (Kubernetes)
kubectl autoscale deployment whatsapp-engine --cpu-percent=70 --min=2 --max=10
```

### Database Optimization

```sql
-- PostgreSQL optimizations
CREATE INDEX CONCURRENTLY idx_messages_timestamp ON messages(created_at);
CREATE INDEX CONCURRENTLY idx_sessions_phone ON sessions(phone_number);

-- Connection pooling
ALTER SYSTEM SET max_connections = 200;
ALTER SYSTEM SET shared_buffers = '256MB';
```

### Redis Clustering

```yaml
# Redis cluster configuration
services:
  redis-1:
    image: redis:7-alpine
    command: redis-server --cluster-enabled yes --cluster-config-file nodes.conf
  redis-2:
    image: redis:7-alpine
    command: redis-server --cluster-enabled yes --cluster-config-file nodes.conf
  redis-3:
    image: redis:7-alpine
    command: redis-server --cluster-enabled yes --cluster-config-file nodes.conf
```

## 🛠️ Troubleshooting

### Common Issues

1. **Browser Launch Failures**:
```bash
# Install Chrome dependencies
apt-get update && apt-get install -y \
  chromium-browser \
  libnss3 \
  libxss1 \
  libasound2
```

2. **Memory Issues**:
```bash
# Increase system limits
echo 'vm.max_map_count=262144' >> /etc/sysctl.conf
sysctl -p
```

3. **Network Connectivity**:
```bash
# Test WhatsApp Web connectivity
curl -I https://web.whatsapp.com

# Check DNS resolution
nslookup web.whatsapp.com
```

### Health Checks

```bash
#!/bin/bash
# health-check.sh

# API health
if ! curl -f http://localhost:3000/health > /dev/null 2>&1; then
  echo "API health check failed"
  exit 1
fi

# Database connectivity
if ! pg_isready -h localhost -p 5432 > /dev/null 2>&1; then
  echo "Database health check failed"
  exit 1
fi

# Redis connectivity
if ! redis-cli -h localhost ping > /dev/null 2>&1; then
  echo "Redis health check failed"
  exit 1
fi

echo "All health checks passed"
```

---

## 📋 Post-Deployment Checklist

- [ ] **Service Health**
  - [ ] All containers/services running
  - [ ] Health endpoints responding
  - [ ] Logs showing no errors

- [ ] **Security**
  - [ ] HTTPS properly configured
  - [ ] API tokens secure and working
  - [ ] Firewall rules applied
  - [ ] Security headers present

- [ ] **Monitoring**
  - [ ] Metrics collection working
  - [ ] Dashboards accessible
  - [ ] Alerts configured
  - [ ] Log aggregation functioning

- [ ] **Performance**
  - [ ] Response times acceptable
  - [ ] Resource usage within limits
  - [ ] Auto-scaling configured (if applicable)

- [ ] **Backup & Recovery**
  - [ ] Database backups scheduled
  - [ ] Session data backed up
  - [ ] Recovery procedures tested

This deployment guide ensures reliable, secure, and scalable WhatsApp Engine deployments across various environments.
