FROM rust:1.93-slim AS builder

WORKDIR /app
COPY . .

# Build the daemon and HTTP engine in release mode.
RUN cargo build --release -p rnb_agent -p rnb_engine_http

FROM debian:bookworm-slim

RUN useradd -m rinnovo
USER rinnovo
WORKDIR /home/rinnovo

COPY --from=builder /app/target/release/rnb_agent /usr/local/bin/rnb_daemon
COPY --from=builder /app/target/release/rnb_engine_http /usr/local/bin/rnb_engine_http

# Default wiring; the daemon will learn to use these as configuration.
ENV RINNOVO_REGISTRAR_URL="http://registrar:8000" \
    RINNOVO_ENGINE_CMD="rnb_engine_http" \
    RINNOVO_ENGINE_PORT="8787"

EXPOSE 8787

# For now the daemon has no subcommands; it just starts and logs config.
ENTRYPOINT ["rnb_daemon"]

