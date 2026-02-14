use std::{
    ffi::{c_char, c_int, CStr},
    sync::{mpsc::Sender, LazyLock, Mutex, RwLock},
};

use crate::{ChatId, ConnId, MsgId, StatusFlags};

#[derive(Debug, Clone)]
pub enum ChatEvent {
    NewContactsNotify {
        name: String,
        phone: String,
        is_self: bool,
        is_alias: bool,
        notify: c_int,
    },
    NewChatsNotify {
        is_unread: bool,
        is_muted: bool,
        is_pinned: bool,
        last_message_time: c_int,
    },
    NewMessagesNotify {
        msg_id: MsgId,
        sender_id: String,
        text: String,
        from_me: c_int,
        quoted_id: String,
        file_id: String,
        file_path: String,
        file_status: c_int,
        time_sent: c_int,
        is_read: bool,
        is_edited: bool,
    },
    NewTypingNotify {
        user_id: String,
        is_typing: bool,
    },
    NewMessageStatusNotify {
        msg_id: MsgId,
        is_read: bool,
    },
    NewMessageFileNotify {
        msg_id: MsgId,
        file_path: String,
        file_status: c_int,
        action: c_int,
    },
    NewMessageReactionNotify {
        msg_id: MsgId,
        sender_id: String,
        text: String,
        from_me: c_int,
    },
    DeleteChatNotify,
    DeleteMessageNotify(MsgId),
    UpdateIsMuted(bool),
    UpdatePinNotify {
        is_pinned: bool,
        time_pinned: c_int,
    },
}

#[derive(Debug, Clone)]
pub enum Event {
    ChatEvent(ChatId, ChatEvent),

    NewStatusNotify {
        user_id: String,
        is_online: c_int,
        time_seen: c_int,
    },

    Reinit,
    /// Open WhatsApp on your phone, click the menu bar and select "Linked devices".
    /// Click on "Link a device", unlock the phone and aim its camera at the
    /// Qr code displayed on the computer screen.
    ///
    /// Scan the Qr code to authenticate
    QrCodeAtPath(String),
    /// Open the WhatsApp notification "Enter code to link new device" on your phone,
    /// click "Confirm" and enter below pairing code on your phone, or press CTRL-C
    /// to abort.
    PairingCode(String),
    /// When it's about to print something, so it's releasing the TUI?
    SetProtocolUiControl {
        is_take_control: bool,
    },
    SetStatus(StatusFlags),
    ClearStatus(StatusFlags),
}

type SentEvent = (ConnId, Event);
pub static SENDER: LazyLock<RwLock<Option<Sender<SentEvent>>>> =
    LazyLock::new(|| RwLock::new(None));

fn sendm(id: c_int, event: Event) {
    if let Ok(Some(s)) = SENDER.read().as_deref() {
        _ = s.send((ConnId(id as isize), event));
    }
}
fn sendc(id: c_int, chat_id: *const c_char, event: ChatEvent) {
    sendm(id, Event::ChatEvent(ChatId(cstr(chat_id)), event));
}

fn cstr(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return "".to_string();
    }
    let out = unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() };
    unsafe { libc::free(ptr as _) };
    out
}

#[no_mangle]
pub extern "C" fn WmNewContactsNotify(
    conn_id: c_int,
    chat_id: *const c_char,
    name: *const c_char,
    phone: *const c_char,
    is_self: c_int,
    is_alias: c_int,
    notify: c_int,
) {
    sendc(
        conn_id,
        chat_id,
        ChatEvent::NewContactsNotify {
            name: cstr(name),
            phone: cstr(phone),
            is_self: is_self != 0,
            is_alias: is_alias != 0,
            notify,
        },
    )
}

#[no_mangle]
pub extern "C" fn WmNewChatsNotify(
    conn_id: c_int,
    chat_id: *const c_char,
    is_unread: c_int,
    is_muted: c_int,
    is_pinned: c_int,
    last_message_time: c_int,
) {
    sendc(
        conn_id,
        chat_id,
        ChatEvent::NewChatsNotify {
            is_unread: is_unread != 0,
            is_muted: is_muted != 0,
            is_pinned: is_pinned != 0,
            last_message_time,
        },
    );
}

