use thiserror::Error;

/// Library result type.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when using the WhatsApp client.
#[derive(Error, Debug)]
pub enum Error {
    #[error("connection: {0}")]
    Connection(#[from] ConnectionError),

    #[error("pairing: {0}")]
    Pairing(#[from] PairingError),

    #[error("store: {0}")]
    Store(#[from] StoreError),

    #[error("send: {0}")]
    Send(#[from] SendError),

    #[error("binary protocol: {0}")]
    Binary(String),

    #[error("not connected")]
    NotConnected,

    #[error("not logged in")]
    NotLoggedIn,

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

/// Connection-related errors.
#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("websocket: {0}")]
    WebSocket(String),

    #[error("handshake failed")]
    HandshakeFailed,

    #[error("timeout")]
    Timeout,

    #[error("disconnected")]
    Disconnected,

    #[error("connect failure: {0}")]
    ConnectFailure(ConnectFailureReason),
}

/// Reason code for connection failures (maps to whatsmeow ConnectFailureReason).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl std::fmt::Display for ConnectFailureReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Self::LoggedOut => "logged out from another device",
            Self::TempBanned => "account temporarily banned",
            Self::MainDeviceGone => "primary device was logged out",
            Self::UnknownLogout => "logged out for unknown reason",
            Self::ClientOutdated => "client is out of date",
            Self::BadUserAgent => "client user agent was rejected",
            Self::CATExpired => "messenger crypto auth token has expired",
            Self::CATInvalid => "messenger crypto auth token is invalid",
            _ => "connection failure",
        };
        write!(f, "{} (code {})", msg, *self as i32)
    }
}

/// Pairing-related errors.
#[derive(Error, Debug)]
pub enum PairingError {
    #[error("invalid device identity HMAC")]
    InvalidDeviceIdentityHmac,

    #[error("invalid device signature")]
    InvalidDeviceSignature,

    #[error("pairing rejected locally")]
    RejectedLocally,

    #[error("protocol: {0}")]
    Protocol(String),

    #[error("database: {0}")]
    Database(String),
}

/// Store (device/session) errors.
#[derive(Error, Debug)]
pub enum StoreError {
    #[error("save failed: {0}")]
    Save(String),

    #[error("load failed: {0}")]
    Load(String),

    #[error("identity not found")]
    IdentityNotFound,
}

/// Send message errors.
#[derive(Error, Debug)]
pub enum SendError {
    #[error("message not found for retry")]
    MessageNotFoundForRetry,

    #[error("encryption failed")]
    EncryptionFailed,

    #[error("timeout waiting for response")]
    Timeout,

    #[error("server error: {0}")]
    Server(String),
}
