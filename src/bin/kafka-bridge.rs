#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use futures::executor::block_on;
use kafka_bridge::kafka;
use kafka_bridge::pubnub;
use std::{env, process};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::{delay_for, Duration};

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Configuration via Environmental Variables
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
struct Configuration {
    pub kafka_brokers: Vec<String>,
    pub kafka_topic: String,
    pub kafka_group: String,
    pub kafka_partition: i32,
    pub pubnub_host: String,
    pub pubnub_channel: String,
    pub pubnub_channel_root: String,
    pub publish_key: String,
    pub subscribe_key: String,
    pub secret_key: String,
    #[cfg(any(feature = "sasl-plain", feature = "sasl-ssl"))]
    pub sasl_username: String,
    #[cfg(any(feature = "sasl-plain", feature = "sasl-ssl"))]
    pub sasl_password: String,
    #[cfg(any(feature = "sasl-ssl", feature = "sasl-gssapi"))]
    pub ssl_ca_location: String,
    #[cfg(any(feature = "sasl-ssl", feature = "sasl-gssapi"))]
    pub ssl_certificate_location: String,
    #[cfg(any(feature = "sasl-ssl", feature = "sasl-gssapi"))]
    pub ssl_key_location: String,
    #[cfg(any(feature = "sasl-ssl", feature = "sasl-gssapi"))]
    pub ssl_key_password: String,
    #[cfg(feature = "sasl-gssapi")]
    pub kerberos_service_name: String,
    #[cfg(feature = "sasl-gssapi")]
    pub kerberos_keytab: String,
    #[cfg(feature = "sasl-gssapi")]
    pub kerberos_principal: String,
}

fn environment_variables() -> Configuration {
    Configuration {
        kafka_brokers: fetch_env_var("KAFKA_BROKERS")
            .split(',')
            .map(ToString::to_string)
            .collect(),
        kafka_topic: fetch_env_var("KAFKA_TOPIC"),
        kafka_group: fetch_env_var("KAFKA_GROUP"),
        kafka_partition: fetch_env_var("KAFKA_PARTITION")
            .parse::<i32>()
            .unwrap(),
        pubnub_host: "psdsn.pubnub.com:80".into(),
        pubnub_channel: fetch_env_var("PUBNUB_CHANNEL"),
        pubnub_channel_root: fetch_env_var("PUBNUB_CHANNEL_ROOT"),
        publish_key: fetch_env_var("PUBNUB_PUBLISH_KEY"),
        subscribe_key: fetch_env_var("PUBNUB_SUBSCRIBE_KEY"),
        secret_key: fetch_env_var("PUBNUB_SECRET_KEY"),
        #[cfg(any(feature = "sasl-plain", feature = "sasl-ssl"))]
        sasl_username: fetch_env_var("SASL_USERNAME"),
        #[cfg(any(feature = "sasl-plain", feature = "sasl-ssl"))]
        sasl_password: fetch_env_var("SASL_PASSWORD"),
        #[cfg(any(feature = "sasl-ssl", feature = "sasl-gssapi"))]
        ssl_ca_location: fetch_env_var("SSL_CA_LOCATION"),
        #[cfg(any(feature = "sasl-ssl", feature = "sasl-gssapi"))]
        ssl_certificate_location: fetch_env_var("SSL_CERTIFICATE_LOCATION"),
        #[cfg(any(feature = "sasl-ssl", feature = "sasl-gssapi"))]
        ssl_key_location: fetch_env_var("SSL_KEY_LOCATION"),
        #[cfg(any(feature = "sasl-ssl", feature = "sasl-gssapi"))]
        ssl_key_password: fetch_env_var("SSL_KEY_PASSWORD"),
        #[cfg(feature = "sasl-gssapi")]
        kerberos_service_name: fetch_env_var("KERBEROS_SERVICE_NAME"),
        #[cfg(feature = "sasl-gssapi")]
        kerberos_keytab: fetch_env_var("KERBEROS_KEYTAB"),
        #[cfg(feature = "sasl-gssapi")]
        kerberos_principal: fetch_env_var("KERBEROS_PRINCIPAL"),
    }
}

