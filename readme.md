# PubNub Kafka Bridge
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

### Dependency

Docker is used for running the sample Kafka/Zookeeper feed.
If you don't have docker, download and install it first.

> Docker is not required to run the bridge in production.
> There is an alternative non-container bridge install lower in this document.

**Install Docker:** https://docs.docker.com/install/#supported-platforms

## Part 1: Up and running in 60 seconds - Kafka, Zookeeper and Sample Feed

> If you already have Kafka cluster, you can skip to **Part 2**.

![Kafka to Mobile](https://i.imgur.com/aweweBQ.gif)

Start the docker compose file in a terminal window.
This will launch Kafka, Zookeeper and
a sample feed generator on the `topic` topic.

### Without SASL

```shell
git clone git@github.com:pubnub/kafka-bridge.git
cd kafka-bridge
docker-compose -f kafka/plain/docker-compose.yaml up --force-recreate --remove-orphans
```

### Using SASL_PLAINTEXT

```shell
git clone git@github.com:pubnub/kafka-bridge.git
cd kafka-bridge
docker-compose -f kafka/sasl_plaintext/docker-compose.yaml up --force-recreate --remove-orphans
```

### Using SASL_SSL

```shell
git clone git@github.com:pubnub/kafka-bridge.git
cd kafka-bridge
```

To use SASL_SSL user needs to generate sample certificates, this needs to be done only once:
```shell
cd kafka/sasl_ssl && ./generate-certs.sh && cd -
```

```shell
docker-compose -f kafka/sasl_ssl/docker-compose.yaml up --force-recreate --remove-orphans
```

Great!
Messages are bing simulated on the `topic` topic as we speak.
Now in a separate terminal session, run the dockerfile in **Part 2**.

## Part 2: Up and running in 60 seconds - PubNub Kafka Bridge

For security, you will need to get your private API keys from: 
https://dashboard.pubnub.com/signup
The following API Keys are for public-use and may be rotated.

Open a new terminal session and run the following commands:

### Without SASL

```shell
cd kafka-bridge
docker build -f kafka/plain/dockerfile -t kafka-bridge .
docker run                                                                        \
    --network=host                                                                \
    ## ~ Replace with your own API Keys ~ https://dashboard.pubnub.com/signup     \
    -e PUBNUB_PUBLISH_KEY=pub-c-6b57a39e-79e7-4d1d-926e-5c376a4cb021              \
    -e PUBNUB_SUBSCRIBE_KEY=sub-c-df3799ee-704b-11e9-8724-8269f6864ada            \
    -e PUBNUB_SECRET_KEY=sec-c-YWY3NzE0NTYtZTBkMS00YjJjLTgxZDQtN2YzOTY0NWNkNGVk   \
    ## ~ Replace with your own API Keys ~ https://dashboard.pubnub.com/signup     \
    -e PUBNUB_CHANNEL_ROOT=topics                                                 \
    -e PUBNUB_CHANNEL=*                                                           \
    -e KAFKA_GROUP=test-group                                                     \
    -e KAFKA_PARTITION=0                                                          \
    -e KAFKA_TOPIC=topic                                                          \
    -e KAFKA_BROKERS=0.0.0.0:9094                                                 \
    kafka-bridge
```

### Using SASL_PLAINTEXT

```shell
cd kafka-bridge
docker build -f kafka/sasl_plaintext/dockerfile -t kafka-bridge .
docker run                                                                        \
    --network=host                                                                \
    ## ~ Replace with your own API Keys ~ https://dashboard.pubnub.com/signup     \
    -e PUBNUB_PUBLISH_KEY=pub-c-6b57a39e-79e7-4d1d-926e-5c376a4cb021              \
    -e PUBNUB_SUBSCRIBE_KEY=sub-c-df3799ee-704b-11e9-8724-8269f6864ada            \
    -e PUBNUB_SECRET_KEY=sec-c-YWY3NzE0NTYtZTBkMS00YjJjLTgxZDQtN2YzOTY0NWNkNGVk   \
    ## ~ Replace with your own API Keys ~ https://dashboard.pubnub.com/signup     \
    -e PUBNUB_CHANNEL_ROOT=topics                                                 \
    -e PUBNUB_CHANNEL=*                                                           \
    -e KAFKA_GROUP=test-group                                                     \
    -e KAFKA_PARTITION=0                                                          \
    -e KAFKA_TOPIC=topic                                                          \
    -e KAFKA_BROKERS=0.0.0.0:9094                                                 \
    -e SASL_USERNAME=admin                                                        \
    -e SASL_USERNAME=admin-secret                                                 \
    kafka-bridge
```

### Using SASL_SSL

```shell
cd kafka-bridge
docker build -f kafka/sasl_plaintext/dockerfile -t kafka-bridge .
docker run                                                                        \
    --network=host                                                                \
    --hostname=kafka.confluent.io                                                 \
    --add-host=kafka.confluent.io:0.0.0.0                                         \
    ## ~ Replace with your own API Keys ~ https://dashboard.pubnub.com/signup     \
    -e PUBNUB_PUBLISH_KEY=pub-c-6b57a39e-79e7-4d1d-926e-5c376a4cb021              \
    -e PUBNUB_SUBSCRIBE_KEY=sub-c-df3799ee-704b-11e9-8724-8269f6864ada            \
    -e PUBNUB_SECRET_KEY=sec-c-YWY3NzE0NTYtZTBkMS00YjJjLTgxZDQtN2YzOTY0NWNkNGVk   \
    ## ~ Replace with your own API Keys ~ https://dashboard.pubnub.com/signup     \
    -e PUBNUB_CHANNEL_ROOT=topics                                                 \
    -e PUBNUB_CHANNEL=*                                                           \
    -e KAFKA_GROUP=test-group                                                     \
    -e KAFKA_PARTITION=0                                                          \
    -e KAFKA_TOPIC=topic                                                          \
    -e KAFKA_BROKERS=kafka.confluent.io:9094                                      \
    -e SASL_USERNAME=admin                                                        \
    -e SASL_USERNAME=admin-secret                                                 \
    -e SSL_CA_LOCATION=/etc/kafka/secrets/kafka-cluster.pem                       \
    -e SSL_CERTIFICATE_LOCATION=/etc/kafka/secrets/client/client.pem              \
    -e SSL_KEY_LOCATION=/etc/kafka/secrets/client/client.key                      \
    -e SSL_KEY_PASSWORD=secret                                                    \
    -v $PWD/kafka/sasl_ssl/secrets:/etc/kafka/secrets                             \
    kafka-bridge
```

Now your Kafka messages will be securely available to any device on the Internet.
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
You won't want to use this command directly, it is only for demonstration purposes.
To get a durable and secure connection, use one of the PubNub SDKs.

Messages from your Kafka cluster can be received on a mobile and web device.
Continue reading to get data onto your target devices easily using a PubNub SDK.

### Kafka -> Mobile Device

Messages from your Kafka cluster can be received
on a target mobile device.

> Find the mobile SDKs here: https://www.pubnub.com/docs

#### Test Console Output

Open the test console to see messages being received in a browser window.

[View Test Console](https://www.pubnub.com//docs/console?channel=topics.*&sub=sub-c-df3799ee-704b-11e9-8724-8269f6864ada&pub=pub-c-6b57a39e-79e7-4d1d-926e-5c376a4cb021)

> Scroll a bit down on this page, you will see an output
element labeled **`messages`** on this screen with message logs:

### Mobile Device -> Kafka

You can send a message from the mobile device and receive it in your Kafka cluster.
The following shell command will **simulate** this:

```shell
while true; do                                                                                \
    PUBLISH_KEY="pub-c-6b57a39e-79e7-4d1d-926e-5c376a4cb021"                                  \
    SUBSCRIBE_KEY="sub-c-df3799ee-704b-11e9-8724-8269f6864ada"                                \
    CHANNEL="topics.mydevice"                                                                 \
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

Now you can run `cargo run --bin kafka-bridge`.
The EMP app is 12 factor and is configured via Environmental Variables.

### Without SASL

```shell
PUBNUB_PUBLISH_KEY=pub-c-6b57a39e-79e7-4d1d-926e-5c376a4cb021              \
PUBNUB_SUBSCRIBE_KEY=sub-c-df3799ee-704b-11e9-8724-8269f6864ada            \
PUBNUB_SECRET_KEY=sec-c-YWY3NzE0NTYtZTBkMS00YjJjLTgxZDQtN2YzOTY0NWNkNGVk   \
PUBNUB_CHANNEL_ROOT=topics                                                 \
PUBNUB_CHANNEL=*                                                           \
KAFKA_GROUP=test-group                                                     \
KAFKA_PARTITION=0                                                          \
KAFKA_TOPIC=topic                                                          \
KAFKA_BROKERS=0.0.0.0:9094                                                 \
RUST_BACKTRACE=1                                                           \
cargo run --bin kafka-bridge
```

### Using SASL_PLAINTEXT

```shell
PUBNUB_PUBLISH_KEY=pub-c-6b57a39e-79e7-4d1d-926e-5c376a4cb021              \
PUBNUB_SUBSCRIBE_KEY=sub-c-df3799ee-704b-11e9-8724-8269f6864ada            \
PUBNUB_SECRET_KEY=sec-c-YWY3NzE0NTYtZTBkMS00YjJjLTgxZDQtN2YzOTY0NWNkNGVk   \
PUBNUB_CHANNEL_ROOT=topics                                                 \
PUBNUB_CHANNEL=*                                                           \
KAFKA_GROUP=test-group                                                     \
KAFKA_PARTITION=0                                                          \
KAFKA_TOPIC=topic                                                          \
KAFKA_BROKERS=0.0.0.0:9094                                                 \
SASL_USERNAME=admin                                                        \
SASL_PASSWORD=admin-secret                                                 \
RUST_BACKTRACE=1                                                           \
cargo run --bin kafka-bridge --features sasl
```

## Reference Links

 - [Confluent Platform Docker Image Reference](https://docs.confluent.io/current/installation/docker/image-reference.html)
 - [DockerHub Confluent Enterprise](https://hub.docker.com/r/confluentinc/cp-enterprise-kafka)
