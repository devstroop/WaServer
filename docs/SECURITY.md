# Security Guide 🔒

Security considerations and best practices for WhatsApp Engine deployment and usage.

## 🛡️ Security Overview

WhatsApp Engine handles sensitive messaging data and authentication credentials. This guide outlines essential security measures for production deployment.

## 🔐 Authentication & Authorization

### API Token Security

**Generate Strong Tokens**:
```bash
# Generate a secure random token
openssl rand -hex 32
# or
uuidgen | tr -d '-' | tr '[:upper:]' '[:lower:]'
```

**Token Storage**:
- ✅ Store in environment variables, not configuration files
- ✅ Use secrets management systems (HashiCorp Vault, AWS Secrets Manager)
- ❌ Never commit tokens to version control
- ❌ Never log tokens in application logs

**Environment Configuration**:
```bash
# Production environment
export AUTH_API_TOKEN="$(cat /run/secrets/whatsapp_api_token)"
export DATABASE_PASSWORD="$(cat /run/secrets/db_password)"
```

**Docker Secrets**:
```yaml
# docker-compose.yml
services:
  whatsapp-engine:
    secrets:
      - whatsapp_api_token
    environment:
      - AUTH_API_TOKEN_FILE=/run/secrets/whatsapp_api_token

secrets:
  whatsapp_api_token:
    external: true
```

### Token Rotation

Implement regular token rotation:

```bash
#!/bin/bash
# rotate-tokens.sh
NEW_TOKEN=$(openssl rand -hex 32)
kubectl create secret generic whatsapp-api-token \
  --from-literal=token=$NEW_TOKEN \
  --dry-run=client -o yaml | kubectl apply -f -
kubectl rollout restart deployment whatsapp-engine
```

## 🌐 Network Security

### HTTPS/TLS Configuration

**Nginx SSL Termination**:
```nginx
server {
    listen 443 ssl http2;
    server_name whatsapp.yourdomain.com;
    
    ssl_certificate /etc/ssl/certs/whatsapp.crt;
    ssl_certificate_key /etc/ssl/private/whatsapp.key;
    
    # Modern SSL configuration
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512;
    ssl_prefer_server_ciphers off;
    
    # Security headers
    add_header Strict-Transport-Security "max-age=63072000" always;
    add_header X-Frame-Options DENY always;
    add_header X-Content-Type-Options nosniff always;
    add_header Referrer-Policy no-referrer always;
    
    location / {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### Firewall Configuration

**iptables Rules**:
```bash
# Allow only necessary ports
iptables -A INPUT -p tcp --dport 22 -j ACCEPT    # SSH
iptables -A INPUT -p tcp --dport 80 -j ACCEPT    # HTTP (redirect)
iptables -A INPUT -p tcp --dport 443 -j ACCEPT   # HTTPS
iptables -A INPUT -j DROP                        # Drop all others
```

**UFW (Ubuntu)**:
```bash
ufw allow ssh
ufw allow 80/tcp
ufw allow 443/tcp
ufw --force enable
```

## 🚫 Input Validation & Sanitization

### Phone Number Validation

```rust
use regex::Regex;

fn validate_phone_number(phone: &str) -> Result<String, ValidationError> {
    // International format: +[country code][number]
    let phone_regex = Regex::new(r"^\+[1-9]\d{1,14}$")?;
    
    if !phone_regex.is_match(phone) {
        return Err(ValidationError::InvalidFormat(
            "Phone number must be in international format (+1234567890)".into()
        ));
    }
    
    // Additional country-specific validation
    validate_country_specific(phone)?;
    
    Ok(phone.to_string())
}
```

### Message Content Validation

```rust
fn validate_message_content(content: &str) -> Result<String, ValidationError> {
    // Length limits
    if content.is_empty() {
        return Err(ValidationError::EmptyContent);
    }
    
    if content.len() > 4096 {
        return Err(ValidationError::ContentTooLong);
    }
    
    // Sanitize content
    let sanitized = html_escape::encode_text(content);
    
    // Check for suspicious patterns
    if contains_malicious_patterns(&sanitized) {
        return Err(ValidationError::SuspiciousContent);
    }
    
    Ok(sanitized.to_string())
}
```

### File Upload Security

```rust
use std::path::Path;

struct FileValidator {
    max_size: usize,
    allowed_extensions: Vec<String>,
    allowed_mime_types: Vec<String>,
}

