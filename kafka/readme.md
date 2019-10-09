# Edge Messaging Platform for Kafka
> Bring Kafka into the real world.

Messages from your Kafka cluster can be received
on a target mobile device.
Bring your Kafka cluster extra consumer power.
Secure Communication for field mobile and IoT devices with your Message Bus.
Audit and access management protection.
Encryption of data in motion 2048bit TLS.
Add push notifications and streaming events to Mobile and Web clients
based on your Kafka topics.
Easy drop-in operations with Docker or Cargo.

## Part 1: Up and running in 60 seconds - Kafka, Zookeeper and Sample Feed

> If you already have Kafka cluster, you can skip to **Part 2**.

![Kafka to Mobile](https://i.imgur.com/aweweBQ.gif)

Start the docker compose file in a terminal window.
This will launch Kafka, Zookeeper and
a sample feed generator on the `topic` topic.

```shell
git clone git@github.com:stephenlb/edge-messaging-platform.git
cd edge-messaging-platform
docker-compose -f kafka/docker-compose.yaml up --force-recreate --remove-orphans
```

Great!
Messages are bing simulated on the `topic` topic as we speak.
Now in a separate terminal session, run the dockerfile in **Part 2**.

## Part 2: Up and running in 60 seconds - Edge Messaging Platform Dockerfile

For security, you will need to get your private API keys from: 
https://dashboard.pubnub.com/signup
The following API Keys are for public-use and may be rotated.

Open a new terminal session and run the following commands:

```shell
cd edge-messaging-platform
docker build -f kafka/dockerfile -t kafka-edge-messaging-platform .
docker run                                                                        \
    --network=host                                                                \
    ## ~ Replace with your own API Keys ~ https://dashboard.pubnub.com/signup     \
    -e PUBNUB_PUBLISH_KEY=pub-c-6b57a39e-79e7-4d1d-926e-5c376a4cb021              \
    -e PUBNUB_SUBSCRIBE_KEY=sub-c-df3799ee-704b-11e9-8724-8269f6864ada            \
    -e PUBNUB_SECRET_KEY=sec-c-YWY3NzE0NTYtZTBkMS00YjJjLTgxZDQtN2YzOTY0NWNkNGVk   \
    ## ~ Replace with your own API Keys ~ https://dashboard.pubnub.com/signup     \
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

Now your Kafka messages will be securly available to any device on the Internet.
You can receive a stream of your messages in **Part 3**.

## Part 3: Receive Messages on Your Mobile Device from Kafka Topics

### Terminal Session Example

The following command is an example of how to receive messages from
your Kafka cluster on the Internet.

> Note the following sample terminal command opens a lossy sample feed.
> Use the PubNub SDKs directly to **enable durable message delivery**.

Open a new terminal session and run the following commands:

```shell
export SUBSCRIBE_KEY="sub-c-df3799ee-704b-11e9-8724-8269f6864ada"
export TOPICS="topics.*"
export HEADERS="HTTP/1.1\r\nHost: pubnub\r\n\r\n"
while true; do (                                                                              \
    printf "GET http://p.pubnub.com/stream/$SUBSCRIBE_KEY/$TOPICS/0/-1 $HEADERS"; sleep 10) | \
    nc p.pubnub.com 80; done
```

You are seeing messages being delivered from your Kafka cluster directly to your terminal session.
This terminal session can be located anywhere on Earth with an active internet connection.
You won't want to use this command directly, it is only for domonstration purposes.
To get a durable and secure connection, use one of the PubNub SDKs.

Messages from your Kafka cluster can be received on a mobile and web device.
Continue reading to get data onto your target devices easily using a PubNub SDK.

# TODO - the documentation below will not work... yet.
# TODO - the documentation below will not work... yet.
# TODO - the documentation below will not work... yet.

### Android Java

### iOS Swift

### iOS Objective-C

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
