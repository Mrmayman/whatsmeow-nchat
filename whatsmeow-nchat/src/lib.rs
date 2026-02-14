pub use sys::{ChatEvent, ConnId, Event};
use whatsmeow_nchat_sys::{self as sys};

use std::{ffi::CString, path::Path, sync::mpsc::Receiver};

mod error;
use error::{attempt, get_error};
pub use error::{Result, WhatsmeowError};

pub fn init(
    path: impl AsRef<Path>,
    proxy: &str,
    send_type: isize,
) -> Result<(ConnId, Receiver<(ConnId, Event)>)> {
    let path = CString::new(path.as_ref().as_os_str().as_encoded_bytes())?;
    let proxy = CString::new(proxy)?;

    let (send, recv) = std::sync::mpsc::channel();
    *sys::SENDER.write().map_err(|_| WhatsmeowError::Poison)? = Some(send);

    let conn_id = unsafe { sys::CWmInit(path.as_ptr() as _, proxy.as_ptr() as _, send_type as _) };
    if conn_id == -1 {
        Err(get_error())
    } else {
        Ok((ConnId(conn_id as isize), recv))
    }
}

pub fn login(id: ConnId) -> Result<()> {
    attempt(unsafe { sys::CWmLogin(id.0 as _) })
}

pub fn logout(id: ConnId) -> Result<()> {
    attempt(unsafe { sys::CWmLogout(id.0 as _) })
}
