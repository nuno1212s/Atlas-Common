use crate::error::*;
use anyhow::anyhow;
#[cfg(feature = "serialize_serde")]
use serde::{Deserialize, Serialize};

/// This struct contains the system parameters of
/// a replica or client in `febft`, i.e. `n` and `f`
/// such that `n >= 3*f + 1`.
#[cfg_attr(feature = "serialize_serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug)]
pub struct SystemParams {
    n: usize,
    f: usize,
}

impl SystemParams {
    /// Creates a new instance of `SystemParams`.
    pub fn new(n: usize, f: usize) -> Result<Self> {
        if n < 3 * f + 1 {
            return Err(anyhow!("Invalid params: n < 3f + 1"));
        }

        Ok(SystemParams { n, f })
    }

    /// Returns the quorum size associated with these
    /// `SystemParams`.
    pub fn quorum(&self) -> usize {
        //2*self.f + 1
        //self.n - self.f
        (self.f << 1) + 1
    }

    /// Returns the `n` parameter.
    pub fn n(&self) -> usize {
        self.n
    }

    /// Returns the `f` parameter.
    pub fn f(&self) -> usize {
        self.f
    }
}
