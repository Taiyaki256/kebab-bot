FROM rust:1.82-slim as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libfreetype6-dev \
    libfontconfig1-dev \
    build-essential \
    cmake \
    && rm -rf /var/lib/apt/lists/*

# Copy manifest files
COPY Cargo.toml Cargo.lock ./
COPY migration/Cargo.toml ./migration/

# Copy source code
COPY src ./src
COPY migration/src ./migration/src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libfreetype6 \
    libfontconfig1 \
    fonts-noto-cjk \
    fonts-liberation \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false kebab-bot

# Copy the binary
COPY --from=builder /app/target/release/kebab-bot /usr/local/bin/kebab-bot

# Set permissions
RUN chmod +x /usr/local/bin/kebab-bot

# Switch to non-root user
USER kebab-bot

# Run the application
CMD ["kebab-bot"]