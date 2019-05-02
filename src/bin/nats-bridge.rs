#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use nats_bridge::nats;
use std::sync::mpsc;
use std::{thread, time};

fn main() {
    // Async Channels
    let (nats_message_tx, pubnub_publish_rx) = mpsc::channel();
    let (pubnub_message_tx, nats_publish_rx) = mpsc::channel();

    // Receive PubNub Messages
    // Subscribe to PubNub messages
    let pubnub_subscriber_thread = thread::Builder::new()
        .name("PubNub Subscriber Thread".into())
        .spawn(move || loop {
            use nats_bridge::pubnub;
            let host = "psdsn.pubnub.com:80";
            let channel = "my_channel";
            let publish_key = "demo";
            let subscribe_key = "demo";
            let secret_key = "secret";
            let mut pubnub = match pubnub::Client::new(
                host,
                channel,
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

            loop {
                let message = match pubnub.next_message() {
                    Ok(message) => message,
                    Err(_error) => continue,
                };
                println!("SUBSCRIBED>>>>>>>>: {}", message.data);
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
            use nats_bridge::pubnub;
            let host = "psdsn.pubnub.com:80";
            let channel = "";
            let publish_key = "demo";
            let subscribe_key = "demo";
            let secret_key = "secret";
            let mut pubnub = match pubnub::Client::new(
                host,
                channel,
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

                println!(
                    "SENDING TO PUBNUB: --> {} -> {}",
                    message.channel, message.data
                );

                // Retry Loop on Failure
                loop {
                    match pubnub.publish(channel, data) {
                        Ok(timetoken) => {
                            println!("MessageID: {}", timetoken);
                            break;
                        }
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
            let host = "0.0.0.0:4222";
            let mut nats = match nats::PublishClient::new(host) {
                Ok(nats) => nats,
                Err(_error) => {
                    thread::sleep(time::Duration::from_millis(1000));
                    continue;
                }
            };

            loop {
                let message: nats_bridge::pubnub::Message =
                    nats_publish_rx.recv().expect("MPSC Channel Receiver");
                //let jsonmsg =
                println!(
                    "SENDING TO NATS: --> {} -> {}",
                    message.channel, message.data
                );
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
            let host = "0.0.0.0:4222";
            let channel = "my_channel";
            let mut counter = 0;
            let mut nats = match nats::SubscribeClient::new(host, channel) {
                Ok(nats) => nats,
                Err(_error) => {
                    thread::sleep(time::Duration::from_millis(1000));
                    continue;
                }
            };
            loop {
                let message = match nats.next_message() {
                    Ok(message) => message,
                    Err(_error) => continue,
                };
                counter += 1;
                println!(
                    " - [ {count} ] Channel:{channel} -> message:{message}",
                    count = counter,
                    channel = message.channel,
                    message = message.data
                );
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
