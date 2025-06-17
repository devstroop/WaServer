# Monitoring and Observability Guide 📊

This document provides comprehensive guidance for monitoring, observability, and operational insights for WhatsApp Engine Rust in production environments.

## 📋 Table of Contents

- [Overview](#-overview)
- [Metrics and Monitoring](#-metrics-and-monitoring)
- [Logging Strategy](#-logging-strategy)
- [Tracing and Distributed Tracing](#-tracing-and-distributed-tracing)
- [Health Checks](#-health-checks)
- [Alerting](#-alerting)
- [Performance Monitoring](#-performance-monitoring)
- [Error Tracking](#-error-tracking)
- [Dashboards](#-dashboards)
- [Troubleshooting Playbooks](#-troubleshooting-playbooks)
- [Tool Integration](#-tool-integration)

## 🎯 Overview

### Observability Pillars

1. **Metrics** - Quantitative measurements of system behavior
2. **Logs** - Timestamped records of discrete events
3. **Traces** - Request flows through distributed systems
4. **Health** - System availability and readiness

### Key Monitoring Goals

- **Availability**: Is the service up and responding?
- **Performance**: How fast is the service responding?
- **Error Rate**: What percentage of requests are failing?
- **Throughput**: How many operations per second?
- **Resource Usage**: CPU, memory, disk, network utilization
- **Business Metrics**: Messages sent, authentication success rate

## 📈 Metrics and Monitoring

### Core Metrics to Track

#### System Metrics

```rust
// src/utils/metrics.rs
use prometheus::{Counter, Histogram, Gauge, Registry};
use std::sync::Arc;

pub struct WhatsAppMetrics {
    // Request metrics
    pub http_requests_total: Counter,
    pub http_request_duration: Histogram,
    pub http_requests_in_flight: Gauge,
    
    // Business metrics
    pub messages_sent_total: Counter,
    pub authentication_attempts_total: Counter,
    pub authentication_success_total: Counter,
    
    // Browser metrics
    pub browser_connections_active: Gauge,
    pub browser_navigation_duration: Histogram,
    pub browser_errors_total: Counter,
    
    // System metrics
    pub memory_usage_bytes: Gauge,
    pub cpu_usage_percent: Gauge,
    pub open_file_descriptors: Gauge,
}

impl WhatsAppMetrics {
    pub fn new() -> Self {
        Self {
            http_requests_total: Counter::new(
                "whatsapp_http_requests_total",
                "Total number of HTTP requests"
            ).unwrap(),
            
            http_request_duration: Histogram::new(
                "whatsapp_http_request_duration_seconds",
                "HTTP request duration in seconds"
            ).unwrap(),
            
            messages_sent_total: Counter::new(
                "whatsapp_messages_sent_total",
                "Total number of messages sent"
            ).unwrap(),
            
            authentication_attempts_total: Counter::new(
                "whatsapp_auth_attempts_total",
                "Total authentication attempts"
            ).unwrap(),
            
            browser_connections_active: Gauge::new(
                "whatsapp_browser_connections_active",
                "Number of active browser connections"
            ).unwrap(),
            
            // ... other metrics
        }
    }
    
    pub fn register_all(&self, registry: &Registry) -> Result<(), prometheus::Error> {
        registry.register(Box::new(self.http_requests_total.clone()))?;
        registry.register(Box::new(self.http_request_duration.clone()))?;
        registry.register(Box::new(self.messages_sent_total.clone()))?;
        registry.register(Box::new(self.authentication_attempts_total.clone()))?;
        registry.register(Box::new(self.browser_connections_active.clone()))?;
        Ok(())
    }
}
```

#### Custom Metrics Collection

```rust
// src/middleware/metrics.rs
use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;

pub async fn metrics_middleware(
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let start = Instant::now();
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    
    // Increment request counter
    METRICS.http_requests_total
        .with_label_values(&[method.as_str(), &path])
        .inc();
    
    // Track in-flight requests
    METRICS.http_requests_in_flight.inc();
    
    let response = next.run(request).await?;
    
    // Record duration
    let duration = start.elapsed().as_secs_f64();
    METRICS.http_request_duration
        .with_label_values(&[method.as_str(), &path, response.status().as_str()])
        .observe(duration);
    
    // Decrement in-flight requests
    METRICS.http_requests_in_flight.dec();
    
    Ok(response)
}
```

### Prometheus Integration

#### Configuration

```toml
# config/app.toml
[monitoring]
enabled = true
metrics_port = 9090
metrics_path = "/metrics"

[prometheus]
job_name = "whatsapp-engine"
scrape_interval = "15s"
```

#### Metrics Endpoint

```rust
// src/handlers/metrics.rs
use axum::{response::Response, http::StatusCode};
use prometheus::{Encoder, TextEncoder, Registry};

pub async fn metrics_handler() -> Result<Response<String>, AppError> {
    let registry = prometheus::default_registry();
    let encoder = TextEncoder::new();
    
    let metric_families = registry.gather();
    let mut buffer = Vec::new();
    
    encoder.encode(&metric_families, &mut buffer)
        .map_err(|e| AppError::Internal(format!("Failed to encode metrics: {}", e)))?;
    
    let response = String::from_utf8(buffer)
        .map_err(|e| AppError::Internal(format!("Failed to convert metrics to string: {}", e)))?;
    
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain; version=0.0.4")
        .body(response)
        .unwrap())
}
```

#### Prometheus Configuration

```yaml
# docker/prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "whatsapp_rules.yml"

scrape_configs:
  - job_name: 'whatsapp-engine'
    static_configs:
      - targets: ['whatsapp-engine:9090']
    scrape_interval: 15s
    metrics_path: /metrics
    
  - job_name: 'node-exporter'
    static_configs:
      - targets: ['node-exporter:9100']

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093
```

## 📝 Logging Strategy

### Structured Logging

```rust
use tracing::{info, warn, error, debug, span, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use serde_json::json;

pub fn init_logging(config: &LoggingConfig) -> Result<()> {
    let json_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_current_span(false)
        .with_span_list(true);
    
    let filter_layer = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&config.level));
    
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(json_layer)
        .init();
    
    Ok(())
}

// Usage in services
impl AuthService {
    pub async fn authenticate_phone(&self, phone: &str) -> Result<PhoneAuthResult> {
        let span = span!(Level::INFO, "authenticate_phone", phone = %phone);
        let _enter = span.enter();
        
        info!("Starting phone authentication");
        
        match self.perform_phone_auth(phone).await {
            Ok(result) => {
                info!(
                    success = result.success,
                    "Phone authentication completed"
                );
                Ok(result)
            }
            Err(e) => {
                error!(
                    error = %e,
                    "Phone authentication failed"
                );
                Err(e)
            }
        }
    }
}
```

### Log Levels and Content

```rust
// Different log levels for different scenarios
use tracing::{trace, debug, info, warn, error};

// TRACE: Very detailed, typically only enabled in development
trace!("Browser element located: {}", selector);

// DEBUG: Detailed information for debugging
debug!(
    browser_pid = process_id,
    url = %current_url,
    "Browser navigation completed"
);

// INFO: General information about system operation
info!(
    user_id = %user_id,
    message_count = message_count,
    "Messages sent successfully"
);

// WARN: Something unexpected happened but service continues
warn!(
    retry_count = retry_count,
    max_retries = max_retries,
    "Retrying failed operation"
);

// ERROR: Error condition occurred
error!(
    error = %error,
    phone = %phone_number,
    operation = "send_message",
    "Critical operation failed"
);
```

### Log Aggregation

#### Fluentd Configuration

```yaml
# docker/fluentd/fluent.conf
<source>
  @type forward
  port 24224
  bind 0.0.0.0
</source>

<filter whatsapp.**>
  @type parser
  key_name log
  <parse>
    @type json
  </parse>
</filter>

<match whatsapp.**>
  @type elasticsearch
  host elasticsearch
  port 9200
  index_name whatsapp-logs
  <buffer>
    @type file
    path /fluentd/log/whatsapp
    flush_mode interval
    flush_interval 30s
  </buffer>
</match>
```

#### Docker Logging Driver

```yaml
# docker-compose.yml
services:
  whatsapp-engine:
    image: whatsapp-engine:latest
    logging:
      driver: fluentd
      options:
        fluentd-address: localhost:24224
        tag: whatsapp.engine
```

## 🔍 Tracing and Distributed Tracing

### OpenTelemetry Integration

```rust
use opentelemetry::{global, sdk::trace as sdktrace, trace::TraceError};
use opentelemetry_jaeger::JaegerPipeline;
use tracing_opentelemetry::OpenTelemetryLayer;

pub fn init_tracing(config: &TracingConfig) -> Result<(), TraceError> {
    let tracer = JaegerPipeline::new()
        .with_service_name("whatsapp-engine")
        .with_agent_endpoint(&config.jaeger_endpoint)
        .install_batch(sdktrace::runtime::Tokio)?;
    
    let telemetry_layer = OpenTelemetryLayer::new(tracer);
    
    tracing_subscriber::registry()
        .with(telemetry_layer)
        .with(tracing_subscriber::EnvFilter::new("info"))
        .init();
    
    Ok(())
}
```

### Custom Spans

```rust
use tracing::{instrument, Span};

impl WhatsAppEngine {
    #[instrument(skip(self), fields(phone = %phone, message_len = message.len()))]
    pub async fn send_message(&self, phone: &str, message: &str) -> Result<SendMessageResult> {
        let current_span = Span::current();
        current_span.record("operation", "send_message");
        
        // Add custom attributes
        current_span.record("user_agent", "WhatsAppEngine/1.0");
        current_span.record("retry_count", 0);
        
        let result = self.chat_service.send_message(phone, message).await?;
        
        current_span.record("success", result.success);
        if let Some(msg_id) = &result.message_id {
            current_span.record("message_id", msg_id);
        }
        
        Ok(result)
    }
}
```

## 🏥 Health Checks

### Health Check Endpoints

```rust
// src/handlers/health.rs
use axum::{Json, response::Json as ResponseJson};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct HealthCheck {
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub version: String,
    pub checks: std::collections::HashMap<String, ComponentHealth>,
}

#[derive(Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: String,
    pub message: Option<String>,
    pub duration_ms: u64,
}

pub async fn health_check(
    State(engine): State<Arc<WhatsAppEngine>>,
) -> Result<ResponseJson<HealthCheck>, AppError> {
    let start = std::time::Instant::now();
    let mut checks = std::collections::HashMap::new();
    
    // Check browser connection
    let browser_check = check_browser_health(&engine).await;
    checks.insert("browser".to_string(), browser_check);
    
    // Check authentication
    let auth_check = check_auth_health(&engine).await;
    checks.insert("authentication".to_string(), auth_check);
    
    // Check memory usage
    let memory_check = check_memory_health().await;
    checks.insert("memory".to_string(), memory_check);
    
    // Check disk space
    let disk_check = check_disk_health().await;
    checks.insert("disk".to_string(), disk_check);
    
    let overall_status = if checks.values().all(|c| c.status == "healthy") {
        "healthy"
    } else if checks.values().any(|c| c.status == "critical") {
        "critical"
    } else {
        "degraded"
    };
    
    let health = HealthCheck {
        status: overall_status.to_string(),
        timestamp: chrono::Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        checks,
    };
    
    Ok(ResponseJson(health))
}

async fn check_browser_health(engine: &WhatsAppEngine) -> ComponentHealth {
    let start = std::time::Instant::now();
    
    match engine.get_browser_status().await {
        Ok(status) if status.connected => ComponentHealth {
            status: "healthy".to_string(),
            message: Some("Browser connected and responsive".to_string()),
            duration_ms: start.elapsed().as_millis() as u64,
        },
        Ok(_) => ComponentHealth {
            status: "degraded".to_string(),
            message: Some("Browser not connected".to_string()),
            duration_ms: start.elapsed().as_millis() as u64,
        },
        Err(e) => ComponentHealth {
            status: "critical".to_string(),
            message: Some(format!("Browser check failed: {}", e)),
            duration_ms: start.elapsed().as_millis() as u64,
        },
    }
}
```

### Kubernetes Health Checks

```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: whatsapp-engine
spec:
  template:
    spec:
      containers:
      - name: whatsapp-engine
        image: whatsapp-engine:latest
        ports:
        - containerPort: 3000
        - containerPort: 9090  # metrics port
        
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
        
        readinessProbe:
          httpGet:
            path: /health/ready
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 2
        
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "500m"
```

## 🚨 Alerting

### Alert Rules

```yaml
# docker/prometheus/whatsapp_rules.yml
groups:
  - name: whatsapp-engine.rules
    rules:
    
    # High error rate
    - alert: HighErrorRate
      expr: |
        (
          rate(whatsapp_http_requests_total{status=~"5.."}[5m]) /
          rate(whatsapp_http_requests_total[5m])
        ) > 0.1
      for: 2m
      labels:
        severity: warning
      annotations:
        summary: "High error rate detected"
        description: "Error rate is {{ $value | humanizePercentage }} for the last 5 minutes"
    
    # Authentication failures
    - alert: AuthenticationFailures
      expr: |
        (
          rate(whatsapp_auth_attempts_total[5m]) -
          rate(whatsapp_auth_success_total[5m])
        ) > 5
      for: 1m
      labels:
        severity: critical
      annotations:
        summary: "High authentication failure rate"
        description: "{{ $value }} authentication failures per second in the last 5 minutes"
    
    # Browser connection issues
    - alert: BrowserConnectionDown
      expr: whatsapp_browser_connections_active == 0
      for: 30s
      labels:
        severity: critical
      annotations:
        summary: "No active browser connections"
        description: "All browser connections are down"
    
    # High memory usage
    - alert: HighMemoryUsage
      expr: whatsapp_memory_usage_bytes > 1073741824  # 1GB
      for: 5m
      labels:
        severity: warning
      annotations:
        summary: "High memory usage"
        description: "Memory usage is {{ $value | humanizeBytes }}"
    
    # Service down
    - alert: ServiceDown
      expr: up{job="whatsapp-engine"} == 0
      for: 1m
      labels:
        severity: critical
      annotations:
        summary: "WhatsApp Engine service is down"
        description: "Service has been down for more than 1 minute"
```

### AlertManager Configuration

```yaml
# docker/alertmanager/alertmanager.yml
global:
  smtp_smarthost: 'localhost:587'
  smtp_from: 'alerts@yourcompany.com'

route:
  group_by: ['alertname']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 1h
  receiver: 'web.hook'

receivers:
- name: 'web.hook'
  email_configs:
  - to: 'ops@yourcompany.com'
    subject: 'WhatsApp Engine Alert: {{ .GroupLabels.alertname }}'
    body: |
      {{ range .Alerts }}
      Alert: {{ .Annotations.summary }}
      Description: {{ .Annotations.description }}
      {{ end }}
  
  slack_configs:
  - api_url: 'YOUR_SLACK_WEBHOOK_URL'
    channel: '#alerts'
    title: 'WhatsApp Engine Alert'
    text: |
      {{ range .Alerts }}
      *{{ .Annotations.summary }}*
      {{ .Annotations.description }}
      {{ end }}

inhibit_rules:
  - source_match:
      severity: 'critical'
    target_match:
      severity: 'warning'
    equal: ['alertname', 'instance']
```

## ⚡ Performance Monitoring

### Application Performance Monitoring (APM)

```rust
// src/utils/performance.rs
use std::time::{Duration, Instant};
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct PerformanceTracker {
    metrics: RwLock<HashMap<String, PerformanceMetric>>,
}

#[derive(Debug, Clone)]
pub struct PerformanceMetric {
    pub count: u64,
    pub total_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub avg_duration: Duration,
}

impl PerformanceTracker {
    pub fn new() -> Self {
        Self {
            metrics: RwLock::new(HashMap::new()),
        }
    }
    
    pub async fn track<F, T>(&self, operation: &str, future: F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        let start = Instant::now();
        let result = future.await;
        let duration = start.elapsed();
        
        self.record_performance(operation, duration).await;
        result
    }
    
    async fn record_performance(&self, operation: &str, duration: Duration) {
        let mut metrics = self.metrics.write().await;
        
        let metric = metrics.entry(operation.to_string()).or_insert(PerformanceMetric {
            count: 0,
            total_duration: Duration::ZERO,
            min_duration: duration,
            max_duration: duration,
            avg_duration: Duration::ZERO,
        });
        
        metric.count += 1;
        metric.total_duration += duration;
        metric.min_duration = metric.min_duration.min(duration);
        metric.max_duration = metric.max_duration.max(duration);
        metric.avg_duration = metric.total_duration / metric.count as u32;
    }
    
    pub async fn get_metrics(&self) -> HashMap<String, PerformanceMetric> {
        self.metrics.read().await.clone()
    }
}
```

### Resource Monitoring

```rust
// src/utils/system_metrics.rs
use sysinfo::{System, SystemExt, ProcessExt};

pub struct SystemMetrics {
    system: System,
}

impl SystemMetrics {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        Self { system }
    }
    
    pub fn collect_metrics(&mut self) -> SystemSnapshot {
        self.system.refresh_all();
        
        let process = self.system.processes()
            .values()
            .find(|p| p.name() == "whatsapp-server")
            .cloned();
        
        SystemSnapshot {
            cpu_usage: self.system.global_cpu_info().cpu_usage(),
            memory_total: self.system.total_memory(),
            memory_used: self.system.used_memory(),
            memory_available: self.system.available_memory(),
            process_memory: process.as_ref().map(|p| p.memory()),
            process_cpu: process.as_ref().map(|p| p.cpu_usage()),
            load_average: self.system.load_average(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SystemSnapshot {
    pub cpu_usage: f32,
    pub memory_total: u64,
    pub memory_used: u64,
    pub memory_available: u64,
    pub process_memory: Option<u64>,
    pub process_cpu: Option<f32>,
    pub load_average: sysinfo::LoadAvg,
}
```

## 🐛 Error Tracking

### Error Aggregation

```rust
// src/utils/error_tracking.rs
use sentry::{configure_scope, capture_exception, add_breadcrumb};
use std::sync::Arc;

pub fn init_sentry(dsn: &str) -> Result<(), Box<dyn std::error::Error>> {
    let _guard = sentry::init((dsn, sentry::ClientOptions {
        release: sentry::release_name!(),
        environment: Some(std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".into()).into()),
        ..Default::default()
    }));
    
    configure_scope(|scope| {
        scope.set_tag("service", "whatsapp-engine");
        scope.set_tag("version", env!("CARGO_PKG_VERSION"));
    });
    
    Ok(())
}

pub fn track_error(error: &WhatsAppError, context: Option<&str>) {
    configure_scope(|scope| {
        if let Some(ctx) = context {
            scope.set_context("operation", sentry::protocol::Context::Other({
                let mut map = std::collections::BTreeMap::new();
                map.insert("context".to_string(), ctx.into());
                map
            }));
        }
        
        // Add breadcrumb
        add_breadcrumb(sentry::Breadcrumb {
            message: Some(format!("Error occurred: {}", error)),
            level: sentry::Level::Error,
            ..Default::default()
        });
    });
    
    capture_exception(&error);
}
```

### Custom Error Dashboards

```json
{
  "dashboard": {
    "title": "WhatsApp Engine Errors",
    "panels": [
      {
        "title": "Error Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(whatsapp_errors_total[5m])",
            "legendFormat": "{{ error_type }}"
          }
        ]
      },
      {
        "title": "Top Errors",
        "type": "table",
        "targets": [
          {
            "expr": "topk(10, increase(whatsapp_errors_total[1h]))",
            "format": "table"
          }
        ]
      }
    ]
  }
}
```

## 📊 Dashboards

### Grafana Dashboard Configuration

```json
{
  "dashboard": {
    "id": null,
    "title": "WhatsApp Engine Monitoring",
    "tags": ["whatsapp", "messaging"],
    "timezone": "UTC",
    "panels": [
      {
        "id": 1,
        "title": "Request Rate",
        "type": "graph",
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 0},
        "targets": [
          {
            "expr": "rate(whatsapp_http_requests_total[5m])",
            "legendFormat": "{{ method }} {{ path }}"
          }
        ]
      },
      {
        "id": 2,
        "title": "Response Time",
        "type": "graph",
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 0},
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(whatsapp_http_request_duration_bucket[5m]))",
            "legendFormat": "95th percentile"
          },
          {
            "expr": "histogram_quantile(0.50, rate(whatsapp_http_request_duration_bucket[5m]))",
            "legendFormat": "50th percentile"
          }
        ]
      },
      {
        "id": 3,
        "title": "Messages Sent",
        "type": "stat",
        "gridPos": {"h": 4, "w": 6, "x": 0, "y": 8},
        "targets": [
          {
            "expr": "increase(whatsapp_messages_sent_total[1h])",
            "legendFormat": "Last Hour"
          }
        ]
      },
      {
        "id": 4,
        "title": "Active Browser Connections",
        "type": "stat",
        "gridPos": {"h": 4, "w": 6, "x": 6, "y": 8},
        "targets": [
          {
            "expr": "whatsapp_browser_connections_active",
            "legendFormat": "Active"
          }
        ]
      },
      {
        "id": 5,
        "title": "Memory Usage",
        "type": "graph",
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 12},
        "targets": [
          {
            "expr": "whatsapp_memory_usage_bytes",
            "legendFormat": "Memory Usage"
          }
        ]
      },
      {
        "id": 6,
        "title": "Error Rate",
        "type": "graph",
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 12},
        "targets": [
          {
            "expr": "rate(whatsapp_errors_total[5m])",
            "legendFormat": "{{ error_type }}"
          }
        ]
      }
    ]
  }
}
```

### Custom Business Metrics Dashboard

```json
{
  "dashboard": {
    "title": "WhatsApp Engine Business Metrics",
    "panels": [
      {
        "title": "Authentication Success Rate",
        "type": "stat",
        "targets": [
          {
            "expr": "(rate(whatsapp_auth_success_total[1h]) / rate(whatsapp_auth_attempts_total[1h])) * 100",
            "legendFormat": "Success Rate %"
          }
        ]
      },
      {
        "title": "Message Delivery Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(whatsapp_messages_sent_total{status=\"success\"}[5m])",
            "legendFormat": "Successful"
          },
          {
            "expr": "rate(whatsapp_messages_sent_total{status=\"failed\"}[5m])",
            "legendFormat": "Failed"
          }
        ]
      }
    ]
  }
}
```

## 🔧 Troubleshooting Playbooks

### High Error Rate Playbook

```markdown
# High Error Rate Response

## Immediate Actions (0-5 minutes)
1. Check current error rate: `rate(whatsapp_errors_total[5m])`
2. Identify error types: `topk(5, rate(whatsapp_errors_total[5m]) by (error_type))`
3. Check recent deployments
4. Verify external dependencies (WhatsApp Web status)

## Investigation (5-15 minutes)
1. Check logs for specific errors:
   ```bash
   kubectl logs -f deployment/whatsapp-engine | grep ERROR
   ```
2. Check browser connection health
3. Verify authentication status
4. Check system resources (CPU, memory)

## Resolution Steps
1. If browser issues: Restart browser service
2. If authentication issues: Clear sessions and re-authenticate
3. If resource issues: Scale up or restart pods
4. If external issues: Implement circuit breaker or retry logic
```

### Authentication Failure Playbook

```markdown
# Authentication Failure Response

## Immediate Checks
1. Verify WhatsApp Web accessibility
2. Check for rate limiting from WhatsApp
3. Verify browser functionality
4. Check session storage

## Investigation
1. Review authentication logs
2. Check QR code generation
3. Verify phone number formatting
4. Test manual authentication

## Resolution
1. Clear stored sessions
2. Restart browser service
3. Implement exponential backoff
4. Switch authentication method if needed
```

## 🛠️ Tool Integration

### ELK Stack Setup

```yaml
# docker-compose.monitoring.yml
version: '3.8'
services:
  elasticsearch:
    image: docker.elastic.co/elasticsearch/elasticsearch:8.11.0
    environment:
      - discovery.type=single-node
      - xpack.security.enabled=false
    ports:
      - "9200:9200"
    volumes:
      - elasticsearch_data:/usr/share/elasticsearch/data

  kibana:
    image: docker.elastic.co/kibana/kibana:8.11.0
    ports:
      - "5601:5601"
    environment:
      - ELASTICSEARCH_HOSTS=http://elasticsearch:9200
    depends_on:
      - elasticsearch

  logstash:
    image: docker.elastic.co/logstash/logstash:8.11.0
    volumes:
      - ./logstash/pipeline:/usr/share/logstash/pipeline
    environment:
      - xpack.monitoring.elasticsearch.hosts=http://elasticsearch:9200
    depends_on:
      - elasticsearch

volumes:
  elasticsearch_data:
```

### Grafana + Prometheus Setup

```yaml
# docker-compose.monitoring.yml (continued)
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus:/etc/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.console.libraries=/etc/prometheus/console_libraries'
      - '--web.console.templates=/etc/prometheus/consoles'

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3001:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana_data:/var/lib/grafana
      - ./grafana/provisioning:/etc/grafana/provisioning

volumes:
  grafana_data:
```

### Jaeger Tracing Setup

```yaml
# docker-compose.monitoring.yml (continued)
  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "16686:16686"
      - "14268:14268"
    environment:
      - COLLECTOR_OTLP_ENABLED=true
```

This comprehensive monitoring guide provides everything needed to observe, monitor, and troubleshoot WhatsApp Engine Rust in production environments.
