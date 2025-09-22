# syntax=docker/dockerfile:1.7

# ---- Builder stage: compile the Rust CLI ----
FROM rust:1.89-slim-bookworm AS builder

# Create app dir and copy manifests first to leverage Docker layer cache
WORKDIR /app

# Install minimal build toolchain for crates with native code (if any)
RUN apt-get update \
    && apt-get install -y --no-install-recommends build-essential pkg-config ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy source
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY build.rs ./build.rs
COPY locales ./locales
COPY README.md ./README.md
COPY cofg.schema.json ./cofg.schema.json

# Build in release mode
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo build --release --locked

# ---- Optional tests stage ----
FROM builder AS tester
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo test --locked

# ---- Runtime stage: minimal image to run the CLI ----
FROM debian:bookworm-slim AS runtime

# Install only what we need at runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user and working directory
RUN groupadd -r app && useradd -r -g app app \
    && mkdir -p /work \
    && chown -R app:app /work

# Copy the compiled binary
COPY --from=builder /app/target/release/mod-dev /usr/local/bin/mod-dev
RUN chmod 0755 /usr/local/bin/mod-dev

# Provide a sensible default configuration to avoid interactive prompts.
# Users can override by bind-mounting their own cofg.json into /work.
COPY --chown=app:app docker/cofg.json /work/cofg.json

# Drop privileges
USER app
WORKDIR /work

# Basic healthcheck to ensure the binary can execute
HEALTHCHECK --interval=30s --timeout=5s --retries=3 CMD /usr/local/bin/mod-dev --version > /dev/null 2>&1 || exit 1

# The tool reads/writes files in the working directory (./mods, ./tmp, ./results)
# Mount a host directory to /work to persist outputs
# Example: docker run --rm -v %cd%:/work rust-mod-dev:latest --help

ENTRYPOINT ["/usr/local/bin/mod-dev"]
CMD ["--help"]
