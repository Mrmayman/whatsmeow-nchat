use std::ffi::{c_char, c_int, CStr};

fn cstr_to_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return "".to_string();
    }
    unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

#[no_mangle]
pub extern "C" fn WmNewContactsNotify(
    p_ConnId: c_int,
    p_ChatId: *const c_char,
    p_Name: *const c_char,
    p_Phone: *const c_char,
    p_IsSelf: c_int,
    p_IsAlias: c_int,
    p_Notify: c_int,
) {
    println!(
        "[HOOK] WmNewContactsNotify conn={} chat={} name={} phone={} self={} alias={} notify={}",
        p_ConnId,
        cstr_to_string(p_ChatId),
        cstr_to_string(p_Name),
        cstr_to_string(p_Phone),
        p_IsSelf,
        p_IsAlias,
        p_Notify
    );
}

#[no_mangle]
pub extern "C" fn WmNewChatsNotify(
    p_ConnId: c_int,
    p_ChatId: *const c_char,
    p_IsUnread: c_int,
    p_IsMuted: c_int,
    p_IsPinned: c_int,
    p_LastMessageTime: c_int,
) {
    println!(
        "[HOOK] WmNewChatsNotify conn={} chat={} unread={} muted={} pinned={} time={}",
        p_ConnId,
        cstr_to_string(p_ChatId),
        p_IsUnread,
        p_IsMuted,
        p_IsPinned,
        p_LastMessageTime
    );
}

#[no_mangle]
pub extern "C" fn WmNewMessagesNotify(
    p_ConnId: c_int,
    p_ChatId: *const c_char,
    p_MsgId: *const c_char,
    p_SenderId: *const c_char,
    p_Text: *const c_char,
    p_FromMe: c_int,
    p_QuotedId: *const c_char,
    p_FileId: *const c_char,
    p_FilePath: *const c_char,
    p_FileStatus: c_int,
    p_TimeSent: c_int,
    p_IsRead: c_int,
    p_IsEdited: c_int,
) {
    println!(
        "[HOOK] WmNewMessagesNotify conn={} chat={} msg={} sender={} text={} from_me={} quoted={} fileid={} filepath={} file_status={} time={} read={} edited={}",
        p_ConnId,
        cstr_to_string(p_ChatId),
        cstr_to_string(p_MsgId),
        cstr_to_string(p_SenderId),
        cstr_to_string(p_Text),
        p_FromMe,
        cstr_to_string(p_QuotedId),
        cstr_to_string(p_FileId),
        cstr_to_string(p_FilePath),
        p_FileStatus,
        p_TimeSent,
        p_IsRead,
        p_IsEdited
    );
}

#[no_mangle]
pub extern "C" fn WmNewStatusNotify(
    p_ConnId: c_int,
    p_UserId: *const c_char,
    p_IsOnline: c_int,
    p_TimeSeen: c_int,
) {
    println!(
        "[HOOK] WmNewStatusNotify conn={} user={} online={} time={}",
        p_ConnId,
        cstr_to_string(p_UserId),
        p_IsOnline,
        p_TimeSeen
    );
}

#[no_mangle]
pub extern "C" fn WmNewTypingNotify(
    p_ConnId: c_int,
    p_ChatId: *const c_char,
    p_UserId: *const c_char,
    p_IsTyping: c_int,
) {
    println!(
        "[HOOK] WmNewTypingNotify conn={} chat={} user={} typing={}",
        p_ConnId,
        cstr_to_string(p_ChatId),
        cstr_to_string(p_UserId),
        p_IsTyping
    );
}

#[no_mangle]
pub extern "C" fn WmNewMessageStatusNotify(
    p_ConnId: c_int,
    p_ChatId: *const c_char,
    p_MsgId: *const c_char,
    p_IsRead: c_int,
) {
    println!(
        "[HOOK] WmNewMessageStatusNotify conn={} chat={} msg={} read={}",
        p_ConnId,
        cstr_to_string(p_ChatId),
        cstr_to_string(p_MsgId),
        p_IsRead
    );
}

#[no_mangle]
pub extern "C" fn WmNewMessageFileNotify(
    p_ConnId: c_int,
    p_ChatId: *const c_char,
    p_MsgId: *const c_char,
    p_FilePath: *const c_char,
    p_FileStatus: c_int,
    p_Action: c_int,
) {
    println!(
        "[HOOK] WmNewMessageFileNotify conn={} chat={} msg={} file={} status={} action={}",
        p_ConnId,
        cstr_to_string(p_ChatId),
        cstr_to_string(p_MsgId),
        cstr_to_string(p_FilePath),
        p_FileStatus,
        p_Action
    );
}

#[no_mangle]
pub extern "C" fn WmNewMessageReactionNotify(
    p_ConnId: c_int,
    p_ChatId: *const c_char,
    p_MsgId: *const c_char,
    p_SenderId: *const c_char,
    p_Text: *const c_char,
    p_FromMe: c_int,
) {
    println!(
        "[HOOK] WmNewMessageReactionNotify conn={} chat={} msg={} sender={} text={} from_me={}",
        p_ConnId,
        cstr_to_string(p_ChatId),
        cstr_to_string(p_MsgId),
        cstr_to_string(p_SenderId),
        cstr_to_string(p_Text),
        p_FromMe
    );
}

