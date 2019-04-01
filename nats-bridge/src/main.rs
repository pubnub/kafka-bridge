extern crate nats;
extern crate hyper;

fn main() {
    println!("Connecting NATS");

    let mut natsclient =
        nats::Client::new("nats://user:password@0.0.0.0:4444").unwrap();
    natsclient.subscribe( "channel", None ).unwrap();

    //println!("NATS Message Received");
    //let event  = natsclient.wait();

    for _event in natsclient.events() {
        //...kj
        println!("NATS Message Received");
    }
}