#[no_mangle]
pub extern "C" fn WmNewMessagesNotify(
    conn_id: c_int,
    chat_id: *const c_char,
    msg_id: *const c_char,
    sender_id: *const c_char,
    text: *const c_char,
    from_me: c_int,
    quoted_id: *const c_char,
    file_id: *const c_char,
    file_path: *const c_char,
    file_status: c_int,
    time_sent: c_int,
    is_read: c_int,
    is_edited: c_int,
) {
    sendc(
        conn_id,
        chat_id,
        ChatEvent::NewMessagesNotify {
            msg_id: MsgId(cstr(msg_id)),
            sender_id: cstr(sender_id),
            text: cstr(text),
            from_me,
            quoted_id: cstr(quoted_id),
            file_id: cstr(file_id),
            file_path: cstr(file_path),
            file_status,
            time_sent,
            is_read: is_read != 0,
            is_edited: is_edited != 0,
        },
    );
}

#[no_mangle]
pub extern "C" fn WmNewStatusNotify(
    conn_id: c_int,
    user_id: *const c_char,
    is_online: c_int,
    time_seen: c_int,
) {
    sendm(
        conn_id,
        Event::NewStatusNotify {
            user_id: cstr(user_id),
            is_online,
            time_seen,
        },
    );
}

#[no_mangle]
pub extern "C" fn WmNewTypingNotify(
    conn_id: c_int,
    chat_id: *const c_char,
    user_id: *const c_char,
    is_typing: c_int,
) {
    sendc(
        conn_id,
        chat_id,
        ChatEvent::NewTypingNotify {
            user_id: cstr(user_id),
            is_typing: is_typing != 0,
        },
    );
}

#[no_mangle]
pub extern "C" fn WmNewMessageStatusNotify(
    conn_id: c_int,
    chat_id: *const c_char,
    msg_id: *const c_char,
    is_read: c_int,
) {
    sendc(
        conn_id,
        chat_id,
        ChatEvent::NewMessageStatusNotify {
            msg_id: MsgId(cstr(msg_id)),
            is_read: is_read != 0,
        },
    );
}

#[no_mangle]
pub extern "C" fn WmNewMessageFileNotify(
    conn_id: c_int,
    chat_id: *const c_char,
    msg_id: *const c_char,
    file_path: *const c_char,
    file_status: c_int,
    action: c_int,
) {
    sendc(
        conn_id,
        chat_id,
        ChatEvent::NewMessageFileNotify {
            msg_id: MsgId(cstr(msg_id)),
            file_path: cstr(file_path),
            file_status,
            action,
        },
    );
}

#[no_mangle]
pub extern "C" fn WmNewMessageReactionNotify(
    conn_id: c_int,
    chat_id: *const c_char,
    msg_id: *const c_char,
    sender_id: *const c_char,
    text: *const c_char,
    from_me: c_int,
) {
    sendc(
        conn_id,
        chat_id,
        ChatEvent::NewMessageReactionNotify {
            msg_id: MsgId(cstr(msg_id)),
            sender_id: cstr(sender_id),
            text: cstr(text),
            from_me,
        },
    );
}

#[no_mangle]
pub extern "C" fn WmDeleteChatNotify(conn_id: c_int, chat_id: *const c_char) {
    sendc(conn_id, chat_id, ChatEvent::DeleteChatNotify);
}

#[no_mangle]
pub extern "C" fn WmDeleteMessageNotify(
    conn_id: c_int,
    chat_id: *const c_char,
    msg_id: *const c_char,
) {
    sendc(
        conn_id,
        chat_id,
        ChatEvent::DeleteMessageNotify(MsgId(cstr(msg_id))),
    );
}

