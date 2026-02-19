//! Event types emitted by the client.

use crate::types::Jid;
use std::time::Duration;

/// Events emitted by [Client](crate::Client) to registered handlers.
#[derive(Clone, Debug)]
pub enum Event {
    /// QR codes for pairing. Show as QR one by one (first ~60s, others ~20s).
    Qr { codes: Vec<String> },

    /// Pairing completed after scanning QR.
    PairSuccess {
        id: Jid,
        lid: Jid,
        business_name: String,
        platform: String,
    },

    /// Pairing failed after pair-success from server.
    PairError {
        id: Jid,
        lid: Jid,
        business_name: String,
        platform: String,
        error: String,
    },

    /// QR scanned but phone didn't have multidevice enabled.
    QrScannedWithoutMultidevice,

    /// Client connected and authenticated.
    Connected,

    /// Keepalive pings timing out.
    KeepAliveTimeout {
        error_count: u32,
        last_success: Option<std::time::SystemTime>,
    },

    /// Keepalive restored after timeouts.
    KeepAliveRestored,

    /// Logged out from another device or connect failure.
    LoggedOut {
        on_connect: bool,
        reason: Option<ConnectFailureReason>,
    },

    /// Another client connected with same keys (stream replaced).
    StreamReplaced,

    /// Temporary ban.
    TemporaryBan {
        code: TempBanReason,
        expire: Duration,
    },

    /// Disconnected (transient).
    Disconnected { reason: String },

    /// Incoming message (decrypted).
    Message(MessageEvent),

    /// Receipt (delivery/read).
    Receipt(ReceiptEvent),

    /// History sync notification.
    HistorySync { chunk_order: u32, progress: u32 },

    /// App state update.
    AppStateSync,
}

#[derive(Clone, Debug)]
pub struct MessageEvent {
    pub from: Jid,
    pub to: Jid,
    pub id: crate::types::MessageId,
    pub timestamp: std::time::SystemTime,
    pub is_group: bool,
    pub is_from_me: bool,
    /// Raw message payload (protobuf) - decode per message type.
    pub raw: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct ReceiptEvent {
    pub from: Jid,
    pub id: crate::types::MessageId,
    pub timestamp: std::time::SystemTime,
    pub is_read: bool,
    pub is_from_me: bool,
}

/// Connect failure reason.
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ConnectFailureReason {
    Generic = 400,
    LoggedOut = 401,
    TempBanned = 402,
    MainDeviceGone = 403,
    ClientOutdated = 405,
    UnknownLogout = 406,
    BadUserAgent = 409,
    CATExpired = 413,
    CATInvalid = 414,
    NotFound = 415,
    ClientUnknown = 418,
    InternalServerError = 500,
    Experimental = 501,
    ServiceUnavailable = 503,
}

impl ConnectFailureReason {
    pub fn from_code(code: i32) -> Option<Self> {
        match code {
            400 => Some(Self::Generic),
            401 => Some(Self::LoggedOut),
            402 => Some(Self::TempBanned),
            403 => Some(Self::MainDeviceGone),
            405 => Some(Self::ClientOutdated),
            406 => Some(Self::UnknownLogout),
            409 => Some(Self::BadUserAgent),
            413 => Some(Self::CATExpired),
            414 => Some(Self::CATInvalid),
            415 => Some(Self::NotFound),
            418 => Some(Self::ClientUnknown),
            500 => Some(Self::InternalServerError),
            501 => Some(Self::Experimental),
            503 => Some(Self::ServiceUnavailable),
            _ => None,
        }
    }

    pub fn is_logged_out(&self) -> bool {
        matches!(
            self,
            Self::LoggedOut | Self::MainDeviceGone | Self::UnknownLogout
        )
    }
}

/// Temporary ban reason.
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum TempBanReason {
    SentToTooManyPeople = 101,
    BlockedByUsers = 102,
    CreatedTooManyGroups = 103,
    SentTooManySameMessage = 104,
    BroadcastList = 106,
}
