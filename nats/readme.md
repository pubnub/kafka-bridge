# WANBus for NATS
> Bring NATS to the real world.

Give your NATS cluster super power.
Secure Communication for field mobile and IoT devices with your Message Bus.
Audit and access management protection.
Encryption of data in motion 2048bit TLS.
Additional AES Symmetric key Cipher.
Add push notifications and streaming events to Mobile and Web clients
based on a NATs subjects.
Easy drop-in operations.
Dashboard management page included.

 - TODO - Channel mapping and `ROOT` concepts.
 - TODO - Docker Compose
 - TODO - Examples
 - TODO - println!() DEV CONSOLE URL ( to be replaced with DASHBOARD )
 - TODO - 
 - TODO - 
 - TODO - 
 - TODO - 
 - TODO - 

## Run with Docker

Production docker runtime Alpine image size is **6MB**.
Start by building the NATS WANbus image.

```shell
cd wanbus
docker build -f nats/dockerfile --cpuset-cpus 3 -t nats-wanbus .
```

Run a local NATS instance.

```shell
docker run -p 4222:4222 nats
```

Run the NATS WANBus.
For security, you will need to get your private API keys from: 
https://dashboard.pubnub.com/signup
The following API Keys are for public use and may be rotated.

```shell
docker run \
  --network=host \
  -e PUBNUB_PUBLISH_KEY=pub-c-6b57a39e-79e7-4d1d-926e-5c376a4cb021 \
  -e PUBNUB_SUBSCRIBE_KEY=sub-c-df3799ee-704b-11e9-8724-8269f6864ada \
  -e PUBNUB_SECRET_KEY=sec-c-YWY3NzE0NTYtZTBkMS00YjJjLTgxZDQtN2YzOTY0NWNkNGVk \
  -e PUBNUB_CIPHER_KEY=pAsSwOrD \
  -e PUBNUB_CHANNEL_ROOT=channels \
  -e PUBNUB_CHANNEL=* \
  -e NATS_SUBJECT_ROOT=subjects \
  -e NATS_SUBJECT=* \
  -e NATS_HOST=0.0.0.0:4222 \
  nats-wanbus
```

Visit the URL printed from the output.

## Test Runtime with Docker Compose

Want to try WANBus with NATS?
Easily test using `docker-compose`.

```shell
# TODO Test this command
# TODO Test this command
# TODO Test this command
# TODO Test this command
docker-compose -f nats/docker-compose.yaml up 
```

##  Easy NATS Terminal Test

Publish Messages in a Loop.

```shell
while true;
    do (printf "PUB demo 5\r\nKNOCK\r\n"; sleep 0.4) | nc 0.0.0.0 4222;
done
```

Subscribe to these messages in another termianl window.

```shell
while true;
    do (printf "SUB demo 1\r\n"; sleep 60) | nc 0.0.0.0 4222;
done
```

```shell
(printf "SUB FOO 1\r\n"; sleep 5) | nc 0.0.0.0 4222 &
(printf "PING\r\n";                        sleep 0.4; \
 printf "CONNECT {\"verbose\":false}\r\n"; sleep 0.4; \
 printf "SUB BAR 1\r\n";                   sleep 0.4; \
 printf "PING\r\n";                        sleep 0.4; \
 printf "PUB FOO 11\r\nKNOCK KNOCK\r\n";   sleep 0.4; \
 printf "PING\r\n";                        sleep 0.4; \
 printf "PUB BAR 11\r\nKNOCK KNOCK\r\n";   sleep 0.4; \
 printf "PING\r\n";                        sleep 0.4; \
) | nc 0.0.0.0 4222 
```

## Deploying Docker Compose

```shell
docker run -e "NATS_HOST=0.0.0.0:4222" -e "NATS_CHANNELS=a,b,c" nats-wanbus
```

## Deploying Sidecar in Kubernetes (K8s)

Add Secrets to secret store.
Add YAML sidecar

## Build and test with Rust Cargo

You'll need a local NATS server running.


Now you can run `cargo run --bin nats-wanbus`.
The WANBus app is 12 factor and is configured via Environmental Variables.

```shell
PUBNUB_PUBLISH_KEY=pub-c-6b57a39e-79e7-4d1d-926e-5c376a4cb021 \
PUBNUB_SUBSCRIBE_KEY=sub-c-df3799ee-704b-11e9-8724-8269f6864ada \
PUBNUB_SECRET_KEY=sec-c-YWY3NzE0NTYtZTBkMS00YjJjLTgxZDQtN2YzOTY0NWNkNGVk \
PUBNUB_CHANNEL_ROOT=channels \
PUBNUB_CHANNEL=* \
PUBNUB_CIPHER_KEY=pAsSwOrD \
NATS_SUBJECT_ROOT=subjects \
NATS_SUBJECT=* \
NATS_HOST=0.0.0.0:4222 \
cargo run --bin nats-wanbus
```

## Reference Links

[https://hub.docker.com/nats](https://hub.docker.com/_/nats)
