# syntax=docker/dockerfile:1
# check=error=true

FROM node:25-trixie-slim AS frontend-builder
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

FROM rust:1.99.0-trixie AS chef
ENV RUSTUP_TOOLCHAIN=${RUST_VERSION}
RUN cargo install --locked cargo-chef --version 0.1.78
WORKDIR /app

FROM chef AS planner
COPY backend/ ./
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS backend-builder
RUN apt-get update \
    && apt-get install --yes --no-install-recommends protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --locked --package sopho --recipe-path recipe.json
COPY backend/ ./
RUN cargo build --release --locked --package sopho

FROM debian:trixie-slim
RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates libssl3t64 \
    && rm -rf /var/lib/apt/lists/*
ENV FRONTEND_DIR=/app/frontend/dist/
COPY --from=frontend-builder /app/frontend/dist /app/frontend/dist
WORKDIR /app/backend
COPY --from=backend-builder /app/target/release/sopho ./sopho
COPY backend/demo /app/demo
EXPOSE 8000
CMD ["./sopho"]
