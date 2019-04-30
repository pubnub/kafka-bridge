#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use nats_bridge::nats;
use std::{thread, time};
use std::sync::mpsc;

fn main() {
    // Async Channels
    let (nats_message_tx, _pubnub_publish_rx) = mpsc::channel();

    // Send PubNub Messages
    // Publish as fast as possible
    /*
    let pubnub_publisher_thread = thread::spawn(move || {
        let mut pubnub = nats::Client::new("0.0.0.0:4222", "");
        loop {
            let message: nats::Message = pubnub_publish_rx.recv().expect("MPSC Channel receiver");
            let channel = message.channel;
            let data = message.data;
            pubnub.publish(&channel, &data);
        };
    } );
    */

    // Send NATS Messages
    // Publish as fast as possible
    let nats_publisher_thread = thread::spawn(move || {
        let channel = "demo";
        let mut nats = nats::PublishClient::new("0.0.0.0:4222").expect("NATS");
        let mut counter = 0;
        loop {
            counter += 1;
            nats.publish(channel, &format!("Hello {}", counter))
                .expect("message sent");
            thread::sleep(time::Duration::from_millis(300));
        };
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
            nats_message_tx.send(message).expect("NATS mpsc::channel channel write");
        };
    });

    //pubnub_publisher_thread.join().expect("Error while joining thread");
    nats_publisher_thread.join().expect("Error while joining thread");
    nats_subscriber_thread.join().expect("Error while joining thread");
}
