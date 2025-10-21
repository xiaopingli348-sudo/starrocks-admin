# Multi-stage build for StarRocks Admin
FROM rust:1.75 as builder

# Install Node.js for frontend build
RUN curl -fsSL https://deb.nodesource.com/setup_18.x | bash - \
    && apt-get install -y nodejs

WORKDIR /app

# Copy backend source
COPY backend/ ./backend/
WORKDIR /app/backend

# Build backend
RUN cargo build --release

# Build frontend
WORKDIR /app
COPY frontend/ ./frontend/
WORKDIR /app/frontend

# Install dependencies and build
RUN npm ci --only=production
RUN npm run build -- --configuration production

# Final stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false starrocks

# Create app directory structure
WORKDIR /app
RUN mkdir -p bin conf web lib data logs

# Copy built artifacts
COPY --from=builder /app/backend/target/release/starrocks-admin ./bin/
COPY --from=builder /app/frontend/dist/ngx-admin/ ./web/
COPY conf/ ./conf/

# Set permissions
RUN chown -R starrocks:starrocks /app
RUN chmod +x ./bin/starrocks-admin

# Switch to non-root user
USER starrocks

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Start command
CMD ["./bin/starrocks-admin"]
