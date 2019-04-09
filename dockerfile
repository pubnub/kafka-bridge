# Dependency Build
FROM rust:latest as dependencies
ENV USER=root
RUN cargo new app
WORKDIR /app
COPY Cargo.* /app/
RUN apt-get update
RUN apt-get -y install musl-tools libssl-dev
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target=x86_64-unknown-linux-musl
RUN rm ./src/*.rs ./target/x86_64-unknown-linux-musl/release/nats-bridge

# Application Build
FROM dependencies as build
ARG version=0.0.0
WORKDIR /app
COPY . /app/
RUN OLD_VERSION=$(egrep '^version = "[0-9a-z.+-]+"$' Cargo.toml | cut -d'"' -f2 | sed -e 's/\./\\./g') && \
    sed -i -e "s/$OLD_VERSION/$version/" Cargo.toml
RUN cargo build --release --target=x86_64-unknown-linux-musl
RUN strip target/x86_64-unknown-linux-musl/release/nats-bridge

# Runtime Build
FROM alpine:latest
RUN apk --no-cache add tini
RUN addgroup -g 1000 appuser
RUN adduser -S -u 1000 -g appuser -G appuser appuser
USER appuser
WORKDIR /app
COPY --from=build /app/target/x86_64-unknown-linux-musl/release/nats-bridge /app

# Setup the environment
ENTRYPOINT ["/sbin/tini", "--"]

# Runtime
ENV RUST_BACKTRACE=1
ENV NATS_HOST="0.0.0.0:4222"
ENV NATS_CHANNELS="a,b,c,d"
CMD ["/app/nats-bridge"]
