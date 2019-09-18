#![cfg_attr(feature = "nightly", feature(external_doc))]
#![cfg_attr(feature = "nightly", doc(include = "../readme.md"))]

pub mod nats;
pub mod kafka;
pub mod pubnub;
pub mod socket;
