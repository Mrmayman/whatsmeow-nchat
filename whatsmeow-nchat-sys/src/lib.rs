#![allow(non_camel_case_types)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub unsafe fn connect(
    path: *mut ::std::os::raw::c_char,
    proxy: *mut ::std::os::raw::c_char,
    send_type: GoInt,
) -> Result<(ConnId, Receiver<SentEvent>), ()> {
    let r = unsafe { CWmInit(path, proxy, send_type) };
    if r == -1 {
        Err(())
    } else {
        let id = ConnId(r as _);
        let (sender, receiver) = std::sync::mpsc::channel();
        add_sender(id, sender);
        Ok((id, receiver))
    }
}

use std::{
    ffi::{CString, NulError},
    sync::mpsc::Receiver,
};

mod events;
mod handlers;

pub use events::{ChatEvent, Event, SentEvent};
pub use handlers::{LogMsg, LogState, LOG_STATE};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnId(pub(crate) isize);

impl ConnId {
    pub fn raw(self) -> GoInt {
        self.0 as _
    }
}

/// JIDs are unique identifiers for chats/users on WhatsApp.
///
/// They come in the format `SOME_NUMBER@SERVER`, where
/// `SOME_NUMBER` is phone number for normal users.
/// I don't know what it is for others.
///
/// `SERVER` can be:
/// ```txt
/// DefaultUserServer = "s.whatsapp.net"
/// GroupServer       = "g.us"
/// LegacyUserServer  = "c.us"
/// BroadcastServer   = "broadcast"
/// HiddenUserServer  = "lid"
/// MessengerServer   = "msgr"
/// InteropServer     = "interop"
/// NewsletterServer  = "newsletter"
/// HostedServer      = "hosted"
/// HostedLIDServer   = "hosted.lid"
/// BotServer         = "bot"
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Jid(pub String);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MsgId(pub String);

impl TryInto<CString> for &Jid {
    type Error = NulError;
    fn try_into(self) -> Result<CString, NulError> {
        CString::new(self.0.as_str())
    }
}
impl TryInto<CString> for &MsgId {
    type Error = NulError;
    fn try_into(self) -> Result<CString, NulError> {
        CString::new(self.0.as_str())
    }
}

use bitflags::bitflags;

use crate::events::add_sender;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadFileAction {
    None,
    Open,
    Save,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadFileStatus {
    None = -1,
    NotDownloaded = 0,
    Downloaded = 1,
    Downloading = 2,
    DownloadFailed = 3,
}
