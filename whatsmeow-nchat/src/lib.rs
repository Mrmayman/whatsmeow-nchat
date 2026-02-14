//! Higher level, (relatively) safe bindings to the
//! Go `whatsmeow` library.
//!
//! - You create connections using [`create_connection`],
//!   each representing  a different WhatsApp profile or account.
//!   - You can have multiple running concurrently, just pass the right id.
//! - Then you operate on them using the other functions (eg. to send messages).
//! - You periodically poll and receive events using the `Receiver`
//!   you got from [`create_connection`]. If supported you could hook this up
//!   to your UI (eg: `iced::Task::sip`).
//!
//! # Safety
//! - These functions are implemented in memory-safe Go
//! - The Go implementation uses Mutexes, so this is thread-safe
//!
//! # TODO
//! - Use different senders for different connections to avoid confusion
//! - Have system to prevent using connections after cleaning them up

pub use sys::{ChatEvent, ChatId, ConnId, Event, MsgId, StatusFlags};
use whatsmeow_nchat_sys as sys;

use std::{
    ffi::{c_char, CStr, CString},
    fmt::Display,
    path::Path,
    sync::mpsc::Receiver,
};

mod error;
use error::{attempt, get_error};
pub use error::{Result, WhatsmeowError};

static EMPTY: &CStr = c"";

/// Initializes a connection. The first thing to do on startup!
pub fn create_connection(
    path: impl AsRef<Path>,
    proxy: &str,
    send_type: isize,
) -> Result<(ConnId, Receiver<(ConnId, Event)>)> {
    let path = CString::new(path.as_ref().to_string_lossy().to_string())?;
    let proxy = CString::new(proxy)?;

    let (send, recv) = std::sync::mpsc::channel();
    *sys::SENDER.write().map_err(|_| WhatsmeowError::Poison)? = Some(send);

    let conn_id = unsafe {
        sys::CWmInit(
            path.as_ptr().cast_mut(),
            proxy.as_ptr().cast_mut(),
            send_type as _,
        )
    };
    if conn_id == -1 {
        Err(get_error())
    } else {
        Ok((ConnId(conn_id as isize), recv))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountState {
    None,
    Connecting,
    Connected,
    Disconnected,
    Outdated,
}

impl AccountState {
    /// Gets the current state of the connection `id`
    pub fn get(id: ConnId) -> Self {
        unsafe {
            match sys::CWmExtGetState(id.0 as _) {
                1 => Self::Connecting,
                2 => Self::Connected,
                3 => Self::Disconnected,
                4 => Self::Outdated,
                _ => Self::None,
            }
        }
    }
}

/// Logs into an account with the connection.
/// Call this only if you haven't logged in yet.
///
/// Use [`AccountState::get`] to check if you're logged in.
pub fn login(id: ConnId) -> Result<()> {
    attempt(unsafe { sys::CWmLogin(id.0 as _) })
}

/// Logs out of the account linked in the connection.
pub fn logout(id: ConnId) -> Result<()> {
    attempt(unsafe { sys::CWmLogout(id.0 as _) })
}

// TODO: find out what this is
pub fn cleanup(id: ConnId) -> Result<()> {
    attempt(unsafe { sys::CWmCleanup(id.0 as _) })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Audio,
    Video,
    Image,
    Document,
}

impl Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                FileType::Audio => "audio",
                FileType::Video => "video",
                FileType::Image => "image",
                FileType::Document => "document",
            }
        )
    }
}

/// Sends/edits a message with the given parameters.
///
/// - To edit the message instead of sending it,
///   use `edit_msg` argument (Id and timestamp of message being edited)
/// - To attach a file, use `file` argument
/// - To reply to a quoted message, use `quoted_*` arguments (TODO)
///   - `quoted_id`: TODO (empty for none)
///   - `quoted_text`: text that's being replied to
///   - `quoted_sender`: TODO
pub fn send_message(
    id: ConnId,
    chat_id: &ChatId,
    contents: &str,
    quoted_id: &str,
    quoted_text: &str,
    quoted_sender: &str,
    file: Option<(impl AsRef<Path>, FileType)>,
    edit_msg: Option<(&MsgId, isize)>,
) -> Result<()> {
    let chat_id = CString::new(chat_id.0.as_str())?;
    let text = CString::new(contents)?;
    let quoted_id = CString::new(quoted_id)?;
    let quoted_text = CString::new(quoted_text)?;
    let quoted_sender = CString::new(quoted_sender)?;

    let (edit_msg_id, edit_msg_sent) = if let Some((id, timestamp)) = &edit_msg {
        (Some(CString::new(id.0.as_str())?), *timestamp as _)
    } else {
        (None, 0)
    };

    let (file_path, file_type) = if let Some((path, file_type)) = file {
        (
            Some(CString::new(path.as_ref().to_string_lossy().to_string())?),
            Some(CString::new(file_type.to_string())?),
        )
    } else {
        (None, None)
    };

    attempt(unsafe {
        sys::CWmSendMessage(
            id.0 as _,
            chat_id.as_ptr().cast_mut(),
            text.as_ptr().cast_mut(),
            quoted_id.as_ptr().cast_mut(),
            quoted_text.as_ptr().cast_mut(),
            quoted_sender.as_ptr().cast_mut(),
            cstr_maybe(file_path.as_ref()),
            cstr_maybe(file_type.as_ref()),
            cstr_maybe(edit_msg_id.as_ref()),
            edit_msg_sent,
        )
    })
}

fn cstr_maybe(c: Option<&CString>) -> *mut c_char {
    c.as_ref()
        .map(|n| n.as_ptr())
        .unwrap_or(EMPTY.as_ptr())
        .cast_mut()
}

/// Forces an update of the contact list with new info if any.
///
/// As the new info loads, it will be streamed in
/// through [`ChatEvent::NewContactsNotify`],
/// so watch your events for that.
pub fn load_contacts(id: ConnId) -> Result<()> {
    attempt(unsafe { sys::CWmGetContacts(id.0 as _) })
}
