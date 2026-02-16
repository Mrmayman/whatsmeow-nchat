use std::{
    ffi::{CString, NulError},
    fmt::Display,
    str::FromStr,
};

use bitflags::bitflags;

use crate::GoInt;

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

/// The server or origin of a [`Jid`].
/// Used to differentiate between DMs, groups, bots, etc.
///
/// Here's the mapping
/// ```txt
/// DefaultUser = "s.whatsapp.net"
/// Group       = "g.us"
/// LegacyUser  = "c.us"
/// Broadcast   = "broadcast"
/// HiddenUser  = "lid"
/// Messenger   = "msgr"
/// Interop     = "interop"
/// Newsletter  = "newsletter"
/// Hosted      = "hosted"
/// HostedLID   = "hosted.lid"
/// Bot         = "bot"
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum JidServer {
    DefaultUser,
    Group,
    LegacyUser,
    Broadcast,
    HiddenUser,
    Messenger,
    Interop,
    Newsletter,
    Hosted,
    HostedLID,
    Bot,
}

impl FromStr for JidServer {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "s.whatsapp.net" => Ok(Self::DefaultUser),
            "g.us" => Ok(Self::Group),
            "c.us" => Ok(Self::LegacyUser),
            "broadcast" => Ok(Self::Broadcast),
            "lid" => Ok(Self::HiddenUser),
            "msgr" => Ok(Self::Messenger),
            "interop" => Ok(Self::Interop),
            "newsletter" => Ok(Self::Newsletter),
            "hosted" => Ok(Self::Hosted),
            "hosted.lid" => Ok(Self::HostedLID),
            "bot" => Ok(Self::Bot),
            _ => Err(()),
        }
    }
}

impl Display for JidServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::DefaultUser => "s.whatsapp.net",
                Self::Group => "g.us",
                Self::LegacyUser => "c.us",
                Self::Broadcast => "broadcast",
                Self::HiddenUser => "lid",
                Self::Messenger => "msgr",
                Self::Interop => "interop",
                Self::Newsletter => "newsletter",
                Self::Hosted => "hosted",
                Self::HostedLID => "hosted.lid",
                Self::Bot => "bot",
            }
        )
    }
}

/// JIDs are unique identifiers for chats/users on WhatsApp.
///
/// They come in the format `SOME_NUMBER@SERVER`, where
/// `SOME_NUMBER` is phone number for normal users.
/// I don't know what it is for others.
///
/// See [`JidServer`] for info about servers.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Jid(String, JidServer);

impl Jid {
    pub fn parse(input: &str) -> Option<Self> {
        let mut i = input.split('@');
        let name = i.next()?;
        let server = JidServer::from_str(i.next()?).ok()?;

        Some(Self(name.to_owned(), server))
    }

    #[must_use]
    pub fn from_phone_no(phone_no: String) -> Self {
        Jid(phone_no, JidServer::DefaultUser)
    }

    /// Returns the phone number or id (for groups/bots) of the Jid
    #[must_use]
    pub fn number(&self) -> &str {
        self.0.as_str()
    }

    /// Returns the server, or origin, of the id
    #[must_use]
    pub fn server(&self) -> JidServer {
        self.1
    }

    pub fn to_id(&self) -> String {
        format!("{}@{}", self.0, self.1)
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MsgId(pub String);

impl TryInto<CString> for &Jid {
    type Error = NulError;
    fn try_into(self) -> Result<CString, NulError> {
        CString::new(self.to_id().as_str())
    }
}
impl TryInto<CString> for &MsgId {
    type Error = NulError;
    fn try_into(self) -> Result<CString, NulError> {
        CString::new(self.0.as_str())
    }
}

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
