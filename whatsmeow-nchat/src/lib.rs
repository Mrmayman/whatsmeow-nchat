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
//! - Have system to prevent using connections after cleaning them up

pub use sys::{ChatEvent, ConnId, Event, Jid, MsgId, StatusFlags};
use whatsmeow_nchat_sys::{self as sys, DownloadFileAction};

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

    unsafe {
        sys::connect(
            path.as_ptr().cast_mut(),
            proxy.as_ptr().cast_mut(),
            send_type as _,
        )
    }
    .map_err(|()| get_error())
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
            match sys::CWmExtGetState(id.raw()) {
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
    attempt(unsafe { sys::CWmLogin(id.raw()) })
}

/// Logs out of the account linked in the connection.
///
/// # Errors
/// - `id` is invalid
pub fn logout(id: ConnId) -> Result<()> {
    attempt(unsafe { sys::CWmLogout(id.raw()) })
}

// TODO: find out what this is
pub fn cleanup(id: ConnId) -> Result<()> {
    attempt(unsafe { sys::CWmCleanup(id.raw()) })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Audio,
    Video,
    Image,
    Document,
}

impl FileType {
    const fn as_str(self) -> &'static str {
        match self {
            FileType::Audio => "audio",
            FileType::Video => "video",
            FileType::Image => "image",
            FileType::Document => "document",
        }
    }
}

pub struct QuotedMessage {
    pub sender: Jid,
    pub contents: String,
    pub message_id: MsgId,
}

