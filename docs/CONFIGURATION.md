# Configuration Guide

WAS uses TOML configuration with environment variable overrides.

## Configuration File

```bash
cp config/app.example.toml config/app.toml
```

## Server

```toml
[server]
host = "0.0.0.0"
port = 3000
```

## Browser

```toml
[browser]
headless = true
timeout_ms = 30000
args = [
    "--no-sandbox",
    "--disable-dev-shm-usage",
    "--disable-gpu"
]
```

## Authentication

```toml
[auth]
secret_key = "change-this-in-production"
```

## Logging

```toml
[logging]
level = "info"  # trace, debug, info, warn, error
```

## CORS

```toml
[cors]
allow_origins = ["*"]
allow_methods = ["GET", "POST", "PUT", "DELETE", "OPTIONS"]
allow_headers = ["authorization", "content-type"]
```

## Limits

```toml
[limits]
max_concurrent_requests = 50
request_timeout_ms = 30000
max_upload_size = 10485760  # 10MB
```

## Environment

```toml
[environment]
environment = "development"  # development, staging, production
```

## Swagger

```toml
[swagger]
enabled = true
path = "/api-docs"
```

## Instances

```toml
[instances]
base_directory = "~/.was/instances"

[instances.defaults]
idle_timeout = 300
headless = true
```

## Environment Variables

| Variable | Config Path |
|----------|-------------|
| `WAS_HOST` | server.host |
| `WAS_PORT` | server.port |
| `WAS__AUTH__SECRET_KEY` | auth.secret_key |
| `WAS__BROWSER__HEADLESS` | browser.headless |
| `RUST_LOG` | Log level |

## Examples

### Development
```toml
[browser]
headless = false

[swagger]
enabled = true

[logging]
level = "debug"
```

### Production
```toml
[browser]
headless = true

[swagger]
enabled = false

[auth]
secret_key = "strong-random-key"

[cors]
allow_origins = ["https://yourdomain.com"]
```
