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
    let _ = pubnub.publish("demo".to_string(), "1234".to_string());

    let _result = nats::NATS::new(
        "0.0.0.0:4222".to_string(),
        "".to_string(),
        "".to_string(),
        "".to_string()
    );

    Ok(())
}
