#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use kafka_bridge::kafka;
#[cfg(any(feature = "sasl-plain", feature = "sasl-ssl"))]
use kafka_bridge::kafka::SASLConfig;
use kafka_bridge::pubnub;
use std::{env, process, thread, time};
use tokio::sync::mpsc;
use tokio::time::{delay_for, Duration};

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Configuration via Environmental Variables
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
struct Configuration {
    pub kafka_brokers: Vec<String>,
    pub kafka_topic: String,
    pub kafka_group: String,
    pub pubnub_host: String,
    pub pubnub_channel: String,
    pub pubnub_channel_root: String,
    pub publish_key: String,
    pub subscribe_key: String,
    pub secret_key: String,
    #[cfg(any(feature = "sasl-plain", feature = "sasl-ssl"))]
    pub sasl_cfg: SASLConfig,
}

fn environment_variables() -> Configuration {
    Configuration {
        kafka_brokers: fetch_env_var("KAFKA_BROKERS")
            .split(',')
            .map(std::string::ToString::to_string)
            .collect(),
        kafka_topic: fetch_env_var("KAFKA_TOPIC"),
        kafka_group: fetch_env_var("KAFKA_GROUP"),
        pubnub_host: "psdsn.pubnub.com:80".into(),
        pubnub_channel: fetch_env_var("PUBNUB_CHANNEL"),
        pubnub_channel_root: fetch_env_var("PUBNUB_CHANNEL_ROOT"),
        publish_key: fetch_env_var("PUBNUB_PUBLISH_KEY"),
        subscribe_key: fetch_env_var("PUBNUB_SUBSCRIBE_KEY"),
        secret_key: fetch_env_var("PUBNUB_SECRET_KEY"),
        #[cfg(feature = "sasl-plain")]
        sasl_cfg: SASLConfig {
            username: fetch_env_var("SASL_USERNAME"),
            password: fetch_env_var("SASL_PASSWORD"),
        },
        #[cfg(feature = "sasl-ssl")]
        sasl_cfg: SASLConfig {
            username: fetch_env_var("SASL_USERNAME"),
            password: fetch_env_var("SASL_PASSWORD"),
            ca_location: fetch_env_var("SSL_CA_LOCATION"),
            certificate_location: fetch_env_var("SSL_CERTIFICATE_LOCATION"),
            key_location: fetch_env_var("SSL_KEY_LOCATION"),
            key_password: fetch_env_var("SSL_KEY_PASSWORD"),
        },
    }
}

impl std::fmt::Display for Configuration {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let channel = if self.pubnub_channel_root.is_empty() {
            self.pubnub_channel.to_string()
        } else {
            format!(
                "{root}.{channel}",
                channel = self.pubnub_channel,
                root = self.pubnub_channel_root
            )
        };

        let protocol = "https";
        let domain = "www.pubnub.com";
        let url = "/docs/console";
        write!(f,
            "{proto}://{domain}/{url}?channel={channel}&sub={sub}&pub={pub}",
            proto=protocol,
            domain=domain,
            url=url,
            channel=channel,
            sub=self.subscribe_key,
            pub=self.publish_key,
        )
    }
}

fn fetch_env_var(name: &str) -> String {
    if let Ok(value) = env::var(name) {
        value
    } else {
        eprintln!("Missing '{}' Environmental Variable", name);
        process::exit(1);
    }
}

// Receive messages from Kafka
// Consumes messages on Kafka topic and sends to MPSC PubNub Publisher
async fn run_async_kafka_consumer(
    kafka_message_tx: mpsc::Sender<kafka::Message>,
) {
    loop {
        let config = environment_variables();
        #[cfg(not(any(feature = "sasl-plain", feature = "sasl-ssl")))]
        let kafka = kafka::SubscribeClient::new(
            &config.kafka_brokers,
            kafka_message_tx.clone(),
            &config.kafka_topic,
            &config.kafka_group,
        );

        #[cfg(any(feature = "sasl-plain", feature = "sasl-ssl"))]
        let kafka = kafka::SubscribeClient::new_with_sasl(
            &config.kafka_brokers,
            kafka_message_tx.clone(),
            &config.kafka_topic,
            &config.kafka_group,
            &config.sasl_cfg,
        );

        let mut kafka = match kafka {
            Ok(kafka) => kafka,
            Err(error) => {
                println!("Retrying Consumer Connection {:?}", error);
                delay_for(Duration::from_millis(1000)).await;
                continue;
            }
        };

        // Send KAFKA Messages to pubnub_publish_rx via kafka_message_tx
        kafka.consume().await.expect("Consuming failed");
    }
}

