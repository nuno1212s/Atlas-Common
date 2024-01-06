#[cfg(feature = "serialize_serde")]
use serde::{Deserialize, Serialize};

/// An abstraction for the serializable types in this framework, so that they don't have to
/// have multiple declarations (which made parsing the project a whole lot harder for the compiler
/// leading to lots of stupid "errors" which were actually just the compiler being "dumb")
#[cfg(feature = "serialize_serde")]
pub trait SerType: for<'a> Deserialize<'a> + Serialize + Send + Clone {}

#[cfg(feature = "serialize_capnp")]
pub trait SerType: Send + Clone {}

/// Automatically implement the SerType trait for all types that implement the serde traits
/// which, since we do not require any function impls, is the only thing we require
#[cfg(feature = "serialize_serde")]
impl<T> SerType for T where T: for<'a> Deserialize<'a> + Serialize + Send + Clone {}

#[cfg(feature = "serialize_capnp")]
impl<T> SerType for T where T: Send + Clone {}