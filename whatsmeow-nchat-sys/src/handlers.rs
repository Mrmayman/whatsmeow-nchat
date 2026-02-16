use std::{
    ffi::{c_char, c_int, CStr},
    sync::{LazyLock, Mutex},
};

use crate::{
    events::{sendc, sendm, ChatEvent, Event},
    DownloadFileAction, DownloadFileStatus, Jid, MsgId, StatusFlags,
};

pub fn cstr(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let out = unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() };
    unsafe { libc::free(ptr as _) };
    out
}

#[no_mangle]
extern "C" fn WmNewContactsNotify(
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
            is_group: is_alias == 0,
            notify,
        },
    );
}

#[no_mangle]
extern "C" fn WmNewChatsNotify(
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
extern "C" fn WmNewMessagesNotify(
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
    let Some(sender_id) = Jid::parse(&cstr(sender_id)) else {
        return;
    };
    let quoted_id = cstr(quoted_id);
    let file_id = cstr(file_id);
    let file_path = cstr(file_path);
    sendc(
        conn_id,
        chat_id,
        ChatEvent::NewMessagesNotify {
            msg_id: MsgId(cstr(msg_id)),
            sender_id,
            text: cstr(text),
            from_me,
            quoted_id: (!quoted_id.is_empty()).then_some(MsgId(quoted_id)),
            file_id_path: (!file_id.is_empty() && !file_path.is_empty())
                .then_some((file_id, file_path)),
            file_status: DownloadFileStatus::from_raw(file_status),
            time_sent,
            is_read: is_read != 0,
            is_edited: is_edited != 0,
        },
    );
}

#[no_mangle]
extern "C" fn WmNewStatusNotify(
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
extern "C" fn WmNewTypingNotify(
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
extern "C" fn WmNewMessageStatusNotify(
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
extern "C" fn WmNewMessageFileNotify(
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
            file_status: DownloadFileStatus::from_raw(file_status),
            action: match action {
                1 => DownloadFileAction::Open,
                2 => DownloadFileAction::Save,
                _ => DownloadFileAction::None,
            },
        },
    );
}

#[no_mangle]
extern "C" fn WmNewMessageReactionNotify(
    conn_id: c_int,
    chat_id: *const c_char,
    msg_id: *const c_char,
    sender_id: *const c_char,
    text: *const c_char,
    from_me: c_int,
) {
    let Some(sender_id) = Jid::parse(&cstr(sender_id)) else {
        return;
    };
    sendc(
        conn_id,
        chat_id,
        ChatEvent::NewMessageReactionNotify {
            msg_id: MsgId(cstr(msg_id)),
            sender_id,
            emoji: cstr(text),
            from_me,
        },
    );
}

#[no_mangle]
extern "C" fn WmDeleteChatNotify(conn_id: c_int, chat_id: *const c_char) {
    sendc(conn_id, chat_id, ChatEvent::DeleteChatNotify);
}

#[no_mangle]
extern "C" fn WmDeleteMessageNotify(conn_id: c_int, chat_id: *const c_char, msg_id: *const c_char) {
    sendc(
        conn_id,
        chat_id,
        ChatEvent::DeleteMessageNotify(MsgId(cstr(msg_id))),
    );
}

#[no_mangle]
extern "C" fn WmUpdateMuteNotify(conn_id: c_int, chat_id: *const c_char, is_muted: c_int) {
    sendc(conn_id, chat_id, ChatEvent::UpdateIsMuted(is_muted != 0));
}

#[no_mangle]
extern "C" fn WmUpdatePinNotify(
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
extern "C" fn WmReinit(conn_id: c_int) {
    sendm(conn_id, Event::Reinit);
}

#[no_mangle]
extern "C" fn WmSetProtocolUiControl(conn_id: c_int, is_take_control: c_int) {
    sendm(
        conn_id,
        Event::SetProtocolUiControl {
            is_take_control: is_take_control != 0,
        },
    );
}

#[no_mangle]
extern "C" fn WmSetStatus(conn_id: c_int, flags: c_int) {
    sendm(
        conn_id,
        Event::SetStatus(StatusFlags::from_bits_retain(flags as _)),
    );
}

#[no_mangle]
extern "C" fn WmClearStatus(conn_id: c_int, flags: c_int) {
    sendm(
        conn_id,
        Event::ClearStatus(StatusFlags::from_bits_retain(flags as _)),
    );
}

#[no_mangle]
extern "C" fn WmAppConfigGetNum(param: *const c_char) -> c_int {
    println!("[HOOK] WmAppConfigGetNum param={}", cstr(param));
    0 // placeholder
}

#[no_mangle]
extern "C" fn WmAppConfigSetNum(param: *const c_char, value: c_int) {
    println!(
        "[HOOK] WmAppConfigSetNum param={} value={}",
        cstr(param),
        value
    );
}

#[no_mangle]
extern "C" fn WmLogTrace(filename: *const c_char, line_no: c_int, message: *const c_char) {
    println!("TRACE {}:{} {}", cstr(filename), line_no, cstr(message));
}

#[no_mangle]
extern "C" fn WmLogDebug(filename: *const c_char, line_no: c_int, message: *const c_char) {
    println!("DEBUG {}:{} {}", cstr(filename), line_no, cstr(message));
}

#[no_mangle]
extern "C" fn WmLogInfo(filename: *const c_char, line_no: c_int, message: *const c_char) {
    println!("INFO {}:{} {}", cstr(filename), line_no, cstr(message));
}

#[no_mangle]
extern "C" fn WmLogWarning(filename: *const c_char, line_no: c_int, message: *const c_char) {
    let message = cstr(message);
    let filename = cstr(filename);
    println!("WARN {filename}:{line_no} {message}");
    WARNINGS.lock().unwrap().push(LogMsg {
        filename,
        line_no,
        message,
    });
}

#[no_mangle]
extern "C" fn WmLogError(filename: *const c_char, line_no: c_int, message: *const c_char) {
    println!("ERROR {}:{} {}", cstr(filename), line_no, cstr(message));
}

static WARNINGS: LazyLock<Mutex<Vec<LogMsg>>> = LazyLock::new(|| Mutex::new(Vec::new()));

#[derive(Debug)]
pub struct LogMsg {
    pub filename: String,
    pub line_no: c_int,
    pub message: String,
}

pub fn get_error() -> Option<LogMsg> {
    let Ok(mut lock) = WARNINGS.lock() else {
        return None;
    };
    lock.pop()
}

#[no_mangle]
extern "C" fn WmExtQrCode(path: *const c_char) {
    sendm(0, Event::QrCode(cstr(path)));
}

#[no_mangle]
extern "C" fn WmExtLoginPairingCode(code: *const c_char) {
    sendm(0, Event::PairingCode(cstr(code)));
}
