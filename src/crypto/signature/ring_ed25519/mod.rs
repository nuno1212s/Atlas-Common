use anyhow::Context;
#[cfg(feature = "serialize_serde")]
use serde::{Serialize, Deserialize};

#[cfg(feature = "serialize_serde")]
use serde_big_array::BigArray;

use ring::{
    signature as rsig,
    signature::KeyPair as RKeyPair,
    signature::ED25519_PUBLIC_KEY_LEN,
};
use ring::error::{KeyRejected, Unspecified};
use ring::signature::Ed25519KeyPair;
use rustls::PrivateKey;
use crate::crypto::signature::{SignError, VerifyError};
use crate::Err;

use crate::error::*;

pub struct KeyPair {
    sk: rsig::Ed25519KeyPair,
    pk: PublicKey,
}

type RPubKey = <rsig::Ed25519KeyPair as RKeyPair>::PublicKey;

#[derive(Copy, Clone)]
pub struct PublicKey {
    pk: rsig::UnparsedPublicKey<RPubKey>,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
#[cfg_attr(feature = "serialize_serde", derive(Serialize, Deserialize))]
pub struct Signature(
    #[cfg_attr(feature = "serialize_serde", serde(with = "BigArray"))]
    [u8; Signature::LENGTH]
);

impl KeyPair {
    pub fn from_pkcs8(priv_key: &[u8]) -> Result<(Self, Vec<u8>)> {
        let sk = match Ed25519KeyPair::from_pkcs8(priv_key) {
            Ok(sk) => sk,
            Err(err) => {
                return Err!(SignError::InvalidPK(format!("{}", err)));
            }
        };

        let pk = sk.public_key().clone();

        let pk_bytes = pk.as_ref();

        let pk = PublicKey::from_bytes_unchecked(pk_bytes);

        Ok((Self {
            sk,
            pk,
        }, pk_bytes.to_vec()))
    }

    pub fn from_bytes(seed_bytes: &[u8]) -> Result<(Self, Vec<u8>)> {
        let sk = match rsig::Ed25519KeyPair::from_seed_unchecked(seed_bytes) {
            Ok(sk) => sk,
            Err(err) => {
                return Err!(SignError::InvalidSignature(format!("{}", err)));
            }
        };

        let pk = sk.public_key().clone();
        let pk_bytes = pk.as_ref();

        let pk = PublicKey::from_bytes_unchecked(pk_bytes);

        Ok((KeyPair { pk, sk }, pk_bytes.to_vec()))
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.pk
    }

    pub fn sign(&self, message: &[u8]) -> Result<Signature> {
        let signature = self.sk.sign(message);
        Ok(Signature::from_bytes_unchecked(signature.as_ref()))
    }
}

impl PublicKey {
    pub fn from_bytes(raw_bytes: &[u8]) -> Result<Self> {
        if raw_bytes.len() < ED25519_PUBLIC_KEY_LEN {
            return Err!(SignError::PublicKeyLen(raw_bytes.len()));
        }

        Ok(Self::from_bytes_unchecked(raw_bytes))
    }

    fn from_bytes_unchecked(raw_bytes: &[u8]) -> Self {
        let mut buf = [0; ED25519_PUBLIC_KEY_LEN];
        buf.copy_from_slice(&raw_bytes[..ED25519_PUBLIC_KEY_LEN]);
        let pk: RPubKey = unsafe {
            // safety remarks: ring represents `RPubKey` as:
            // pub struct PublicKey([u8; ED25519_PUBLIC_KEY_LEN])
            std::mem::transmute(buf)
        };
        let pk = rsig::UnparsedPublicKey::new(&rsig::ED25519, pk);
        PublicKey { pk }
    }

    pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<()> {
        match self.pk.verify(message, signature.as_ref()) {
            Ok(_) => {
                Ok(())
            }
            Err(err) => {
                Err!(VerifyError::VerificationError(format!("{}", err)))
            }
        }
    }
}

impl Signature {
    pub const LENGTH: usize = 64;

    pub fn from_bytes(raw_bytes: &[u8]) -> Result<Self> {
        if raw_bytes.len() < Self::LENGTH {
            return Err!(SignError::SignatureLen(raw_bytes.len()));
        }

        Ok(Self::from_bytes_unchecked(raw_bytes))
    }

    fn from_bytes_unchecked(raw_bytes: &[u8]) -> Self {
        let mut inner = [0; Self::LENGTH];
        inner.copy_from_slice(&raw_bytes[..Self::LENGTH]);
        Self(inner)
    }
}

impl AsRef<[u8]> for Signature {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::{Signature, KeyPair};

    #[test]
    fn test_sign_verify() {
        let k = KeyPair::from_bytes(&[0; 32][..]).expect("Invalid key bytes");

        let message = b"test message";
        let signature = k.sign(message)
            .expect("Signature failed");
        k
            .public_key()
            .verify(message, &signature)
            .expect("Verify failed");
    }
}
