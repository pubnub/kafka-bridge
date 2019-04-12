// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Libs
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
mod nats;
mod pubnub;

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Main
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
fn main() -> Result<(), std::io::Error> {
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
        //let jsonmsg = format!("\"{}\"", message.data);
        let _status = pubnub.publish(&message.channel, &message.data);
    }
}
