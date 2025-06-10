# Multi-stage build for optimal image size
FROM rust:1.70-bookworm AS builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release
RUN rm src/main.rs

# Copy source code
COPY src ./src

# Build application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies for Playwright
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    wget \
    gnupg \
    libnss3 \
    libnspr4 \
    libxss1 \
    libasound2 \
    libxtst6 \
    libxrandr2 \
    libasound2 \
    libpangocairo-1.0-0 \
    libatk1.0-0 \
    libcairo-gobject2 \
    libgtk-3-0 \
    libgdk-pixbuf2.0-0 \
    fonts-liberation \
    libappindicator3-1 \
    xdg-utils \
    && rm -rf /var/lib/apt/lists/*

# Install Node.js (required for Playwright)
RUN curl -fsSL https://deb.nodesource.com/setup_18.x | bash - \
    && apt-get install -y nodejs

# Install Playwright browsers
RUN npx playwright install chromium --with-deps

# Create app user
RUN useradd -m -u 1000 appuser

# Create app directory and temp directory
WORKDIR /app
RUN mkdir -p .temp && chown -R appuser:appuser /app

# Copy binary from builder
COPY --from=builder /app/target/release/whatsapp-engine-rust /app/

# Copy configuration
COPY config/ ./config/

# Change ownership
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/api/auth/status || exit 1

# Run the application
CMD ["./whatsapp-engine-rust"]
