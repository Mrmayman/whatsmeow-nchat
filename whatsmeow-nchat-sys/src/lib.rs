#![allow(non_camel_case_types)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

mod handlers;
pub use handlers::{ChatEvent, ConnId, Event, SENDER};
