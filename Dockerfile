FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
RUN apt-get update && \
    apt-get install -y musl-tools build-essential pkg-config gcc libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY backend/ .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY backend/ .
RUN cargo build --release

FROM ubuntu AS runtime
COPY --from=builder /app/target/release/backend /usr/local/bin/backend
COPY backend/smeed /usr/local/bin/smeed
WORKDIR /usr/local/bin
USER root

ENV BRONTES_HOST='34.85.191.184'
ENV BRONTES_PORT='REDACTED_BRONTES_PORT'
ENV BRONTES_USER='REDACTED_BRONTES_USER'
ENV BRONTES_PASSWORD='REDACTED_BRONTES_PASSWORD'
ENV AURORA_GCP_HOST='10.150.0.27'
ENV AURORA_PUBLIC_HOST='35.245.164.72'
ENV AURORA_PORT='REDACTED_AURORA_PORT'
ENV AURORA_USER='fenbushi'
ENV AURORA_PASSWORD='dummy-password'
ENV AURORA_DATABASE='REDACTED_AURORA_DATABASE'
ENV RUNNING_LOCALLY='false'

EXPOSE 50001
ENTRYPOINT ["/usr/local/bin/backend", "serve", "--host", "0.0.0.0", "--port", "50001"]
