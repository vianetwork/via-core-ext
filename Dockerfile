FROM rust:1.90 as builder

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./

COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

RUN useradd -m backenduser

COPY --from=builder /usr/src/app/target/release/via_bridge_backend /usr/bin/via_bridge_backend

USER backenduser

CMD ["via_core_ext"]