/// Sends/edits a message with the given parameters.
///
/// Optional parameters:
///
/// - To edit the message instead of sending it,
///   use `edit_msg` argument (Id and timestamp of message being edited)
/// - To attach a file, use `file` argument
/// - To reply to a quoted message, use `reply_to` argument
///
/// # Errors
/// - `id` is invalid
/// - not logged in
/// - JID couldn't be parsed
/// - attached file couldn't be read or uploaded
/// - other errors from sending messages
pub fn send_message(
    id: ConnId,
    chat_id: &Jid,
    contents: &str,
    reply_to: Option<&QuotedMessage>,
    file: Option<(impl AsRef<Path>, FileType)>,
    edit_msg: Option<(&MsgId, isize)>,
) -> Result<()> {
    let chat_id: CString = chat_id.try_into()?;
    let text = CString::new(contents)?;

    let (quoted_id, quoted_text, quoted_sender) = if let Some(msg) = reply_to {
        (
            Some((&msg.message_id).try_into()?),
            Some(CString::new(msg.contents.as_str())?),
            Some((&msg.sender).try_into()?),
        )
    } else {
        (None, None, None)
    };

    let (edit_msg_id, edit_msg_sent) = if let Some((id, timestamp)) = &edit_msg {
        (Some(CString::new(id.0.as_str())?), *timestamp as _)
    } else {
        (None, 0)
    };

    let (file_path, file_type) = if let Some((path, file_type)) = file {
        (
            Some(CString::new(path.as_ref().to_string_lossy().to_string())?),
            Some(CString::new(file_type.as_str())?),
        )
    } else {
        (None, None)
    };

    attempt(unsafe {
        sys::CWmSendMessage(
            id.raw(),
            chat_id.as_ptr().cast_mut(),
            text.as_ptr().cast_mut(),
            cstr_maybe(quoted_id.as_ref()),
            cstr_maybe(quoted_text.as_ref()),
            cstr_maybe(quoted_sender.as_ref()),
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
///
/// # Errors
/// - `id` is invalid
pub fn fetch_contacts(id: ConnId) -> Result<()> {
    attempt(unsafe { sys::CWmGetContacts(id.raw()) })
}

/// Fetches the status (online/offline/...) of a user.
///
/// The actual data will be returned later through [`Event`].
/// Call this when you open a DM page.
///
/// # Errors
/// - `id` is invalid
/// - Tried checking status of self
/// - Other protocol errors
pub fn fetch_status(id: ConnId, user_id: &Jid) -> Result<()> {
    let user_id_c: CString = user_id.try_into()?;
    let r = unsafe { sys::CWmGetStatus(id.raw(), user_id_c.as_ptr().cast_mut()) };
    if r == -1 {
        Err(get_error())
    } else {
        Ok(())
    }
}

/// Marks a message as read.
///
/// You might need to call this for every unread message,
/// even if they come in bulk (not sure).
///
/// # Errors
/// - `id` is invalid
/// - Not logged in
/// - Other protocol errors
pub fn mark_message_read(id: ConnId, chat_id: &Jid, sender_id: &Jid, msg: &MsgId) -> Result<()> {
    let chat_id: CString = chat_id.try_into()?;
    let sender_id: CString = sender_id.try_into()?;
    let msg_id: CString = msg.try_into()?;
    attempt(unsafe {
        sys::CWmMarkMessageRead(
            id.raw(),
            chat_id.as_ptr().cast_mut(),
            sender_id.as_ptr().cast_mut(),
            msg_id.as_ptr().cast_mut(),
        )
    })
}

/// Deletes the specified message.
///
/// **You can only delete either your own message,
/// or messages in a group you're admin in.**
/// Otherwise this simply does nothing
pub fn delete_message(id: ConnId, chat_id: &Jid, sender_id: &Jid, msg: &MsgId) -> Result<()> {
    let chat_id: CString = chat_id.try_into()?;
    let sender_id: CString = sender_id.try_into()?;
    let msg_id: CString = msg.try_into()?;
    attempt(unsafe {
        sys::CWmDeleteMessage(
            id.raw(),
            chat_id.as_ptr().cast_mut(),
            sender_id.as_ptr().cast_mut(),
            msg_id.as_ptr().cast_mut(),
        )
    })
}

/// Exits a group
pub fn exit_group(id: ConnId, chat_id: &Jid) -> Result<()> {
    let chat_id: CString = chat_id.try_into()?;
    attempt(unsafe { sys::CWmDeleteChat(id.raw(), chat_id.as_ptr().cast_mut()) })
}

/// Enables/disables the "PERSON is typing..." indicator.
///
/// # Errors
/// - `id` is invalid
/// - Not logged in
/// - Other protocol errors
pub fn send_typing_indicator(id: ConnId, chat_id: &Jid, is_typing: bool) -> Result<()> {
    let chat_id: CString = chat_id.try_into()?;
    attempt(unsafe { sys::CWmSendTyping(id.raw(), chat_id.as_ptr().cast_mut(), is_typing as _) })
}

/// Sets your online status.
///
/// This will also trigger a
/// [`Event::SetStatus`] or [`Event::ClearStatus`]
/// for client-side updation.
pub fn set_is_online(id: ConnId, is_online: bool) -> Result<()> {
    attempt(unsafe { sys::CWmSendStatus(id.raw(), is_online as _) })
}

/// Downloads a file attachment.
///
/// This triggers a [`ChatEvent::NewMessageFileNotify`].
/// The `action` is just passed along to that event for later use
/// and doesn't affect the logic here.
pub fn download_file(
    id: ConnId,
    chat_id: &Jid,
    msg: &MsgId,
    file_id: &str,
    action: DownloadFileAction,
) -> Result<()> {
    let chat_id: CString = chat_id.try_into()?;
    let msg: CString = msg.try_into()?;
    let file_id = CString::new(file_id)?;

    attempt(unsafe {
        sys::CWmDownloadFile(
            id.raw(),
            chat_id.as_ptr().cast_mut(),
            msg.as_ptr().cast_mut(),
            file_id.as_ptr().cast_mut(),
            action as _,
        )
    })
}

/// Reacts to a message with an emoji.
///
/// Triggers a [`ChatEvent::NewMessageReactionNotify`]
/// for client-side updation.
pub fn send_reaction(
    id: ConnId,
    chat_id: &Jid,
    sender_id: &Jid,
    msg_id: &MsgId,
    emoji: &str,
) -> Result<()> {
    let chat_id: CString = chat_id.try_into()?;
    let sender_id: CString = sender_id.try_into()?;
    let msg_id: CString = msg_id.try_into()?;
    let emoji = CString::new(emoji)?;

    attempt(unsafe {
        sys::CWmSendReaction(
            id.raw(),
            chat_id.as_ptr().cast_mut(),
            sender_id.as_ptr().cast_mut(),
            msg_id.as_ptr().cast_mut(),
            emoji.as_ptr().cast_mut(),
        )
    })
}
