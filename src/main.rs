// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Imports
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
#[macro_use] extern crate json;

use std::time::{SystemTime, UNIX_EPOCH};
//use std::env;

mod nats;
mod pubnub;

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Main
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
fn main() {
    // PubNub SDK
    let mut pubnub = pubnub::PubNub::new(
        "psdsn.pubnub.com:80",
        "demo",
        "demo",
        "secret"
    ).unwrap();

    // NATS SDK
    let mut nats = nats::NATS::new(
        "0.0.0.0:4222",
        "demo",
        "",
        "",
        ""
    ).unwrap();

    // Sync NATS to PubNub
    loop {
        let message = nats.next_message().unwrap();
        let status = pubnub.publish(&message.channel, &message.data);

        let now = SystemTime::now();
        let epoch = now.duration_since(UNIX_EPOCH).unwrap().as_secs();

        println!("{}", json::stringify(object!{
            "sync" => status.is_ok(),
            "epoch" => epoch,
            "channel" => message.channel,
            "message" => message.data,
        }));
    }
}
