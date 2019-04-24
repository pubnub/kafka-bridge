// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Cargo Clippy
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
#![deny(clippy::all)]
#![deny(clippy::pedantic)]

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
    let channel = "demo";
    let mut counter = 0;
    let mut nats = nats::NATS::new("0.0.0.0:4222", channel);

    loop {
        nats.publish(channel, "Hello");
        let message = nats.next_message();
        counter+=1;
        assert!(message.ok);
        println!(
            "[ {count} ] Channel:{channel} -> message:{message}",
            count=counter,
            channel=message.channel,
            message=message.data
        );
    }

    //let pubnub = pubnub::PubNub::new("psdsn.pubnub.com", "demo");
    //println!("{}", pubnub.channel);
}
