FROM docker.io/lukemathwalker/cargo-chef:latest-rust-alpine AS chef
# aws-lc-sys (rustls' crypto backend) compiles C code
RUN apk add --no-cache build-base
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
# Rust on Alpine targets musl, which links the binary statically
RUN cargo build --release --locked \
    && cp target/release/bookorbit-moly-provider /bookorbit-moly-provider

# The binary bundles its TLS roots, so the image needs no CA store
FROM scratch
COPY --from=builder /bookorbit-moly-provider /bookorbit-moly-provider
USER 65534:65534
EXPOSE 8080
ENTRYPOINT ["/bookorbit-moly-provider"]
