extern crate nats;
extern crate hyper;

fn main() {
    println!("Connecting NATS");

    // Craete connection to NATS Cluster
    let mut natsclient =
        nats::Client::new("nats://user:password@0.0.0.0:4444").unwrap();

    // TODO: Listen on configured channels
    natsclient.subscribe( "channel", None ).unwrap();

    // Listen for New Messages
    for _event in natsclient.events() {
        //...kj
        println!("NATS Message Received");
        // TODO: send to PubNub
    }
}