// Send messages to Kafka
// Reads MPSC from PubNub Subscriptions and Sends to Kafka
async fn run_async_kafka_producer(
    kafka_publish_rx: mpsc::Receiver<pubnub::Message>,
) {
    let mut kafka_publish_rx = kafka_publish_rx;
    loop {
        let config = environment_variables();

        #[cfg(not(any(feature = "sasl-plain", feature = "sasl-ssl")))]
        let kafka = kafka::PublishClient::new(
            &config.kafka_brokers,
            &config.kafka_topic,
        );

        #[cfg(any(feature = "sasl-plain", feature = "sasl-ssl"))]
        let kafka = kafka::PublishClient::new_with_sasl(
            &config.kafka_brokers,
            &config.kafka_topic,
            &config.sasl_cfg,
        );

        let mut kafka = match kafka {
            Ok(kafka) => kafka,
            Err(_error) => {
                delay_for(Duration::from_millis(1000)).await;
                continue;
            }
        };

        loop {
            let message: kafka_bridge::pubnub::Message = kafka_publish_rx
                .recv()
                .await
                .expect("Async MPSC Channel receiver");
            match kafka.produce(&message.data).await {
                Ok(()) => {}
                Err(_error) => {
                    delay_for(Duration::from_millis(1000)).await;
                }
            };
        }
    }
}

// Send messages to PubNub
// Receives messages from MPSC from Kafka and Publishes to PubNub
async fn run_async_pubnub_publisher(
    pubnub_publish_rx: mpsc::Receiver<kafka::Message>,
) {
    let mut pubnub_publish_rx = pubnub_publish_rx;
    let config = environment_variables();
    let host = &config.pubnub_host;
    let root = &config.pubnub_channel_root;
    let publish_key = &config.publish_key;
    let subscribe_key = &config.subscribe_key;
    let secret_key = &config.secret_key;
    let agent = "kafka-bridge";

    loop {
        let mut pubnub = match pubnub::PublishClient::new(
            host,
            root,
            publish_key,
            subscribe_key,
            secret_key,
            agent,
        ) {
            Ok(pubnub) => pubnub,
            Err(_error) => {
                delay_for(Duration::from_millis(1000)).await;
                continue;
            }
        };

        // Message Receiver Loop
        loop {
            let message: kafka::Message = pubnub_publish_rx
                .recv()
                .await
                .expect("MPSC Channel Receiver");
            let channel = &message.topic;
            let data = &message.data;

            // Retry Loop on Failure
            loop {
                match pubnub.publish(channel, data) {
                    Ok(_timetoken) => break,
                    Err(_error) => {
                        delay_for(Duration::from_millis(1000)).await
                    }
                };
            }
        }
    }
}

#[tokio::main]
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Main Loop
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
async fn main() {
    // Async Channels
    let (kafka_message_tx, pubnub_publish_rx) = mpsc::channel(100);
    let (mut pubnub_message_tx, kafka_publish_rx) = mpsc::channel(010);

    let rt_handle = tokio::runtime::Handle::current();

    // Receive messages from PubNub
    // Saves messages into MPSC for Kafka Producer Thread
    let pubnub_subscriber_thread = thread::Builder::new()
        .name("PubNub Subscriber Thread".into())
        .spawn(move || loop {
            use kafka_bridge::pubnub;

            let config = environment_variables();
            let host = &config.pubnub_host;
            let root = &config.pubnub_channel_root;
            let channel = &config.pubnub_channel;
            let subscribe_key = &config.subscribe_key;
            let secret_key = &config.secret_key;
            let agent = "kafka-bridge";

            let mut pubnub = match pubnub::SubscribeClient::new(
                host,
                root,
                channel,
                subscribe_key,
                secret_key,
                agent,
            ) {
                Ok(pubnub) => pubnub,
                Err(_error) => {
                    thread::sleep(time::Duration::new(1, 0));
                    continue;
                }
            };

            loop {
                let message = match pubnub.next_message() {
                    Ok(message) => message,
                    Err(_error) => continue,
                };

                rt_handle
                    .block_on(pubnub_message_tx.send(message))
                    .map_err(|_| ())
                    .expect("KAFKA mpsc::channel channel write");
            }
        });

    // Print Follow-on Instructions
    let config = environment_variables();
    println!("{{\"info\":\"Dashboard: {}\"}}", config);

    tokio::join!(
        run_async_kafka_consumer(kafka_message_tx),
        run_async_kafka_producer(kafka_publish_rx),
        run_async_pubnub_publisher(pubnub_publish_rx)
    );

    // The Threads Gather
    pubnub_subscriber_thread
        .expect("PubNub Subscriber thread builder join handle")
        .join()
        .expect("Joining PubNub Subscriber Thread");
}
