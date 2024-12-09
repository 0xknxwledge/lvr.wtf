ARG TARGETOS=linux
ARG TARGETARCH=x86_64

FROM rustlang/rust:nightly AS chef
RUN apt-get update && apt-get -y upgrade && apt-get install -y libclang-dev pkg-config cmake libclang-dev
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json
RUN cargo +nightly chef cook --release --recipe-path recipe.json
COPY . .

RUN cargo build --release 

FROM alpine AS runtime
RUN addgroup -S myuser && adduser -S myuser -G myuser
COPY --from=builder /app/target/release/backend  /usr/local/bin/backend
USER myuser

ENV BRONTES_HOST='34.149.107.219'
ENV BRONTES_PORT='8123'
ENV BRONTES_USER='john_beecher'
ENV BRONTES_PASSWORD='dummy-password'
ENV AURORA_HOST='lvr-instance-1.cl6g4egiomeo.us-east-1.rds.amazonaws.com'
ENV AURORA_PORT='3306'
ENV AURORA_USER='admin'
ENV AURORA_PASSWORD='0XqTs6YHpd6A7MWtKf0N'
ENV AURORA_DATABASE='lvr'

EXPOSE 3000
ENTRYPOINT ["/usr/local/bin/backend serve --host 0.0.0.0 --port 3000"]