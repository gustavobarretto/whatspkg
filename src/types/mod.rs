mod jid;

pub use jid::Jid;

/// Message ID type (WhatsApp internal ID string).
pub type MessageId = String;

/// Server-assigned ID for newsletter messages.
pub type MessageServerId = i32;