impl FileValidator {
    fn validate_file(&self, file_path: &Path, content: &[u8]) -> Result<(), ValidationError> {
        // Size check
        if content.len() > self.max_size {
            return Err(ValidationError::FileTooLarge);
        }
        
        // Extension check
        let extension = file_path.extension()
            .and_then(|ext| ext.to_str())
            .ok_or(ValidationError::InvalidExtension)?;
            
        if !self.allowed_extensions.contains(&extension.to_lowercase()) {
            return Err(ValidationError::DisallowedExtension);
        }
        
        // MIME type validation
        let mime_type = infer::get(content)
            .map(|kind| kind.mime_type())
            .ok_or(ValidationError::UnknownFileType)?;
            
        if !self.allowed_mime_types.contains(&mime_type.to_string()) {
            return Err(ValidationError::DisallowedMimeType);
        }
        
        // Scan for malware (integrate with ClamAV or similar)
        scan_for_malware(content)?;
        
        Ok(())
    }
}
```

## 🚧 Rate Limiting & DDoS Protection

### Application-Level Rate Limiting

```rust
use tower_http::limit::RequestBodyLimitLayer;
use tower_governor::{GovernorLayer, governor::GovernorConfig};

// Rate limiting configuration
let governor_conf = Box::new(
    GovernorConfig::default()
        .per_second(2)
        .burst_size(5)
);

let app = Router::new()
    .route("/api/chat/send", post(send_message))
    .layer(GovernorLayer {
        config: Box::leak(governor_conf),
    })
    .layer(RequestBodyLimitLayer::new(16 * 1024 * 1024)); // 16MB limit
```

### Nginx Rate Limiting

```nginx
# Rate limiting zones
limit_req_zone $binary_remote_addr zone=api:10m rate=10r/m;
limit_req_zone $binary_remote_addr zone=auth:10m rate=5r/m;

server {
    location /api/auth/ {
        limit_req zone=auth burst=3 nodelay;
        proxy_pass http://backend;
    }
    
    location /api/ {
        limit_req zone=api burst=10 nodelay;
        proxy_pass http://backend;
    }
}
```

### Fail2Ban Configuration

```ini
# /etc/fail2ban/jail.local
[whatsapp-engine]
enabled = true
port = 80,443
filter = whatsapp-engine
logpath = /var/log/nginx/access.log
maxretry = 5
bantime = 3600
findtime = 600
```

```ini
# /etc/fail2ban/filter.d/whatsapp-engine.conf
[Definition]
failregex = ^<HOST> - - \[.*\] "(POST|GET) /api/.* HTTP/.*" (401|403|429) .*$
ignoreregex =
```

## 📊 Security Monitoring & Logging

### Structured Logging

```rust
use tracing::{info, warn, error, instrument};
use serde_json::json;

#[instrument(skip(token))]
async fn authenticate_request(token: &str) -> Result<User, AuthError> {
    let start = std::time::Instant::now();
    
    match validate_token(token).await {
        Ok(user) => {
            info!(
                user_id = %user.id,
                duration_ms = start.elapsed().as_millis(),
                "Authentication successful"
            );
            Ok(user)
        }
        Err(e) => {
            warn!(
                error = %e,
                duration_ms = start.elapsed().as_millis(),
                "Authentication failed"
            );
            Err(e)
        }
    }
}
```

### Security Event Logging

```rust
#[derive(Serialize)]
struct SecurityEvent {
    event_type: String,
    severity: String,
    source_ip: String,
    user_agent: String,
    endpoint: String,
    details: serde_json::Value,
    timestamp: DateTime<Utc>,
}

fn log_security_event(event: SecurityEvent) {
    error!(
        target: "security",
        event_type = %event.event_type,
        severity = %event.severity,
        source_ip = %event.source_ip,
        "{}", serde_json::to_string(&event).unwrap()
    );
}
```

### Alerting Rules

```yaml
# Prometheus alerting rules
groups:
  - name: whatsapp-engine-security
    rules:
      - alert: HighAuthFailureRate
        expr: rate(whatsapp_auth_failures_total[5m]) > 0.1
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "High authentication failure rate detected"
          
      - alert: SuspiciousFileUpload
        expr: increase(whatsapp_malicious_files_total[1h]) > 0
        for: 0m
        labels:
          severity: critical
        annotations:
          summary: "Malicious file upload detected"
```

## 🐳 Container Security

### Secure Docker Image

```dockerfile
# Use non-root user
FROM rust:1.70-slim-bullseye as builder
RUN groupadd -r whatsapp && useradd -r -g whatsapp whatsapp

# Build stage
WORKDIR /app
COPY . .
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim
RUN groupadd -r whatsapp && useradd -r -g whatsapp whatsapp

