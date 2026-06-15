FROM rust:1.75 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ffmpeg \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/vidspeed /usr/local/bin/vidspeed
COPY --from=builder /app/.env.example /app/.env

EXPOSE 3000

CMD ["vidspeed", "server"]