impl From<&Configuration> for kafka::ClientConfig {
    fn from(config: &Configuration) -> Self {
        #[cfg(not(any(
            feature = "sasl-plain",
            feature = "sasl-ssl",
            feature = "sasl-gssapi"
        )))]
        {
            let _ = config;
            Self::Plain
        }
        #[cfg(feature = "sasl-plain")]
        {
            Self::SaslPlain {
                username: config.sasl_username.clone(),
                password: config.sasl_password.clone(),
            }
        }
        #[cfg(feature = "sasl-ssl")]
        {
            Self::SaslSsl {
                username: config.sasl_username.clone(),
                password: config.sasl_password.clone(),
                ca_location: config.ssl_ca_location.clone(),
                certificate_location: config.ssl_certificate_location.clone(),
                key_location: config.ssl_key_location.clone(),
                key_password: config.ssl_key_password.clone(),
            }
        }
        #[cfg(feature = "sasl-gssapi")]
        {
            Self::SaslGssapi {
                kerberos_service_name: config.kerberos_service_name.clone(),
                kerberos_keytab: config.kerberos_keytab.clone(),
                kerberos_principal: config.kerberos_principal.clone(),
                ca_location: config.ssl_ca_location.clone(),
                certificate_location: config.ssl_certificate_location.clone(),
                key_location: config.ssl_key_location.clone(),
                key_password: config.ssl_key_password.clone(),
            }
        }
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
        let domain   = "www.pubnub.com";
        let url      = "docs/console";
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

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Main Loop
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
#[allow(unused_must_use)] // generated inside `tokio::join!`
#[tokio::main]
async fn main() {
    // Async Channels
    let (kafka_message_tx, pubnub_publish_rx) = mpsc::channel(100);
    let (pubnub_message_tx, kafka_publish_rx) = mpsc::channel(100);

    let pubnub_subscriber = spawn_pubnub_subscriber(pubnub_message_tx);
    let pubnub_publisher = spawn_pubnub_publisher(pubnub_publish_rx);
    let kafka_publisher = spawn_kafka_publisher(kafka_publish_rx);
    let kafka_subscriber = spawn_kafka_subscriber(kafka_message_tx);

    // Print Follow-on Instructions
    let config = environment_variables();
    println!("{{\"info\":\"Dashboard: {}\"}}", config);

    // The Threads Gather
    tokio::join!(
        pubnub_subscriber,
        pubnub_publisher,
        kafka_publisher,
        kafka_subscriber,
    );
}

/// Receives messages from PubNub.
///
/// Saves messages into MPSC for Kafka Producer Thread
fn spawn_pubnub_subscriber(
    mut pubnub_message_tx: mpsc::Sender<pubnub::Message>,
) -> JoinHandle<()> {
    tokio::task::spawn_blocking(move || loop {
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
                block_on(delay_for(Duration::new(1, 0)));
                continue;
            }
        };

        loop {
            let message = match pubnub.next_message() {
                Ok(message) => {
                    message
                }
                Err(error) => {
                    continue;
                }
            };

            println!("{chan} -> {msg}", chan=message.channel, msg=message.data);
            if let Err(err) = block_on(pubnub_message_tx.send(message)) {
                panic!("KAFKA mpsc::channel channel write: {}", err);
            }
        }
    })
}

/// Sends messages to PubNub.
///
/// Receives messages from MPSC from Kafka and Publishes to PubNub.
fn spawn_pubnub_publisher(
    mut pubnub_publish_rx: mpsc::Receiver<kafka::Message>,
) -> JoinHandle<()> {
    tokio::task::spawn_blocking(move || loop {
        let config = environment_variables();
        let host = &config.pubnub_host;
        let root = &config.pubnub_channel_root;
        let publish_key = &config.publish_key;
        let subscribe_key = &config.subscribe_key;
        let secret_key = &config.secret_key;
        let agent = "kafka-bridge";

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
                block_on(delay_for(Duration::new(1, 0)));
                continue;
            }
        };

        // Message Receiver Loop
        loop {
            let message: kafka::Message = block_on(pubnub_publish_rx.recv())
                .expect("MPSC Channel Receiver");
            let channel = &message.topic;
            let data = &message.data;

            // Retry Loop on Failure
            loop {
                match pubnub.publish(channel, data) {
                    Ok(_timetoken) => break,
                    Err(_error) => block_on(delay_for(Duration::new(1, 0))),
                };
            }
        }
    })
}

/// Sends messages to Kafka.
///
/// Reads MPSC from PubNub Subscriptions and Sends to Kafka.
fn spawn_kafka_publisher(
    mut kafka_publish_rx: mpsc::Receiver<pubnub::Message>,
) -> JoinHandle<()> {
    tokio::spawn(async move {

        loop {
            let config = &environment_variables();

            let res = kafka::PublishClient::new(
                config.into(),
                &config.kafka_brokers,
                &config.kafka_topic,
            );

            let mut kafka = match res {
                Ok(kafka) => kafka,
                Err(error) => {
                    println!("Error connecting to Kafka: {:?}", error);
                    delay_for(Duration::from_millis(1000)).await;
                    continue;
                }
            };

            loop {
                let message: kafka_bridge::pubnub::Message = kafka_publish_rx
                    .recv()
                    .await
                    .expect("MPSC Channel Receiver");

                println!("Sendning message to Kafka Stream: {message}", message=&message.data);
                match kafka.produce(&message.data).await {
                    Ok(()) => {}
                    Err(error) => {
                        println!("Error sending to Kafka: {:?}", error);
                        delay_for(Duration::from_millis(1000)).await;
                    }
                };
            }
        }
    })
}

/// Receives messages from Kafka.
///
/// Consumes messages on Kafka topic and sends to MPSC PubNub Publisher.
fn spawn_kafka_subscriber(
    kafka_message_tx: mpsc::Sender<kafka::Message>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            let config = &environment_variables();

            let res = kafka::SubscribeClient::new(
                config.into(),
                &config.kafka_brokers,
                kafka_message_tx.clone(),
                &config.kafka_topic,
                &config.kafka_group,
                config.kafka_partition,
            );

            let mut kafka = match res {
                Ok(kafka) => kafka,
                Err(error) => {
                    println!("Retrying Consumer Connection {:?}", error);
                    delay_for(Duration::from_millis(1000)).await;
                    continue;
                }
            };

            // Send KAFKA Messages to pubnub_publish_rx via kafka_message_tx
            kafka
                .consume()
                .await
                .expect("Error consuming Kafka messages");
        }
    })
}
