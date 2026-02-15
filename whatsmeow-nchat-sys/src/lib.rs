#[allow(non_camel_case_types)]
#[allow(clippy::pub_underscore_fields)]
mod sys {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
pub use sys::*;

/// # Safety
/// `path` and `proxy` must be valid C Strings
#[allow(clippy::result_unit_err)]
pub unsafe fn create_connection(
    path: *mut ::std::os::raw::c_char,
    proxy: *mut ::std::os::raw::c_char,
    send_type: GoInt,
) -> Result<(ConnId, Receiver<Event>), ()> {
    let r = unsafe { CWmInit(path, proxy, send_type) };
    if r == -1 {
        Err(())
    } else {
        let id = ConnId(r as _);
        let (sender, receiver) = mpsc::unbounded_channel();
        add_sender(id, sender);
        Ok((id, receiver))
    }
}

use std::ffi::{CString, NulError};
use tokio::sync::mpsc::{self, UnboundedReceiver as Receiver};

mod events;
mod handlers;

pub use events::{ChatEvent, Event};
pub use handlers::{get_error, LogMsg};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnId(pub(crate) isize);

impl ConnId {
    /// Creates a connection id from a number.
    ///
    /// If you just want to get a connection,
    /// create it the proper way using `create_connection`.
    ///
    /// Be careful using invalid [`ConnId`]'s
    /// with API functions. It won't crash or anything,
    /// but you will get `Err`'s.
    #[must_use]
    pub fn from_inner(inner: isize) -> Self {
        Self(inner)
    }

    #[must_use]
    pub fn into_inner(self) -> isize {
        self.0
    }

    #[must_use]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Jid(pub String);
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
