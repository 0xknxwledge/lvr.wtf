ARG TARGETOS=linux
ARG TARGETARCH=x86_64


FROM rust:1-alpine AS chef
RUN apk add --no-cache musl-dev build-base openssl-dev pkgconfig
RUN cargo install cargo-chef
WORKDIR /app/backend

FROM chef AS planner
COPY backend/ .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/backend/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY backend/ .

RUN cargo build --release 

FROM alpine:latest AS runtime
RUN apk add --no-cache ca-certificates libgcc
RUN addgroup -S myuser && adduser -S myuser -G myuser
COPY --from=builder /app/backend/target/release/backend /usr/local/bin/backend
USER myuser

ENV BRONTES_HOST='REDACTED_BRONTES_HOST'
ENV BRONTES_PORT='REDACTED_BRONTES_PORT'
ENV BRONTES_USER='REDACTED_BRONTES_USER'
ENV BRONTES_PASSWORD='REDACTED_BRONTES_PASSWORD'
ENV AURORA_HOST='REDACTED_AURORA_HOST'
ENV AURORA_PORT='REDACTED_AURORA_PORT'
ENV AURORA_USER='REDACTED_AURORA_USER'
ENV AURORA_PASSWORD='REDACTED_AURORA_PASSWORD'
ENV AURORA_DATABASE='REDACTED_AURORA_DATABASE'

EXPOSE 3000
ENTRYPOINT ["/usr/local/bin/backend", "serve", "--host", "0.0.0.0", "--port", "3000"]