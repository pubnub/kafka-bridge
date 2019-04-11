// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Libs
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
mod pubnub;
mod nats;

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Main
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
fn main() -> Result<(), std::io::Error> {
    let mut pubnub = pubnub::PubNub::new(
        "psdsn.pubnub.com:80".to_string(),
        "demo".to_string(),
        "demo".to_string()
    ).unwrap();
    let _ = pubnub.publish("demo".to_string(), "123".to_string());

    let mut nats = nats::NATS::new(
        "0.0.0.0:4222".to_string(),
        "*".to_string(),
        "".to_string(),
        "".to_string(),
        "".to_string()
    ).unwrap();

    loop {
        let result = nats.listen();
        let message = result.unwrap();
        println!("{}", message);
    }

    Ok(())
}
