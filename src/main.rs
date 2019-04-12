// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Libs
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
mod nats;
mod pubnub;

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Main
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
fn main() -> Result<(), std::io::Error> {
    let mut pubnub = pubnub::PubNub::new("psdsn.pubnub.com:80", "demo", "demo", "secret").unwrap();

    let mut nats = nats::NATS::new("0.0.0.0:4222", "demo", "", "", "").unwrap();

    loop {
        let message = nats.next_message().unwrap();

        println!("CHANNEL:{} MSG:{}", message.channel, message.data,);

        let jsonmsg = format!("\"{}\"", message.data);
        println!("PUBLISHING: {}", jsonmsg);
        let _ = pubnub.publish(&message.channel, &jsonmsg);
    }
}
