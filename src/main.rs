// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Cargo Clippy
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
#![deny(clippy::all)]
#![deny(clippy::pedantic)]

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Imports
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
use json::object;
use std::sync::mpsc::channel;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
//use std::env;

mod socket;
mod nats;
mod pubnub;

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Main
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
fn main() -> thread::Result<()> {
    // PubNub SDK
    let mut pubnub = pubnub::PubNub::new("psdsn.pubnub.com:80", "demo", "demo", "secret").unwrap();

    // NATS SDK
    let mut nats = nats::NATS::new("0.0.0.0:4222", "demo", "", "", "");

    // Async Channels
    let (sender, receiver) = channel();

    // Receive NATS Messages
    let nats_thread = thread::spawn(move || loop {
        let message = nats.next_message().unwrap();
        sender.send(message).unwrap();
    });

    // Sync NATS to PubNub
    let pubnub_thread = thread::spawn(move || loop {
        let message = receiver.recv().unwrap();
        let status = pubnub.publish(&message.channel, &message.data);
        let now = SystemTime::now();
        let epoch = now.duration_since(UNIX_EPOCH).unwrap().as_secs();

        println!(
            "{}",
            json::stringify(object! {
                "sync" => status.is_ok(),
                "epoch" => epoch,
                "channel" => message.channel,
                "message" => message.data,
            })
        );
    });

    nats_thread.join()?;
    pubnub_thread.join()?;

    Ok(())
}
