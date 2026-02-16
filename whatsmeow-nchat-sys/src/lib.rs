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

use tokio::sync::mpsc::{self, UnboundedReceiver as Receiver};

mod events;
mod handlers;
mod types;

pub use events::{ChatEvent, Event};
pub use handlers::{get_error, LogMsg};
pub use types::{
    ConnId, DownloadFileAction, DownloadFileStatus, Jid, JidServer, MsgId, StatusFlags,
};

use crate::events::add_sender;
