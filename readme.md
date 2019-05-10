# WANBus by PubNub
> Bring NATS, Kafka, Redis and RabbitMQ to the real world.

Give your Queue super power.
Secure Communication for field mobile and IoT devices with your Message Bus.
Audit and access management protection.
Encryption of data in motion 2048bit TLS.
Add push notifications and streaming events to Mobile and Web clients
based on a NATs subjects.
Driver support for Kafka, Redis, RabbitMQ and more.
Easy drop-in operations.
Dashboard management page included.

![WANBus](https://repository-images.githubusercontent.com/178954890/dcda8900-6ce9-11e9-9cc5-a6b7b476ad65)

The WANBus driver is written in Rust.
WANBus connects locally on your data channels/topics specified by runtime
and deploy time configuration.
This creates a bridge which tunnels WAN Traffic over TLS
to your end-point field Mobile and IoT devices.
Built-in security model allows encrypted data in motion using 2048bit TLS.
Access management allows you to control
who can read/write events on your message bus.
Built-in detection for Unauthorized Access Attempts.

## Dashboard Mockups

![](https://i.imgur.com/BjC73ZN.png)

![](https://i.imgur.com/qP6nNBr.png)

## Up and running in 10 seconds

Want to try WANBus with NATS?
Test runtime with Docker Compose.
Easily test using `docker-compose`.

```shell
cd wanbus
docker-compose -f nats/docker-compose.yaml up 
```

Follow URL printed at boot up.
The URL will be visible inside JSON formated output.

You can modify the ENVVARs in `./nats/docker-compose.yaml` file.

## Build and Run with Docker

Production runtime Alpine image size is 6MB.

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

## Reference Links

[https://hub.docker.com/nats](https://hub.docker.com/_/nats)


## TODOs List

 - TODO - 
 - TODO - prevent broadcast loop ( message filter + metadata )
 -      - don't receive own message (pubnub)
 - TODO - use STORAGE for catchup [ optionally ]
 - TODO - 
 - TODO - add pub/sub mpsc instead of direct access
 - TODO - add enhanced pubnub subscribe via /v2/stream/
 - TODO - add improved pubnub publish via mpsc.
 - TODO - add a STATS thread that collects counters and sends them over a PubNub channel.
 -      - this allows for a dashboard to see that status of the container.
 - TODO - TLS on PubNub
 -      - https://docs.rs/native-tls/0.2.2/native_tls/
 - TODO - Secret Key Signing on Publish and Subscribe
 - TODO - AsyncAwait & EPOLL ( https://jsdw.me/posts/rust-asyncawait-preview/  )
 - TODO - dashboard
