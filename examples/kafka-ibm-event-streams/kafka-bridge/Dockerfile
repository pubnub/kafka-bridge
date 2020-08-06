# Application Build
FROM rust:latest as build

RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        cmake \
    ; \
    rm -rf /var/lib/apt/lists/*;

WORKDIR /app
COPY . /app/

RUN cargo test
RUN cargo build --release --features sasl-plain
RUN strip target/release/kafka-bridge

# Runtime Build
FROM alpine:latest

# Install glibc
RUN set -eux; \
    wget -q -O /etc/apk/keys/sgerrand.rsa.pub https://alpine-pkgs.sgerrand.com/sgerrand.rsa.pub; \
    wget https://github.com/sgerrand/alpine-pkg-glibc/releases/download/2.31-r0/glibc-2.31-r0.apk; \
    apk add glibc-2.31-r0.apk

# Install runtime dependencies
RUN apk --no-cache add \
        libgcc \
        tini

RUN addgroup -g 1000 appuser && adduser -S -u 1000 -g appuser -G appuser appuser
USER appuser

WORKDIR /app
COPY --from=build /app/target/release/kafka-bridge /app

# Setup the environment
ENTRYPOINT ["/sbin/tini", "--"]

# Runtime
ENV LD_LIBRARY_PATH=/lib:/usr/lib \
    RUST_BACKTRACE=1
CMD ["/app/kafka-bridge"]
