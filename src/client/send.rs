//! Send message types (mirrors whatsmeow SendResponse, SendRequestExtra).

use crate::types::{Jid, MessageId};
use std::time::SystemTime;

/// Response from sending a message (whatsmeow SendResponse).
#[derive(Clone, Debug)]
pub struct SendResponse {
    pub timestamp: SystemTime,
    pub id: MessageId,
    pub server_id: Option<i32>,
    pub sender: Option<Jid>,
}

/// Optional parameters for SendMessage (whatsmeow SendRequestExtra).
#[derive(Clone, Debug, Default)]
pub struct SendRequestExtra {
    pub id: Option<MessageId>,
    pub peer: bool,
    pub timeout: Option<std::time::Duration>,
}
