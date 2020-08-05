#![cfg_attr(feature = "nightly", feature(external_doc))]
#![cfg_attr(feature = "nightly", doc(include = "../readme.md"))]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]

pub mod kafka;
pub mod pubnub;
pub mod socket;
