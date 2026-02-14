#![allow(non_camel_case_types)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

mod handlers;
pub use handlers::{ChatEvent, Event, LogMsg, LogState, LOG_STATE, SENDER};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnId(pub isize);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChatId(pub String);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MsgId(pub String);

use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct StatusFlags: isize {
        const None = 0;
        const Offline = (1 << 0);
        const Connecting = (1 << 1);
        const Online = (1 << 2);
        const Fetching = (1 << 3);
        const Sending = (1 << 4);
        const Updating = (1 << 5);
        const Syncing = (1 << 6);
        const Away = (1 << 7);
    }
}
