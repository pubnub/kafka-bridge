# Edge Messaging Platform for Kafka
> Bring Kafka into the real world.

Messages from your Kafka cluster can be received
on a target mobile device.
Bring your Kafka cluster extra consumer power.
Secure Communication for field mobile and IoT devices with your Message Bus.
Audit and access management protection.
Encryption of data in motion 2048bit TLS.
Additional AES Symmetric key Cipher.
Add push notifications and streaming events to Mobile and Web clients
based on a Kafka topics.
Easy drop-in operations.
Dashboard management page included.

## Docker Compose - Up and running in 60 seconds ( includes Kafka and Zookeeper )

Want to try EMP with Kafka as a demo?
Demo is available as a runtime with Docker Compose.
Easily try using `docker-compose`.

```shell
git clone git@github.com:stephenlb/edge-messaging-platform.git
cd edge-messaging-platform
docker-compose -f kafka/docker-compose.yaml up --force-recreate --remove-orphans
```

Great! Everything is running now.
Messages are automatically simulated on `topic`.

## Dockerfile - Up and running in 60 seconds ( excludes Kafka )

This is what you'll use for a production environment.
For security, you will need to get your private API keys from: 
https://dashboard.pubnub.com/signup
The following API Keys are for public-use and may be rotated.

```shell
cd edge-messaging-platform
docker build -f kafka/dockerfile -t kafka-edge-messaging-platform .
docker run                                                                        \
    --network=host                                                                \
    -e PUBNUB_PUBLISH_KEY=pub-c-6b57a39e-79e7-4d1d-926e-5c376a4cb021              \
    -e PUBNUB_SUBSCRIBE_KEY=sub-c-df3799ee-704b-11e9-8724-8269f6864ada            \
    -e PUBNUB_SECRET_KEY=sec-c-YWY3NzE0NTYtZTBkMS00YjJjLTgxZDQtN2YzOTY0NWNkNGVk   \
    -e PUBNUB_CIPHER_KEY=pAsSwOrD                                                 \
    -e PUBNUB_CHANNEL_ROOT=topics                                                 \
    -e PUBNUB_CHANNEL=*                                                           \
    -e KAFKA_TOPIC_ROOT=topics                                                    \
    -e KAFKA_GROUP=                                                               \
    -e KAFKA_PARTITION=0                                                          \
    -e KAFKA_TOPIC=topic                                                          \
    -e KAFKA_BROKERS=0.0.0.0:9094                                                 \
    -e RUST_BACKTRACE=1                                                           \
    kafka-edge-messaging-platform
```

### Kafka -> Mobile Device

Messages from your Kafka cluster can be received
on a target mobile device.


#### Test Console Output

Open the test console to see messages being received in a browser window.

[View Test Console](https://www.pubnub.com//docs/console?channel=topics.*&sub=sub-c-df3799ee-704b-11e9-8724-8269f6864ada&pub=pub-c-6b57a39e-79e7-4d1d-926e-5c376a4cb021)

> Scroll a bit down on this page, you will see an output
element labeled **`messages`** on this screen with message logs:

### Mobile Device -> Kafka

You can send a message from the mobile device and receive it in your Kafka cluster.
The following shell command will simulate this:

```shell
while true; do                                                                                \
    PUBLISH_KEY="pub-c-6b57a39e-79e7-4d1d-926e-5c376a4cb021"                                  \
    SUBSCRIBE_KEY="sub-c-df3799ee-704b-11e9-8724-8269f6864ada"                                \
    CHANNEL="topics.mydevice"                                                               \
    curl "https://ps.pndsn.com/publish/$PUBLISH_KEY/$SUBSCRIBE_KEY/0/$CHANNEL/0/%22Hello%22"; \
    echo;                                                                                     \
    sleep 0.5;                                                                                \
done
```

This command will simulate a mobile device sending messages to the PubNub Edge
where they are copied to your Kafka cluster.

You will see a `"Hello"` message every half-second.

### Few more details

You can modify the ENVVARs in `./kafka/docker-compose.yaml` file.

That's it!
If you can't use Docker Compose,
look at the alternative setup instructions below.

## Kafka Wildcard Channel Support

> Keep this in mind when configuration your runtime
> ENVIRONMENTAL variables.



Visit the URL printed from the output.

Publish Kafka messages repeatedly.

Subscribe to these messages in another terminal window.

```shell
while true;
    do (printf "SUB subjects.mydevice 1\r\n"; sleep 60) | nc 0.0.0.0 4222;
done
```

Issue several Kafka commands in a single key press.

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

## Second Alternate Installation

> Binary Standalone Usage Instructions.

If you can't use the first two installation methods,
then you can use the following alternative installation instructions.

You need `Rust` and `Kafka`.

#### Get Rust

Instructions posted here
https://www.rust-lang.org/tools/install

```shell
## Rust Quick Install
curl https://sh.rustup.rs -sSf | sh
```

#### Run with Cargo

Now you can run `cargo run --bin kafka`.
The EMP app is 12 factor and is configured via Environmental Variables.

```shell
PUBNUB_PUBLISH_KEY=pub-c-6b57a39e-79e7-4d1d-926e-5c376a4cb021              \
PUBNUB_SUBSCRIBE_KEY=sub-c-df3799ee-704b-11e9-8724-8269f6864ada            \
PUBNUB_SECRET_KEY=sec-c-YWY3NzE0NTYtZTBkMS00YjJjLTgxZDQtN2YzOTY0NWNkNGVk   \
PUBNUB_CIPHER_KEY=pAsSwOrD                                                 \
PUBNUB_CHANNEL_ROOT=topics                                                 \
PUBNUB_CHANNEL=*                                                           \
KAFKA_TOPIC_ROOT=topics                                                    \
KAFKA_GROUP=                                                               \
KAFKA_PARTITION=0                                                          \
KAFKA_TOPIC=topic                                                          \
KAFKA_BROKERS=0.0.0.0:9094                                                 \
RUST_BACKTRACE=1                                                           \
cargo run --bin kafka
```

## Reference Links

[https://hub.docker.com/kafka](https://hub.docker.com/_/kafka)