#[no_mangle]
pub extern "C" fn WmUpdateMuteNotify(conn_id: c_int, chat_id: *const c_char, is_muted: c_int) {
    sendc(conn_id, chat_id, ChatEvent::UpdateIsMuted(is_muted != 0));
}

#[no_mangle]
pub extern "C" fn WmUpdatePinNotify(
    conn_id: c_int,
    chat_id: *const c_char,
    is_pinned: c_int,
    time_pinned: c_int,
) {
    sendc(
        conn_id,
        chat_id,
        ChatEvent::UpdatePinNotify {
            is_pinned: is_pinned != 0,
            time_pinned,
        },
    );
}

#[no_mangle]
pub extern "C" fn WmReinit(conn_id: c_int) {
    sendm(conn_id, Event::Reinit);
}

#[no_mangle]
pub extern "C" fn WmSetProtocolUiControl(conn_id: c_int, is_take_control: c_int) {
    sendm(
        conn_id,
        Event::SetProtocolUiControl {
            is_take_control: is_take_control != 0,
        },
    );
}

#[no_mangle]
pub extern "C" fn WmSetStatus(conn_id: c_int, flags: c_int) {
    sendm(
        conn_id,
        Event::SetStatus(StatusFlags::from_bits_retain(flags as _)),
    );
}

#[no_mangle]
pub extern "C" fn WmClearStatus(conn_id: c_int, flags: c_int) {
    sendm(
        conn_id,
        Event::ClearStatus(StatusFlags::from_bits_retain(flags as _)),
    );
}

#[no_mangle]
pub extern "C" fn WmAppConfigGetNum(param: *const c_char) -> c_int {
    println!("[HOOK] WmAppConfigGetNum param={}", cstr(param));
    0 // placeholder
}

#[no_mangle]
pub extern "C" fn WmAppConfigSetNum(param: *const c_char, value: c_int) {
    println!(
        "[HOOK] WmAppConfigSetNum param={} value={}",
        cstr(param),
        value
    );
}

#[no_mangle]
pub extern "C" fn WmLogTrace(filename: *const c_char, line_no: c_int, message: *const c_char) {
    println!("TRACE {}:{} {}", cstr(filename), line_no, cstr(message));
}

#[no_mangle]
pub extern "C" fn WmLogDebug(filename: *const c_char, line_no: c_int, message: *const c_char) {
    println!("DEBUG {}:{} {}", cstr(filename), line_no, cstr(message));
}

#[no_mangle]
pub extern "C" fn WmLogInfo(filename: *const c_char, line_no: c_int, message: *const c_char) {
    println!("INFO {}:{} {}", cstr(filename), line_no, cstr(message));
}

#[no_mangle]
pub extern "C" fn WmLogWarning(filename: *const c_char, line_no: c_int, message: *const c_char) {
    let message = cstr(message);
    let filename = cstr(filename);
    println!("WARN {filename}:{line_no} {message}");
    LOG_STATE.lock().unwrap().warnings.push(LogMsg {
        filename,
        line_no,
        message,
    });
}

#[no_mangle]
pub extern "C" fn WmLogError(filename: *const c_char, line_no: c_int, message: *const c_char) {
    println!("ERROR {}:{} {}", cstr(filename), line_no, cstr(message));
}

pub static LOG_STATE: LazyLock<Mutex<LogState>> = LazyLock::new(|| Mutex::new(LogState::default()));

#[derive(Debug)]
pub struct LogMsg {
    pub filename: String,
    pub line_no: c_int,
    pub message: String,
}

#[derive(Default)]
pub struct LogState {
    pub warnings: Vec<LogMsg>,
}

#[no_mangle]
pub extern "C" fn WmExtShowImage(path: *const c_char) {
    sendm(0, Event::QrCodeAtPath(cstr(path)));
}

#[no_mangle]
pub extern "C" fn WmExtLoginPairingCode(code: *const c_char) {
    sendm(0, Event::PairingCode(cstr(code)));
}
