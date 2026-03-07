# Deployment Guide

## Docker

### Quick Start
```bash
docker build -t was:latest .
docker run -d --name was -p 3000:3000 \
  -v was-data:/data \
  -e WAS__AUTH__SECRET_KEY="your-secret" \
  --shm-size=2g \
  was:latest
```

### Docker Compose
```yaml
version: '3.8'
services:
  was:
    build: .
    restart: unless-stopped
    ports:
      - "3000:3000"
    volumes:
      - was-data:/data
    environment:
      - WAS__AUTH__SECRET_KEY=${WAS_SECRET_KEY}
    shm_size: 2g
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/api/health"]
      interval: 30s
      timeout: 10s
      retries: 3

volumes:
  was-data:
```

## Kubernetes

### Deployment
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: was
spec:
  replicas: 1
  selector:
    matchLabels:
      app: was
  template:
    spec:
      containers:
      - name: was
        image: your-registry/was:latest
        ports:
        - containerPort: 3000
        env:
        - name: WAS__AUTH__SECRET_KEY
          valueFrom:
            secretKeyRef:
              name: was-secrets
              key: secret-key
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
        volumeMounts:
        - name: dshm
          mountPath: /dev/shm
        livenessProbe:
          httpGet:
            path: /api/live
            port: 3000
        readinessProbe:
          httpGet:
            path: /api/ready
            port: 3000
      volumes:
      - name: dshm
        emptyDir:
          medium: Memory
          sizeLimit: 2Gi
```

## Systemd

Create `/etc/systemd/system/was.service`:
```ini
[Unit]
Description=WAS WhatsApp Server
After=network.target

[Service]
Type=simple
User=was
WorkingDirectory=/opt/was
ExecStart=/opt/was/was
Restart=always
Environment=RUST_LOG=info
Environment=WAS__AUTH__SECRET_KEY=your-secret

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable was
sudo systemctl start was
```

## Nginx Reverse Proxy

```nginx
upstream was {
    server 127.0.0.1:3000;
}

server {
    listen 443 ssl http2;
    server_name was.yourdomain.com;

    ssl_certificate /etc/ssl/certs/was.crt;
    ssl_certificate_key /etc/ssl/private/was.key;

    location / {
        proxy_pass http://was;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header Connection '';
        proxy_buffering off;
    }
}
```

## Resource Estimation

| Instances | RAM | CPU |
|-----------|-----|-----|
| 1-5 | 2GB | 1 core |
| 5-20 | 4GB | 2 cores |
| 20-50 | 8GB | 4 cores |

## Security Checklist

- [ ] Change default secret key
- [ ] Use HTTPS
- [ ] Restrict CORS origins
- [ ] Disable Swagger in production
- [ ] Configure firewall
- [ ] Enable rate limiting
