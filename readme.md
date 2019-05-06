# WANBus by PubNub
> Bring NATS, Kafka, Redis and RabbitMQ to the real world.

Give your Messages flexibility.
Secure Communication for field mobile and IoT devices with your Message Bus.
Audit and access management protection.
Encryption of data in motion.
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

## Build and Run with Docker Container

Production runtime Alpine image size is 6MB.

```shell
docker build -f nats/dockerfile --cpuset-cpus 3 -t nats-wanbus .
```

```shell
docker run .................
    -e PUBNUB_SUBKEY=demo
    -e PUBNUB_PUBKEY=demo
    -e PUBNUB_SECKEY=demo
    -e NATS_SUBJECTS=* ## comma delimited list of subjects to sync
    -e NATS_HOST=nats
    -e NATS_USER=
    -e NATS_PASSWORD=
    -e NATS_AUTH_TOKEN=
```

## Test Runtime with Docker Compose

Want to try WANBus with NATS?
Easily test using `docker-compose`.

```shell
# TODO Test this command
docker-compose up -f docker-compose.yaml
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


## Reference Links

[https://hub.docker.com/nats](https://hub.docker.com/_/nats)


## TODOs List

 - TODO - BUG - if NATS message isn't JSON, convert to JSON String
 - TODO - 
 - TODO - add pub/sub mpsc instead of direct access
 - TODO - add improved pubnub subscribe via /v2/stream/
 - TODO - add improved pubnub publish via mpsc.
 - TODO - add socket response timeout
 - TODO - prevent broadcast loop ( message filter + metadata )
 - TODO - add a STATS thread that collects counters and sends them over a PubNub channel.
 -      - this allows for a dashboard to see that status of the container.
 - TODO - multiple channels NATS - https://doc.rust-lang.org/std/sync/mpsc/#examples
 -      - Multple Channels support (NATS)
 - TODO - read line one last time to get response body
 - TODO - TLS on PubNub
 -      - https://docs.rs/native-tls/0.2.2/native_tls/
 - TODO - JSON decode PubNub response GET TIMETOKEN return Result
 - TODO - JSON
 - TODO - Secret Key Signing on Publish
 - TODO - agent and sec key PubNub struct
 - TODO - pnsdk=nats-to-mobile / nats-wanbus
 - TODO - AsyncAwait & EPOLL ( https://jsdw.me/posts/rust-asyncawait-preview/  )
 - TODO - socket hangout needs to reconnect
 - TODO - recover from disconnect or restart?
 - TODO - http parser?
 - TODO - pubnub.subscribe via /v2/stream endpoint
 - TODO - /// Rust DOC comments?
 - TODO - ENV vars for runtime config ( pub/sub/sec/origin/channel-list )
 - TODO - add more tests!!!!!!!
 - TODO - remove all `println!()` except the system log stdout
 - TODO - dashboard