#[no_mangle]
pub extern "C" fn WmDeleteChatNotify(p_ConnId: c_int, p_ChatId: *const c_char) {
    println!(
        "[HOOK] WmDeleteChatNotify conn={} chat={}",
        p_ConnId,
        cstr_to_string(p_ChatId)
    );
}

#[no_mangle]
pub extern "C" fn WmDeleteMessageNotify(
    p_ConnId: c_int,
    p_ChatId: *const c_char,
    p_MsgId: *const c_char,
) {
    println!(
        "[HOOK] WmDeleteMessageNotify conn={} chat={} msg={}",
        p_ConnId,
        cstr_to_string(p_ChatId),
        cstr_to_string(p_MsgId)
    );
}

#[no_mangle]
pub extern "C" fn WmUpdateMuteNotify(p_ConnId: c_int, p_ChatId: *const c_char, p_IsMuted: c_int) {
    println!(
        "[HOOK] WmUpdateMuteNotify conn={} chat={} muted={}",
        p_ConnId,
        cstr_to_string(p_ChatId),
        p_IsMuted
    );
}

#[no_mangle]
pub extern "C" fn WmUpdatePinNotify(
    p_ConnId: c_int,
    p_ChatId: *const c_char,
    p_IsPinned: c_int,
    p_TimePinned: c_int,
) {
    println!(
        "[HOOK] WmUpdatePinNotify conn={} chat={} pinned={} time={}",
        p_ConnId,
        cstr_to_string(p_ChatId),
        p_IsPinned,
        p_TimePinned
    );
}

#[no_mangle]
pub extern "C" fn WmReinit(p_ConnId: c_int) {
    println!("[HOOK] WmReinit conn={}", p_ConnId);
}

#[no_mangle]
pub extern "C" fn WmSetProtocolUiControl(p_ConnId: c_int, p_IsTakeControl: c_int) {
    println!(
        "[HOOK] WmSetProtocolUiControl conn={} take_control={}",
        p_ConnId, p_IsTakeControl
    );
}

#[no_mangle]
pub extern "C" fn WmSetStatus(p_ConnId: c_int, p_Flags: c_int) {
    println!("[HOOK] WmSetStatus conn={} flags={}", p_ConnId, p_Flags);
}

#[no_mangle]
pub extern "C" fn WmClearStatus(p_ConnId: c_int, p_Flags: c_int) {
    println!("[HOOK] WmClearStatus conn={} flags={}", p_ConnId, p_Flags);
}

#[no_mangle]
pub extern "C" fn WmAppConfigGetNum(p_Param: *const c_char) -> c_int {
    let s = cstr_to_string(p_Param);
    println!("[HOOK] WmAppConfigGetNum param={}", s);
    0 // placeholder
}

#[no_mangle]
pub extern "C" fn WmAppConfigSetNum(p_Param: *const c_char, p_Value: c_int) {
    println!(
        "[HOOK] WmAppConfigSetNum param={} value={}",
        cstr_to_string(p_Param),
        p_Value
    );
}

#[no_mangle]
pub extern "C" fn WmLogTrace(p_Filename: *const c_char, p_LineNo: c_int, p_Message: *const c_char) {
    println!(
        "[LOG TRACE] {}:{} {}",
        cstr_to_string(p_Filename),
        p_LineNo,
        cstr_to_string(p_Message)
    );
}

#[no_mangle]
pub extern "C" fn WmLogDebug(p_Filename: *const c_char, p_LineNo: c_int, p_Message: *const c_char) {
    println!(
        "[LOG DEBUG] {}:{} {}",
        cstr_to_string(p_Filename),
        p_LineNo,
        cstr_to_string(p_Message)
    );
}

#[no_mangle]
pub extern "C" fn WmLogInfo(p_Filename: *const c_char, p_LineNo: c_int, p_Message: *const c_char) {
    println!(
        "[LOG INFO] {}:{} {}",
        cstr_to_string(p_Filename),
        p_LineNo,
        cstr_to_string(p_Message)
    );
}

#[no_mangle]
pub extern "C" fn WmLogWarning(
    p_Filename: *const c_char,
    p_LineNo: c_int,
    p_Message: *const c_char,
) {
    println!(
        "[LOG WARN] {}:{} {}",
        cstr_to_string(p_Filename),
        p_LineNo,
        cstr_to_string(p_Message)
    );
}

#[no_mangle]
pub extern "C" fn WmLogError(p_Filename: *const c_char, p_LineNo: c_int, p_Message: *const c_char) {
    println!(
        "[LOG ERROR] {}:{} {}",
        cstr_to_string(p_Filename),
        p_LineNo,
        cstr_to_string(p_Message)
    );
}
