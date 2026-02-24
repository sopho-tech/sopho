# Frontend builder stage
FROM node:20 AS frontend-builder
WORKDIR /app
COPY frontend/package*.json ./
RUN npm ci
COPY frontend/ .
RUN npm run build

# Backend builder stage
FROM rust:1.93.1 AS backend-builder
WORKDIR /app
COPY backend/ .
RUN cargo build --release

# Final stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy frontend build output to /app/frontend/dist
# (backend expects frontend at project_root/frontend/dist, where project_root is parent of CARGO_MANIFEST_DIR)
RUN mkdir -p /app/frontend
COPY --from=frontend-builder /app/dist /app/frontend/dist

# Copy backend binary and demo data
WORKDIR /app/backend
COPY --from=backend-builder /app/target/release/sopho ./sopho
COPY backend/demo /app/demo
RUN chmod +x sopho

EXPOSE 8000
CMD ["./sopho"]
