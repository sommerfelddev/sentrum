FROM rust:bookworm as builder

WORKDIR /usr/src/sentrum
COPY . .

RUN cargo build --release

FROM debian:bookworm-slim as final

# Upgrade all packages and install dependencies
RUN apt-get update \
    && apt-get upgrade -y
RUN DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends libssl-dev \
    ca-certificates \
    && apt clean && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

COPY --from=builder /usr/src/sentrum/target/release/sentrum /usr/local/bin/sentrum

COPY sentrum.sample.toml sentrum.toml

CMD ["sentrum"]
