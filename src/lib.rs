#![cfg_attr(feature = "nightly", feature(external_doc))]
#![cfg_attr(feature = "nightly", doc(include = "../readme.md"))]

pub mod nats;
pub mod pubnub;
mod socket;
