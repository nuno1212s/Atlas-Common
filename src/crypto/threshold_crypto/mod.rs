pub mod thold_crypto;
//mod frost;

use crate::error::*;
#[cfg(feature = "serialize_serde")]
use serde::{Deserialize, Serialize};
use threshold_crypto::{Fr, IntoFr};
use crate::crypto::threshold_crypto::thold_crypto::SecretKeySet;

#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
#[cfg_attr(feature = "serialize_serde", derive(Serialize, Deserialize))]
pub struct PublicKeyPart {
    key: thold_crypto::PublicKeyPart,
}

#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct PrivateKeyPart {
    key: thold_crypto::PrivateKeyPart,
}

#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
#[cfg_attr(feature = "serialize_serde", derive(Serialize, Deserialize))]
pub struct PublicKeySet {
    key: thold_crypto::PublicKeySet,
}

#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
#[cfg_attr(feature = "serialize_serde", derive(Serialize, Deserialize))]
pub struct PublicKey {
    key: thold_crypto::PublicKey,
}

#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct PrivateKeySet {
    key: thold_crypto::SecretKeySet,
}

#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
#[cfg_attr(feature = "serialize_serde", derive(Serialize, Deserialize))]
pub struct PartialSignature {
    sig: thold_crypto::PartialSignature,
}

#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
#[cfg_attr(feature = "serialize_serde", derive(Serialize, Deserialize))]
pub struct CombinedSignature {
    sig: thold_crypto::Signature,
}

impl PublicKey {}

impl PublicKeySet {
    pub fn public_key(&self) -> PublicKey {
        PublicKey {
            key: self.key.public_key(),
        }
    }

    pub fn public_key_share(&self, index: usize) -> Result<PublicKeyPart> {
        Ok(PublicKeyPart {
            key: self.key.get_public_key_part(index)?,
        })
    }

    pub fn verify(&self, msg: &[u8], sig: &CombinedSignature) -> Result<()> {
        self.key.verify_combined_signature(msg, &sig.sig)?;

        Ok(())
    }

    pub fn combine_signatures<'a, T, I>(&self, sigs: I) -> Result<CombinedSignature>
    where
        I: IntoIterator<Item = (T, &'a PartialSignature)>,
        T: IntoFr,
    {
        let map = sigs
            .into_iter()
            .map(|(id, sig)| (id, &sig.sig))
            .collect::<Vec<_>>();

        Ok(CombinedSignature {
            sig: self.key.combine_signatures(map)?,
        })
    }
}

impl PublicKeyPart {
    pub fn verify(&self, msg: &[u8], sig: &PartialSignature) -> Result<()> {
        self.key.verify(msg, &sig.sig)
    }
}

impl PrivateKeyPart {
    pub fn public_key_part(&self) -> PublicKeyPart {
        PublicKeyPart {
            key: self.key.public_key_part(),
        }
    }

    pub fn partially_sign(&self, msg: &[u8]) -> Result<PartialSignature> {
        Ok(PartialSignature {
            sig: self.key.partially_sign(msg)?,
        })
    }

    pub fn from_mut(sk: &mut Fr) -> Self {
        PrivateKeyPart {
            key: thold_crypto::PrivateKeyPart::from_mut(sk),
        }
    }
}

impl PrivateKeySet {
    
    pub fn gen_random(n: usize) -> Result<Self> {
        SecretKeySet::generate_random(n).map(|key| Self {
            key
        })
    }
    
    pub fn public_key_set(&self) -> PublicKeySet {
        PublicKeySet {
            key: self.key.public_key_set()
        }
    }
    
    pub fn private_key_part(&self, index: usize) -> Result<PrivateKeyPart> {
        self.key.get_key_share(index).map(|key| PrivateKeyPart {
            key
        })
    }
    
}