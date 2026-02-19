//! Constants for the WhatsApp binary protocol and connection.

/// Noise handshake pattern used by WhatsApp Web.
pub const NOISE_START_PATTERN: &[u8] = b"Noise_XX_25519_AESGCM_SHA256\x00\x00\x00\x00";

/// Magic byte in the connection header.
pub const WA_MAGIC_VALUE: u8 = 6;

/// Dictionary version for token compression (must match server).
pub const DICT_VERSION: u8 = 3;

/// Connection header: "WA" + magic + dict version (4 bytes).
pub const WA_CONN_HEADER: [u8; 4] = [b'W', b'A', WA_MAGIC_VALUE, DICT_VERSION];
