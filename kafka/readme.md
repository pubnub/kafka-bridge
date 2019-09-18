# Edge Messaging Platform for Kafka
> Bring Kafka to the real world.

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

## Up and running in 10 seconds

Want to try EMP with Kafka?
Test runtime with Docker Compose.
Easily test using `docker-compose`.

```shell
git clone git@github.com:stephenlb/edge-messaging-platform.git
cd edge-messaging-platform
docker-compose -f kafka/docker-compose.yaml up
```

Great! Everything is running now.
Continue reading to simulate messages.

### Kafka -> Mobile Device

Messages from your Kafka cluster can be received
on a target mobile device.

#### 1.) Simulate Kafka Stream

Run this command in a terminal window.
This command will send a `"KNOCK"` message each half-second.

```shell
while true;
    do (printf "PUB subjects.mydevice 5\r\nKNOCK\r\n"; sleep 0.5) | nc 0.0.0.0 4222;
done
```

#### 2.) Test Console Output

Open the test console to see messages being received in a browser window.

[View Test Console](https://www.pubnub.com//docs/console?channel=channels.*&sub=sub-c-df3799ee-704b-11e9-8724-8269f6864ada&pub=pub-c-6b57a39e-79e7-4d1d-926e-5c376a4cb021)

> Scroll a bit down on this page, you will see an output
element labeled **`messages`** on this screen with message logs:

### Mobile Device -> Kafka

You can send a message from the mobile device and receive it in your Kafka cluster.
The following shell command will simulate this:

```shell
while true; do                                                                                \
    PUBLISH_KEY="pub-c-6b57a39e-79e7-4d1d-926e-5c376a4cb021"                                  \
    SUBSCRIBE_KEY="sub-c-df3799ee-704b-11e9-8724-8269f6864ada"                                \
    CHANNEL="channels.mydevice"                                                               \
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

Kafka wildcard symbols include `*` and `>`.
These symbols have different meanings.
The `*` symbol captures all messages for `root.*` and
will not capture `root.sub.*`.
The `>` symbol captures all messages below the root including sub nodes.

## Alternate Installation Instructions

If you can't use Docker Compose, then this is an alternative setup.
Production docker runtime Alpine image size is **6MB**.
Start by building the Kafka EMP image.

##### 1 of 3

Build EMP.

```shell
cd edge-messaging-platform
docker build -f kafka/dockerfile -t kafka .
```

##### 2 of 3

Run a local Kafka instance.

```shell
docker run -p 4222:4222 kafka
```

##### 3 of 3

Run the Kafka EMP.
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
  -e Kafka_SUBJECT_ROOT=subjects \
  -e Kafka_SUBJECT=* \
  -e Kafka_HOST=0.0.0.0:4222 \
  kafka-edge-messaging-platform
```

Visit the URL printed from the output.

Publish Kafka messages repeatedly.

```shell
while true;
    do (printf "PUB subjects.mydevice 5\r\nKNOCK\r\n"; sleep 0.4) | nc 0.0.0.0 4222;
done
```

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

#### Get Kafka

```shell
## Kafka Quick Install
docker run -p 4222:4222 kafka
```

Now you can run `cargo run --bin kafka`.
The EMP app is 12 factor and is configured via Environmental Variables.

```shell
PUBNUB_PUBLISH_KEY=pub-c-6b57a39e-79e7-4d1d-926e-5c376a4cb021 \
PUBNUB_SUBSCRIBE_KEY=sub-c-df3799ee-704b-11e9-8724-8269f6864ada \
PUBNUB_SECRET_KEY=sec-c-YWY3NzE0NTYtZTBkMS00YjJjLTgxZDQtN2YzOTY0NWNkNGVk \
PUBNUB_CHANNEL_ROOT=channels \
PUBNUB_CHANNEL=* \
PUBNUB_CIPHER_KEY=pAsSwOrD \
Kafka_SUBJECT_ROOT=subjects \
Kafka_SUBJECT=">" \
Kafka_HOST=0.0.0.0:4222 \
cargo run --bin kafka
```

## Reference Links

[https://hub.docker.com/kafka](https://hub.docker.com/_/kafka)
