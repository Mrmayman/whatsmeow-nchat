use std::{ffi::NulError, fmt::Display};

use thiserror::Error;
use whatsmeow_nchat_sys::LogMsg;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum WhatsmeowError {
    Unknown,
    Error(LogMsg),
    Nul(#[from] NulError),
    Poison,
}

pub type Result<T> = core::result::Result<T, WhatsmeowError>;

impl Display for WhatsmeowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WhatsmeowError::Unknown => write!(f, "whatsmeow unknown error"),
            WhatsmeowError::Error(msg) => {
                write!(f, "{}\n{}:{}", msg.message, msg.filename, msg.line_no)
            }
            WhatsmeowError::Nul(err) => write!(f, "whatsmeow ffi: {err}"),
            WhatsmeowError::Poison => write!(f, "whatsmeow: mutex panicked (poison error)"),
        }
    }
}

pub fn get_error() -> WhatsmeowError {
    let Ok(mut lock) = whatsmeow_nchat_sys::LOG_STATE.lock() else {
        return WhatsmeowError::Poison;
    };
    if let Some(err) = lock.warnings.pop() {
        WhatsmeowError::Error(err)
    } else {
        WhatsmeowError::Unknown
    }
}

pub fn attempt(r: whatsmeow_nchat_sys::GoInt) -> Result<()> {
    if r == -1 {
        Err(get_error())
    } else {
        Ok(())
    }
}
