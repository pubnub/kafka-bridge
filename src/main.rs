// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Cargo Clippy
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
#![deny(clippy::all)]
#![deny(clippy::pedantic)]

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Lib Imports
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
use std::{thread, time};

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Local Imports
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
mod socket;
//mod pubnub;
mod nats;

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Main
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
fn main() {

    // Send NATS Messages
    // Publish as fast as possible
    let nats_publisher_thread = thread::spawn(move || {
        let channel = "demo";
        let mut nats_publisher = nats::NATS::new("0.0.0.0:4222", channel);
        loop {
            nats_publisher.publish(channel, "Hello");
            thread::sleep(time::Duration::from_millis(300));
        }
    });

    // Receive NATS Messages
    // Subscribe as fast as possbile
    let nats_subscriber_thread = thread::spawn(move || {
        let channel = "demo";
        let mut counter = 0;
        let mut nats_subscriber = nats::NATS::new("0.0.0.0:4222", channel);
        loop {
            let message = nats_subscriber.next_message();
            if !message.ok { continue }
            counter += 1;
            assert!(message.ok);
            println!(
                "[ {count} ] Channel:{channel} -> message:{message}",
                count=counter,
                channel=message.channel,
                message=message.data
            );
        }
    });

    let _ = nats_publisher_thread.join();
    let _ = nats_subscriber_thread.join();
}
