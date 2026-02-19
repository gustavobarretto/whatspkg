//! Pairing crypto: device identity verification (HMAC), key generation (X25519, Ed25519),
//! and signed device identity for persistence.

use crate::error::{Error, PairingError};
use crate::Result;
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use hmac::{Hmac, Mac};
use rand::RngCore;
use sha2::Sha256;
use x25519_dalek::{PublicKey, StaticSecret};

/// HMAC-SHA256 tag length in bytes.
const HMAC_LEN: usize = 32;

/// Verified device identity from the server payload (after HMAC check).
#[derive(Clone, Debug)]
pub struct VerifiedIdentity {
    /// Raw payload that was verified (excluding the HMAC tag).
    pub payload: Vec<u8>,
}

/// Keys generated for pairing: Noise key, identity key, and adv secret.
#[derive(Clone, Debug)]
pub struct PairingKeys {
    /// X25519 Noise public key (32 bytes).
    pub noise_public: [u8; 32],
    /// X25519 Noise private key (32 bytes); store securely, not exposed in Device.
    pub noise_private: [u8; 32],
    /// Ed25519 identity public key (32 bytes).
    pub identity_public: [u8; 32],
    /// Ed25519 identity private key (32 bytes); store in Device.identity_key_priv.
    pub identity_private: [u8; 32],
    /// Adv secret for pairing (32 bytes).
    pub adv_secret: [u8; 32],
}

/// Verify device identity payload: last HMAC_LEN bytes are HMAC-SHA256 of the rest with the given key.
/// Returns the payload without the tag, or error if verification fails.
pub fn verify_device_identity(
    payload_with_tag: &[u8],
    hmac_key: &[u8],
) -> Result<VerifiedIdentity> {
    if payload_with_tag.len() < HMAC_LEN {
        return Err(Error::Pairing(PairingError::InvalidDeviceIdentityHmac));
    }
    let split = payload_with_tag.len() - HMAC_LEN;
    let payload = &payload_with_tag[..split];
    let tag = &payload_with_tag[split..];

    let mut mac = Hmac::<Sha256>::new_from_slice(hmac_key)
        .map_err(|_| Error::Pairing(PairingError::Protocol("invalid HMAC key length".into())))?;
    mac.update(payload);
    mac.verify_slice(tag)
        .map_err(|_| Error::Pairing(PairingError::InvalidDeviceIdentityHmac))?;

    Ok(VerifiedIdentity {
        payload: payload.to_vec(),
    })
}

/// Generate fresh pairing keys: Noise (X25519), identity (Ed25519), and adv secret.
pub fn generate_pairing_keys() -> PairingKeys {
    let mut noise_private = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut noise_private);
    let secret = StaticSecret::from(noise_private);
    let noise_public = PublicKey::from(&secret).to_bytes();

    let identity_signing = SigningKey::generate(&mut rand::thread_rng());
    let identity_public = identity_signing.verifying_key().to_bytes();
    let identity_private = identity_signing.to_bytes();

    let mut adv_secret = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut adv_secret);

    PairingKeys {
        noise_public,
        noise_private,
        identity_public,
        identity_private,
        adv_secret,
    }
}

/// Build a signed device identity blob for storage: verifying_key (32) || signature (64) || payload.
/// The payload is signed with the identity private key so the server can verify later.
pub fn sign_device_identity(payload: &[u8], identity_private: &[u8; 32]) -> Result<Vec<u8>> {
    let signing_key = SigningKey::from_bytes(identity_private);
    let verifying_key = signing_key.verifying_key();
    let signature = signing_key.sign(payload);

    let mut out = Vec::with_capacity(32 + 64 + payload.len());
    out.extend_from_slice(verifying_key.as_bytes());
    out.extend_from_slice(&signature.to_bytes());
    out.extend_from_slice(payload);
    Ok(out)
}

/// Verify a stored signed device identity blob (verifying_key || signature || payload).
/// Returns the inner payload on success.
pub fn verify_signed_identity(signed_blob: &[u8]) -> Result<Vec<u8>> {
    if signed_blob.len() < 32 + 64 {
        return Err(Error::Pairing(PairingError::InvalidDeviceSignature));
    }
    let key_bytes: [u8; 32] = signed_blob[..32]
        .try_into()
        .map_err(|_| Error::Pairing(PairingError::InvalidDeviceSignature))?;
    let verifying_key = VerifyingKey::from_bytes(&key_bytes)
        .map_err(|_| Error::Pairing(PairingError::InvalidDeviceSignature))?;
    let sig_bytes: [u8; 64] = signed_blob[32..96]
        .try_into()
        .map_err(|_| Error::Pairing(PairingError::InvalidDeviceSignature))?;
    let signature = ed25519_dalek::Signature::from_bytes(&sig_bytes);
    let payload = &signed_blob[96..];
    verifying_key
        .verify_strict(payload, &signature)
        .map_err(|_| Error::Pairing(PairingError::InvalidDeviceSignature))?;
    Ok(payload.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hmac_verify_roundtrip() {
        let key = b"test-hmac-key-32-bytes-long!!!!!!";
        let payload = b"device-identity-payload";
        let mut mac = Hmac::<Sha256>::new_from_slice(key).unwrap();
        mac.update(payload);
        let tag = mac.finalize().into_bytes();
        let mut with_tag = payload.to_vec();
        with_tag.extend_from_slice(&tag);
        let verified = verify_device_identity(&with_tag, key).unwrap();
        assert_eq!(verified.payload, payload);
    }

    #[test]
    fn hmac_reject_tampered() {
        let key = b"test-hmac-key-32-bytes-long!!!!!!";
        let payload = b"device-identity-payload";
        let mut mac = Hmac::<Sha256>::new_from_slice(key).unwrap();
        mac.update(payload);
        let tag = mac.finalize().into_bytes();
        let mut with_tag = payload.to_vec();
        with_tag.extend_from_slice(&tag);
        with_tag[0] ^= 1;
        assert!(verify_device_identity(&with_tag, key).is_err());
    }

    #[test]
    fn pairing_keys_generated() {
        let keys = generate_pairing_keys();
        assert_eq!(keys.noise_public.len(), 32);
        assert_eq!(keys.identity_public.len(), 32);
    }

    #[test]
    fn sign_verify_identity_roundtrip() {
        let keys = generate_pairing_keys();
        let payload = b"account-payload-to-store";
        let signed = sign_device_identity(payload, &keys.identity_private).unwrap();
        let verified = verify_signed_identity(&signed).unwrap();
        assert_eq!(verified, payload);
    }
}
