# Build stage
FROM rust:1.75-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy src to cache dependencies
RUN mkdir -p src/bin && \
    echo "fn main() {}" > src/main.rs && \
    echo "fn main() {}" > src/bin/whatsapp-server.rs && \
    echo "" > src/lib.rs

# Build dependencies only
RUN cargo build --release --features mcp || true

# Copy actual source
COPY src ./src

# Build the application
RUN touch src/lib.rs src/bin/whatsapp-server.rs && \
    cargo build --release --features mcp

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies (Chrome/Chromium)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    chromium \
    chromium-sandbox \
    fonts-liberation \
    libasound2 \
    libatk-bridge2.0-0 \
    libatk1.0-0 \
    libcups2 \
    libdbus-1-3 \
    libdrm2 \
    libgbm1 \
    libgtk-3-0 \
    libnspr4 \
    libnss3 \
    libxcomposite1 \
    libxdamage1 \
    libxfixes3 \
    libxkbcommon0 \
    libxrandr2 \
    xdg-utils \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/was /usr/local/bin/

# Copy config
COPY config/app.example.toml /app/config/app.toml

# Copy templates and static assets
COPY templates /app/templates
COPY static /app/static

# Create data directory for browser profile
RUN mkdir -p /app/data

# Set environment variables
ENV WHATSAPP_HOST=0.0.0.0
ENV WAS__SERVER__PORT=3000
ENV CHROME_PATH=/usr/bin/chromium

EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/api/health || exit 1

CMD ["was"]
