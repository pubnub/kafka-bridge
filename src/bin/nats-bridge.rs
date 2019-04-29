#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use nats_bridge::nats::Client;
use std::{thread, time};

fn main() {
    // Send NATS Messages
    // Publish as fast as possible
    let nats_publisher_thread = thread::spawn(move || {
        let channel = "demo";
        let mut nats = Client::new("0.0.0.0:4222");
        let mut counter = 0;
        loop {
            counter += 1;
            nats.publish(channel, &format!("Hello {}", counter));
            thread::sleep(time::Duration::from_millis(300));
        }
    });

    // Receive NATS Messages
    // Subscribe as fast as possbile
    let nats_subscriber_thread = thread::spawn(move || {
        let mut nats = Client::new("0.0.0.0:4222");
        nats.subscribe("demo");
        let mut counter = 0;
        loop {
            let message = nats.next_message();
            if !message.ok {
                continue;
            }
            counter += 1;
            assert!(message.ok);
            println!(
                "[ {count} ] Channel:{channel} -> message:{message}",
                count = counter,
                channel = message.channel,
                message = message.data
            );
        }
    });

    nats_publisher_thread.join().expect("Error while joining thread");
    nats_subscriber_thread.join().expect("Error while joining thread");
}
