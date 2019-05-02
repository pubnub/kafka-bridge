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
    let pubnub_subscriber_thread = thread::spawn(move || {
        use nats_bridge::pubnub;
        let host = "psdsn.pubnub.com:80";
        let channel = "demo";
        let publish_key = "demo";
        let subscribe_key = "demo";
        let secret_key = "secret";
        let mut pubnub = pubnub::Client::new(
            host,
            channel,
            publish_key,
            subscribe_key,
            secret_key,
        )
        .expect("PubNub Client");
        loop {
            let message = match pubnub.next_message() {
                Ok(message) => message,
                Err(_error) => continue,
            };
            println!("SUBSCRIBED>>>>>>>>: {}",message.data);
            pubnub_message_tx
                .send(message)
                .expect("NATS mpsc::channel channel write");
        }
    });

    // Send PubNub Messages
    // Publish as fast as possible
    let pubnub_publisher_thread = thread::spawn(move || {
        use nats_bridge::pubnub;
        let host = "psdsn.pubnub.com:80";
        let channel = "";
        let publish_key = "demo";
        let subscribe_key = "demo";
        let secret_key = "secret";
        let mut pubnub = pubnub::Client::new(
            host,
            channel,
            publish_key,
            subscribe_key,
            secret_key,
        )
        .expect("PubNub Client");
        loop {
            let message: nats::Message =
                pubnub_publish_rx.recv().expect("MPSC Channel Receiver");
            let channel = &message.channel;
            let data = &message.data;

            loop {
                match pubnub.publish(channel, data) {
                    Ok(timetoken) => {
                        println!("MessageID: {}", timetoken);
                        break;
                    }
                    Err(_error) => thread::sleep(time::Duration::new(1, 0)),
                };
            }
        }
    });

    // Send NATS Messages
    // Publish as fast as possible
    let nats_publisher_thread = thread::spawn(move || {
        let mut nats =
            nats::PublishClient::new("0.0.0.0:4222").expect("NATS");
        loop {
            let message: nats_bridge::pubnub::Message =
                nats_publish_rx.recv().expect("MPSC Channel Receiver");
            nats.publish(message.channel, message.data)
                .expect("message sent");
            thread::sleep(time::Duration::from_millis(1000));
        }
    });

    // Receive NATS Messages
    // Subscribe as fast as possbile
    let nats_subscriber_thread = thread::spawn(move || {
        let channel = "demo";
        let mut nats = nats::SubscribeClient::new("0.0.0.0:4222", channel)
            .expect("NATS Subscribe Client");
        let mut counter = 0;
        loop {
            let message = match nats.next_message() {
                Ok(message) => message,
                Err(_error) => continue,
            };
            counter += 1;
            println!(
                "[ {count} ] Channel:{channel} -> message:{message}",
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
        .join()
        .expect("Error while joining thread");
    pubnub_publisher_thread
        .join()
        .expect("Error while joining thread");
    nats_publisher_thread
        .join()
        .expect("Error while joining thread");
    nats_subscriber_thread
        .join()
        .expect("Error while joining thread");
}
