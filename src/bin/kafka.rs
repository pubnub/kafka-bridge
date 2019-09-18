#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use json;
use std::sync::mpsc;
use std::{env, process, thread, time};
use edge_messaging_platform::kafka;

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Configuration via Environmental Variables
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
struct Configuration {
    pub kafka_host: String,
    pub kafka_topic: String,
    pub kafka_topic_root: String,
    pub pubnub_host: String,
    pub pubnub_channel: String,
    pub pubnub_channel_root: String,
    pub publish_key: String,
    pub subscribe_key: String,
    pub secret_key: String,
}

fn environment_variables() -> Configuration {
    Configuration {
        kafka_host: fetch_env_var("KAFKA_HOST"),
        kafka_topic: fetch_env_var("KAFKA_TOPIC"),
        kafka_topic_root: fetch_env_var("KAFKA_TOPIC_ROOT"),
        pubnub_host: "psdsn.pubnub.com:80".into(),
        pubnub_channel: fetch_env_var("PUBNUB_CHANNEL"),
        pubnub_channel_root: fetch_env_var("PUBNUB_CHANNEL_ROOT"),
        publish_key: fetch_env_var("PUBNUB_PUBLISH_KEY"),
        subscribe_key: fetch_env_var("PUBNUB_SUBSCRIBE_KEY"),
        secret_key: fetch_env_var("PUBNUB_SECRET_KEY"),
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

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Main Loop
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
fn main() {
    // Async Channels
    let (kafka_message_tx, pubnub_publish_rx) = mpsc::channel();
    let (pubnub_message_tx, kafka_publish_rx) = mpsc::channel();

    // Receive PubNub Messages
    // Subscribe to PubNub messages
    let pubnub_subscriber_thread = thread::Builder::new()
        .name("PubNub Subscriber Thread".into())
        .spawn(move || loop {
            use edge_messaging_platform::pubnub;

            let config = environment_variables();
            let host = &config.pubnub_host;
            let root = &config.pubnub_channel_root;
            let channel = &config.pubnub_channel;
            let subscribe_key = &config.subscribe_key;
            let secret_key = &config.secret_key;
            let agent = "kafka-edge_messaging_platform";

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
                pubnub_message_tx
                    .send(message)
                    .expect("KAFKA mpsc::channel channel write");
            }
        });

    // Send PubNub Messages
    // Publish as fast as possible
    let pubnub_publisher_thread = thread::Builder::new()
        .name("PubNub Publisher Thread".into())
        .spawn(move || loop {
            use edge_messaging_platform::pubnub;

            let config = environment_variables();
            let host = &config.pubnub_host;
            let root = &config.pubnub_channel_root;
            let publish_key = &config.publish_key;
            let subscribe_key = &config.subscribe_key;
            let secret_key = &config.secret_key;
            let agent = "kafka-edge_messaging_platform";

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
                    thread::sleep(time::Duration::new(1, 0));
                    continue;
                }
            };

            // Message Receiver Loop
            loop {
                let message: kafka::Message =
                    pubnub_publish_rx.recv().expect("MPSC Channel Receiver");
                let channel = &message.topic;
                let data = &message.data;

                // Retry Loop on Failure
                loop {
                    match pubnub.publish(channel, data) {
                        Ok(_timetoken) => break,
                        Err(_error) => {
                            thread::sleep(time::Duration::new(1, 0))
                        }
                    };
                }
            }
        });

    // Send KAFKA Messages
    // Publish as fast as possible
    let kafka_publisher_thread = thread::Builder::new()
        .name("KAFKA Publisher Thread".into())
        .spawn(move || loop {
            let config = environment_variables();
            let host = &config.kafka_host;
            let root = &config.kafka_topic_root;

            let mut kafka = match kafka::PublishClient::new(host, root) {
                Ok(kafka) => kafka,
                Err(_error) => {
                    thread::sleep(time::Duration::from_millis(1000));
                    continue;
                }
            };

            loop {
                let message: edge_messaging_platform::pubnub::Message =
                    kafka_publish_rx.recv().expect("MPSC Channel Receiver");
                match kafka.publish(message.channel, message.data) {
                    Ok(()) => {}
                    Err(_error) => {
                        thread::sleep(time::Duration::from_millis(1000));
                    }
                };
            }
        });

    // Receive KAFKA Messages
    // Subscribe as fast as possbile
    let kafka_subscriber_thread = thread::Builder::new()
        .name("KAFKA Subscriber Thread".into())
        .spawn(move || loop {
            let config = environment_variables();
            let host = &config.kafka_host;
            let root = &config.kafka_topic_root;
            let topic = &config.kafka_topic;
            let mut kafka =
                match kafka::SubscribeClient::new(host, root, topic) {
                    Ok(kafka) => kafka,
                    Err(_error) => {
                        thread::sleep(time::Duration::from_millis(1000));
                        continue;
                    }
                };
            loop {
                // Get KAFKA Messages
                let mut message = match kafka.next_message() {
                    Ok(message) => message,
                    Err(_error) => continue,
                };

                // Convert to JSON String if not already JSON
                let parsetest = json::parse(&message.data);
                if parsetest.is_err() {
                    message.data = json::stringify(message.data);
                }

                // Enqueue message to be placed on the WAN
                kafka_message_tx
                    .send(message)
                    .expect("KAFKA mpsc::channel topic write");
            }
        });

    // Print Follow-on Instructions
    let config = environment_variables();
    println!("{{\"info\":\"Dashboard: {}\"}}", config);

    // The Threads Gather
    pubnub_subscriber_thread
        .expect("PubNub Subscriber thread builder join handle")
        .join()
        .expect("Joining PubNub Subscriber Thread");
    pubnub_publisher_thread
        .expect("PubNub Publisher thread builder join handle")
        .join()
        .expect("Joining PubNub Publisher Thread");
    kafka_publisher_thread
        .expect("KAFKA Publisher thread builder join handle")
        .join()
        .expect("Joining KAFKA Publisher Thread");
    kafka_subscriber_thread
        .expect("KAFKA Subscriber thread builder join handle")
        .join()
        .expect("Joining KAFKA Subscriber Thread");
}