# Install security updates
RUN apt-get update && apt-get upgrade -y && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy binary and set ownership
COPY --from=builder /app/target/release/whatsapp-server /usr/local/bin/
RUN chown root:root /usr/local/bin/whatsapp-server && \
    chmod 755 /usr/local/bin/whatsapp-server

# Run as non-root user
USER whatsapp
EXPOSE 3000

CMD ["/usr/local/bin/whatsapp-server"]
```

### Security Scanning

```bash
# Dockerfile linting
hadolint Dockerfile

# Vulnerability scanning
docker run --rm -v /var/run/docker.sock:/var/run/docker.sock \
  -v $(pwd):/src aquasec/trivy image whatsapp-engine:latest

# Container runtime security
docker run --rm -it --pid container:whatsapp-engine \
  falcosecurity/falco
```

## ☁️ Infrastructure Security

### Kubernetes Security

```yaml
# SecurityContext
apiVersion: apps/v1
kind: Deployment
metadata:
  name: whatsapp-engine
spec:
  template:
    spec:
      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
        fsGroup: 1000
      containers:
      - name: whatsapp-engine
        securityContext:
          allowPrivilegeEscalation: false
          readOnlyRootFilesystem: true
          capabilities:
            drop:
            - ALL
        resources:
          limits:
            memory: "512Mi"
            cpu: "500m"
          requests:
            memory: "256Mi"
            cpu: "250m"
```

### Network Policies

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: whatsapp-engine-netpol
spec:
  podSelector:
    matchLabels:
      app: whatsapp-engine
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: ingress-nginx
    ports:
    - protocol: TCP
      port: 3000
  egress:
  - to: []
    ports:
    - protocol: TCP
      port: 443  # HTTPS only
```

## 📋 Security Checklist

### Pre-deployment Security Checklist

- [ ] **Authentication**
  - [ ] Strong API tokens generated and stored securely
  - [ ] Token rotation mechanism implemented
  - [ ] No credentials in source code or logs

- [ ] **Network Security**
  - [ ] HTTPS/TLS properly configured
  - [ ] Security headers implemented
  - [ ] Firewall rules configured
  - [ ] VPN/private networks used where applicable

- [ ] **Input Validation**
  - [ ] All inputs validated and sanitized
  - [ ] File upload restrictions enforced
  - [ ] SQL injection prevention (if using database)
  - [ ] XSS prevention measures

- [ ] **Rate Limiting**
  - [ ] Application-level rate limiting
  - [ ] Infrastructure-level DDoS protection
  - [ ] Fail2Ban or similar intrusion prevention

- [ ] **Monitoring & Logging**
  - [ ] Security event logging implemented
  - [ ] Log aggregation and analysis setup
  - [ ] Alerting rules configured
  - [ ] Incident response procedures documented

- [ ] **Container/Infrastructure**
  - [ ] Non-root containers
  - [ ] Security scanning integrated
  - [ ] Resource limits configured
  - [ ] Network policies implemented

### Regular Security Tasks

- [ ] **Weekly**
  - [ ] Review security logs and alerts
  - [ ] Check for failed authentication attempts
  - [ ] Monitor rate limiting effectiveness

- [ ] **Monthly**
  - [ ] Rotate API tokens
  - [ ] Update dependencies and security patches
  - [ ] Review and update firewall rules
  - [ ] Conduct security scans

- [ ] **Quarterly**
  - [ ] Security assessment and penetration testing
  - [ ] Review and update security policies
  - [ ] Incident response plan testing
  - [ ] Security training for team members

## 🚨 Incident Response

### Security Incident Types

1. **Authentication Breach**
   - Immediately rotate all API tokens
   - Review access logs for suspicious activity
   - Notify affected users

2. **DDoS Attack**
   - Activate DDoS protection
   - Scale infrastructure if needed
   - Block malicious IP ranges

3. **Malicious File Upload**
   - Quarantine and analyze the file
   - Check for system compromise
   - Update file validation rules

4. **Data Exfiltration**
   - Isolate affected systems
   - Assess scope of data access
   - Notify relevant authorities if required

### Emergency Contacts

```bash
# Emergency shutdown
kubectl scale deployment whatsapp-engine --replicas=0

# Block all traffic
iptables -A INPUT -j DROP

# Enable maintenance mode
kubectl apply -f maintenance-mode.yaml
```

## 📚 Security Resources

- **OWASP Top 10**: https://owasp.org/www-project-top-ten/
- **Rust Security Guidelines**: https://anssi-fr.github.io/rust-guide/
- **Container Security**: https://kubernetes.io/docs/concepts/security/
- **API Security**: https://owasp.org/www-project-api-security/

---

**⚠️ Security is an ongoing process. Regularly review and update security measures based on evolving threats and best practices.**
