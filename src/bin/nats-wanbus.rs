#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use json;
use std::sync::mpsc;
use std::{env, thread, time, process};
use wanbus::nats;

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Configuration via Environmental Variables
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
struct Configuration {
    pub pubnub_host: String,
    pub nats_host: String,
    pub nats_subject: String,
    pub pubnub_channel: String,
    pub pubnub_channel_root: String,
    pub publish_key: String,
    pub subscribe_key: String,
    pub _secret_key: String,
}
fn environment_variables() -> Configuration {
    Configuration {
        pubnub_host: "psdsn.pubnub.com:80".into(),
        nats_host: fetch_env_var("NATS_HOST"),
        nats_subject: fetch_env_var("NATS_SUBJECT"),
        pubnub_channel: fetch_env_var("PUBNUB_CHANNEL"),
        pubnub_channel_root: fetch_env_var("PUBNUB_CHANNEL_ROOT"),
        publish_key: fetch_env_var("PUBNUB_PUBLISH_KEY"),
        subscribe_key: fetch_env_var("PUBNUB_SUBSCRIBE_KEY"),
        _secret_key: fetch_env_var("PUBNUB_SECRET_KEY"),
    }
}
fn fetch_env_var(name: &str) -> String {
    match env::var(name) {
        Ok(value) => value,
        Err(_) => {
            eprintln!("Missing '{}' Environmental Variable", name);
            process::exit(1);
        },
    }
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Main Loop
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
fn main() {
    // Async Channels
    let (nats_message_tx, pubnub_publish_rx) = mpsc::channel();
    let (pubnub_message_tx, nats_publish_rx) = mpsc::channel();

    // Receive PubNub Messages
    // Subscribe to PubNub messages
    let pubnub_subscriber_thread = thread::Builder::new()
        .name("PubNub Subscriber Thread".into())
        .spawn(move || loop {
            use wanbus::pubnub;

            let config = environment_variables();
            let host = &config.pubnub_host;
            let root = &config.pubnub_channel_root;
            let channel = &config.pubnub_channel;
            let subscribe_key = &config.subscribe_key;
            let secret_key = &config._secret_key;

            let mut pubnub = match pubnub::SubscribeClient::new(
                host,
                root,
                channel,
                subscribe_key,
                secret_key,
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
                    .expect("NATS mpsc::channel channel write");
            }
        });

    // Send PubNub Messages
    // Publish as fast as possible
    let pubnub_publisher_thread = thread::Builder::new()
        .name("PubNub Publisher Thread".into())
        .spawn(move || loop {
            use wanbus::pubnub;

            let config = environment_variables();
            let host = &config.pubnub_host;
            let root = &config.pubnub_channel_root;
            let publish_key = &config.publish_key;
            let subscribe_key = &config.subscribe_key;
            let secret_key = &config._secret_key;

            let mut pubnub = match pubnub::PublishClient::new(
                host,
                root,
                publish_key,
                subscribe_key,
                secret_key,
            ) {
                Ok(pubnub) => pubnub,
                Err(_error) => {
                    thread::sleep(time::Duration::new(1, 0));
                    continue;
                }
            };

            // Message Receiver Loop
            loop {
                let message: nats::Message =
                    pubnub_publish_rx.recv().expect("MPSC Channel Receiver");
                let channel = &message.channel;
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

    // Send NATS Messages
    // Publish as fast as possible
    let nats_publisher_thread = thread::Builder::new()
        .name("NATS Publisher Thread".into())
        .spawn(move || loop {
            let config = environment_variables();
            let host = &config.nats_host;

            let mut nats = match nats::PublishClient::new(host) {
                Ok(nats) => nats,
                Err(_error) => {
                    thread::sleep(time::Duration::from_millis(1000));
                    continue;
                }
            };

            loop {
                let message: wanbus::pubnub::Message =
                    nats_publish_rx.recv().expect("MPSC Channel Receiver");
                match nats.publish(message.channel, message.data) {
                    Ok(()) => {}
                    Err(_error) => {
                        thread::sleep(time::Duration::from_millis(1000));
                    }
                };
            }
        });

    // Receive NATS Messages
    // Subscribe as fast as possbile
    let nats_subscriber_thread = thread::Builder::new()
        .name("NATS Subscriber Thread".into())
        .spawn(move || loop {
            let config = environment_variables();
            let host = &config.nats_host;
            let subject = &config.nats_subject;
            let mut nats = match nats::SubscribeClient::new(host, subject) {
                Ok(nats) => nats,
                Err(_error) => {
                    thread::sleep(time::Duration::from_millis(1000));
                    continue;
                }
            };
            loop {
                // Get NATS Messages
                let mut message = match nats.next_message() {
                    Ok(message) => message,
                    Err(_error) => continue,
                };

                // Convert to JSON String if not already JSON
                let parsetest = json::parse(&message.data);
                if parsetest.is_err() {
                    message.data = json::stringify(message.data);
                }

                // Enqueue message to be placed on the WAN
                nats_message_tx
                    .send(message)
                    .expect("NATS mpsc::channel channel write");
            }
        });

    // The Threads Gather
    pubnub_subscriber_thread
        .expect("PubNub Subscriber thread builder join handle")
        .join()
        .expect("Joining PubNub Subscriber Thread");
    pubnub_publisher_thread
        .expect("PubNub Publisher thread builder join handle")
        .join()
        .expect("Joining PubNub Publisher Thread");
    nats_publisher_thread
        .expect("NATS Publisher thread builder join handle")
        .join()
        .expect("Joining NATS Publisher Thread");
    nats_subscriber_thread
        .expect("NATS Subscriber thread builder join handle")
        .join()
        .expect("Joining NATS Subscriber Thread");
}
