use std::{
    collections::HashMap,
    ffi::{c_char, c_int},
    sync::{LazyLock, RwLock},
};
use tokio::sync::mpsc::UnboundedSender as Sender;

use crate::{
    handlers::cstr, ConnId, DownloadFileAction, DownloadFileStatus, Jid, MsgId, StatusFlags,
};

#[derive(Debug, Clone)]
pub enum ChatEvent {
    NewContactsNotify {
        name: String,
        phone: String,
        is_self: bool,
        is_group: bool,
        notify: isize,
    },
    NewChatsNotify {
        is_unread: bool,
        is_muted: bool,
        is_pinned: bool,
        last_message_time: isize,
    },
    NewMessagesNotify {
        msg_id: MsgId,
        sender_id: Jid,
        text: String,
        from_me: bool,
        quoted_id: Option<MsgId>,
        file_id_path: Option<(String, String)>,
        file_status: DownloadFileStatus,
        time_sent: isize,
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
    /// File attachment downloaded by user
    NewMessageFileNotify {
        msg_id: MsgId,
        file_path: String,
        file_status: DownloadFileStatus,
        action: DownloadFileAction,
    },
    NewMessageReactionNotify {
        msg_id: MsgId,
        sender_id: Jid,
        emoji: String,
        from_me: bool,
    },
    DeleteChatNotify,
    DeleteMessageNotify(MsgId),
    UpdateIsMuted(bool),
    UpdatePinNotify {
        is_pinned: bool,
        time_pinned: isize,
    },
}

#[derive(Debug, Clone)]
pub enum Event {
    ChatEvent(Jid, ChatEvent),

    NewStatusNotify {
        user_id: String,
        is_online: bool,
        time_seen: isize,
    },

    Reinit,
    /// Open WhatsApp on your phone, click the menu bar and select "Linked devices".
    /// Click on "Link a device", unlock the phone and aim its camera at the
    /// Qr code displayed on the computer screen.
    ///
    /// Scan the Qr code to authenticate
    QrCode(String),
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

type SenderMap = HashMap<ConnId, Sender<Event>>;
pub static SENDERS: LazyLock<RwLock<SenderMap>> = LazyLock::new(|| RwLock::new(HashMap::new()));

pub fn add_sender(id: ConnId, sender: Sender<Event>) {
    if let Ok(mut smap) = SENDERS.write() {
        smap.insert(id, sender);
    }
}

pub fn sendm(id: c_int, event: Event) {
    if let Ok(smap) = SENDERS.read() {
        if let Some(s) = smap.get(&ConnId(id as _)) {
            _ = s.send(event);
        }
    }
}
pub fn sendc(id: c_int, chat_id: *const c_char, event: ChatEvent) {
    let Some(chat_id) = Jid::parse(&cstr(chat_id)) else {
        return;
    };
    sendm(id, Event::ChatEvent(chat_id, event));
}
