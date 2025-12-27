# Frontend builder stage
FROM node:20 AS frontend-builder
WORKDIR /usr/src/app/frontend
COPY frontend/package*.json ./
RUN npm ci
COPY frontend/ .
RUN npm run build

# Backend builder stage
FROM rust:1.78 AS backend-builder
WORKDIR /usr/src/app
RUN USER=root cargo new --bin .
COPY backend/Cargo.toml backend/Cargo.lock ./
RUN cargo build --release
RUN rm src/*.rs
RUN rm target/release/deps/app*
COPY backend/src ./src
RUN cargo build --release

# Final stage
FROM debian:bullseye-slim
ARG APP_USER=appuser
RUN groupadd -r ${APP_USER} && useradd -r -g ${APP_USER} ${APP_USER}

# Copy frontend build output to /usr/src/frontend/dist
# (backend expects frontend at project_root/frontend/dist, where project_root is parent of CARGO_MANIFEST_DIR)
RUN mkdir -p /usr/src/frontend
COPY --from=frontend-builder /usr/src/app/frontend/dist /usr/src/frontend/dist

# Copy backend binary
WORKDIR /usr/local/bin
COPY --from=backend-builder /usr/src/app/target/release/sopho ./sopho
RUN chmod +x sopho

USER ${APP_USER}
EXPOSE 8000
CMD ["./sopho"]
