FROM rust:1.78 AS builder
WORKDIR /usr/src/app
RUN USER=root cargo new --bin .
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release
RUN rm src/*.rs
RUN rm target/release/deps/app*
COPY ./src ./src
RUN cargo build --release

FROM debian:bullseye-slim
ARG APP_USER=appuser
RUN groupadd -r ${APP_USER} && useradd -r -g ${APP_USER} ${APP_USER}
WORKDIR /usr/local/bin
COPY --from=builder /usr/src/app/target/release/sopho .
RUN chmod +x sopho
USER ${APP_USER}
EXPOSE 8000
CMD ["./sopho"]
