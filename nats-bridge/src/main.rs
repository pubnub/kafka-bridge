// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Libs
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
//extern crate actix_web;
extern crate nats;

use failure::Fail;
//use actix_rt::System;
//use actix_web::client::Client;

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
            //pubnub_sync(&channel, message: &str)
            // TODO: send to PubNub
        }
    }
}

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Maintain Connection to NATS and forward message to PubNub Sync
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// ...

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Sync Data to PubNub Data Stream Network
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
/*
fn pubnub_sync(channel: &str, message: &str) {
    System::new("pubnub_sync").block_on(lazy(|| {
       let mut client = Client::default();
       let domain = "pndsn.pubnub.com";
       let proto  = "https://"
       let channel
       // let uri

       client.get("https://pndsn.pubnub.com") // <- Create request builder
          .header("User-Agent", "Actix-web")
          .send()                             // <- Send http request
          .map_err(|_| ())
          .and_then(|response| {              // <- server http response
               println!("Response: {:?}", response);
               Ok(())
          })
    }));
}
*/

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Receive Configuration Changes
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// ...

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Load Configuration
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// ...
