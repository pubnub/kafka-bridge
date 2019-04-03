// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Libs
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
use failure::Fail;

extern crate hyper;
extern crate nats;

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// All the ways in which this app can fail
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
#[derive(Debug, Fail)]
pub enum AppError {
    #[fail(display = "Missing NATS_HOST ENVIRONMENTAL configuration")]
    MissingHost,

    #[fail(display = "Missing NATS_CHANNELS ENVIRONMENTAL configuration")]
    MissingChannels,

    #[fail(display = "NATS Error on command `{}`", _0)]
    NatsError(String, #[cause] nats::NatsError),
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Main
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
//fn main() -> Result<(), nats::NatsError> {
fn main() -> Result<(), AppError> {
    use self::AppError::{MissingHost, MissingChannels, NatsError};

    // Connection to NATS Cluster
    println!("Connecting to NATS");
    let host  = std::env::var("NATS_HOST").map_err( |_| MissingHost )?;
    let chans = std::env::var("NATS_CHANNELS").map_err( |_| MissingChannels )?;
    let url   = format!("nats://{}", host);
    let mut natsclient = nats::Client::new(url.clone())
        .map_err(|e|NatsError(url.clone(), e))?;
    println!("Connected to NATS");

    // TODO: Listen on configured channels
    let channel = format!("{}", "channel");
    natsclient.subscribe(&channel, None).map_err(|e|NatsError(channel, e))?;
    println!("Subscribed to NATS Channel");

    // Listen for New Messages
    loop {
        for _event in natsclient.events() {
            //...kj
            println!("NATS Message Received");
            // TODO: send to PubNub
        }
    }

    //Ok(())
}
