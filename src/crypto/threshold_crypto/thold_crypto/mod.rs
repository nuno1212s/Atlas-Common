use anyhow::{anyhow, Context};
#[cfg(feature = "serialize_serde")]
use serde::{Deserialize, Serialize};
use threshold_crypto::poly::{Commitment, Poly};
use threshold_crypto::{Fr, IntoFr};

use crate::error::*;

pub mod dkg;
//mod async_dkg;

#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
#[cfg_attr(feature = "serialize_serde", derive(Serialize, Deserialize))]
pub struct PublicKey {
    key: threshold_crypto::PublicKey,
}

#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct PrivateKey {
    key: threshold_crypto::SecretKey,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(transparent)]
#[cfg_attr(feature = "serialize_serde", derive(Serialize, Deserialize))]
pub struct PublicKeyPart {
    key: threshold_crypto::PublicKeyShare,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct PrivateKeyPart {
    key: threshold_crypto::SecretKeyShare,
}

#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
#[cfg_attr(feature = "serialize_serde", derive(Serialize, Deserialize))]
pub struct PartialSignature {
    sig: threshold_crypto::SignatureShare,
}

#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
#[cfg_attr(feature = "serialize_serde", derive(Serialize, Deserialize))]
pub struct Signature {
    sig: threshold_crypto::Signature,
}

#[repr(transparent)]
pub struct SecretKeySet {
    sk_set: threshold_crypto::SecretKeySet,
}

/// The public key set
#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(transparent)]
#[cfg_attr(feature = "serialize_serde", derive(Serialize, Deserialize))]
pub struct PublicKeySet {
    pk_set: threshold_crypto::PublicKeySet,
}

impl PublicKeyPart {
    pub fn verify(&self, msg: &[u8], sig: &PartialSignature) -> Result<()> {
        if self.key.verify(&sig.sig, msg) {
            Ok(())
        } else {
            Err(anyhow!("Partial signature verification failed"))
        }
    }
}

impl SecretKeySet {
    pub fn generate_random(n: usize) -> Result<SecretKeySet> {
        let mut rand = rand::rngs::OsRng::default();

        let sk_set = threshold_crypto::SecretKeySet::random(n, &mut rand);

        Ok(SecretKeySet { sk_set })
    }

    pub fn get_key_share(&self, i: usize) -> Result<PrivateKeyPart> {
        let key = self.sk_set.secret_key_share(i);

        Ok(PrivateKeyPart { key })
    }

    pub fn public_key_set(&self) -> PublicKeySet {
        PublicKeySet {
            pk_set: self.sk_set.public_keys(),
        }
    }
}

impl PrivateKeyPart {
    pub fn public_key_part(&self) -> PublicKeyPart {
        PublicKeyPart {
            key: self.key.public_key_share(),
        }
    }

    pub fn partially_sign(&self, msg: &[u8]) -> Result<PartialSignature> {
        let sig = self.key.sign(msg);

        Ok(PartialSignature { sig })
    }

    pub fn from_mut(mut sk: &mut Fr) -> Self {
        let key = threshold_crypto::SecretKeyShare::from_mut(sk);

        PrivateKeyPart { key }
    }
}

impl PublicKey {
    #[inline]
    pub fn verify_combined_signatures(&self, sig: &Signature, msg: &[u8]) -> bool {
        self.key.verify(&sig.sig, msg)
    }

    #[inline]
    pub fn to_bytes(&self) -> [u8; threshold_crypto::PK_SIZE] {
        self.key.to_bytes()
    }

    #[inline]
    pub fn from_bytes(bytes: &[u8; threshold_crypto::PK_SIZE]) -> Result<PublicKey> {
        let key = threshold_crypto::PublicKey::from_bytes(bytes)?;

        Ok(PublicKey { key })
    }
}

impl PublicKeySet {
    #[inline(always)]
    pub fn public_key(&self) -> PublicKey {
        let key = self.pk_set.public_key();

        PublicKey { key }
    }

    #[inline(always)]
    pub fn get_public_key_part(&self, i: usize) -> Result<PublicKeyPart> {
        let key = self.pk_set.public_key_share(i);

        Ok(PublicKeyPart { key })
    }

    #[inline(always)]
    pub fn verify_partial_signature(
        &self,
        index: usize,
        msg: &[u8],
        sig: &PartialSignature,
    ) -> Result<()> {
        self.get_public_key_part(index)?.verify(msg, sig)
    }

    #[inline(always)]
    pub fn combine_signatures<'a, T, I>(&self, sigs: I) -> Result<Signature>
    where
        I: IntoIterator<Item = (T, &'a PartialSignature)>,
        T: IntoFr,
    {
        let sigs = sigs
            .into_iter()
            .map(|(index, sign)| (index, &sign.sig))
            .collect::<Vec<_>>();

        let sig = self.pk_set.combine_signatures(sigs)?;

        Ok(Signature { sig })
    }

    #[inline(always)]
    pub fn verify_combined_signature(&self, msg: &[u8], sig: &Signature) -> Result<()> {
        if self.public_key().verify_combined_signatures(&sig, msg) {
            Ok(())
        } else {
            Err(anyhow!("Signature verification failed"))
        }
    }
}

impl From<Commitment> for PublicKeySet {
    fn from(value: Commitment) -> Self {
        PublicKeySet {
            pk_set: threshold_crypto::PublicKeySet::from(value),
        }
    }
}

impl From<PublicKeySet> for super::PublicKeySet {
    fn from(value: PublicKeySet) -> Self {
        super::PublicKeySet { key: value }
    }
}

impl From<PrivateKeyPart> for super::PrivateKeyPart {
    fn from(value: PrivateKeyPart) -> Self {
        super::PrivateKeyPart { key: value }
    }
}
