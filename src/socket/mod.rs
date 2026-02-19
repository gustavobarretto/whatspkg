//! WebSocket socket layer with length-prefixed framing.
//!
//! Each frame is: 3-byte big-endian length (max 16MiB) then payload.
//! Used as the raw transport under Noise; requires the `full` feature.

#[cfg(feature = "full")]
mod framed;
#[cfg(feature = "full")]
mod noise;
#[cfg(feature = "full")]
mod ws;

#[cfg(feature = "full")]
pub use framed::{FramedRecv, FramedSend};
#[cfg(feature = "full")]
pub use noise::{
    connect_noise, connect_noise_default, run_client_handshake, NoiseRecv, NoiseTransport,
};
#[cfg(feature = "full")]
pub use ws::{connect, connect_default};

/// Maximum frame body size (3-byte length = 2^24 - 1).
pub const MAX_FRAME_SIZE: usize = (1 << 24) - 1;

/// Writes a 3-byte big-endian length prefix into `buf` (must have at least 3 bytes).
#[inline]
pub fn write_frame_len(buf: &mut [u8], len: usize) {
    assert!(buf.len() >= 3 && len <= MAX_FRAME_SIZE);
    buf[0] = (len >> 16) as u8;
    buf[1] = (len >> 8) as u8;
    buf[2] = len as u8;
}

/// Reads a 3-byte big-endian length from `buf` (must have at least 3 bytes).
#[inline]
pub fn read_frame_len(buf: &[u8]) -> usize {
    assert!(buf.len() >= 3);
    ((buf[0] as usize) << 16) | ((buf[1] as usize) << 8) | (buf[2] as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_len_roundtrip() {
        let mut buf = [0u8; 3];
        for len in [0usize, 1, 255, 256, 65535, 65536, MAX_FRAME_SIZE] {
            write_frame_len(&mut buf, len);
            assert_eq!(read_frame_len(&buf), len);
        }
    }
}
