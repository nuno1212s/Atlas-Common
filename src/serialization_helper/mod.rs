#[cfg(feature = "serialize_serde")]
use serde::{Deserialize, Serialize};

/// A marker trait for the serializable types in this framework, so that they don't have to
/// have multiple declarations (which made parsing the project a whole lot harder for the compiler
/// leading to lots of stupid "errors" which were actually just the compiler being hard to love)
///
/// All SerTypes have to be 'static since they are used in multiple threads, which can live for the
/// entire duration of the program, meaning we can't have non static references.
/// For the same reason, they all have to be Sync + Send, as we will want to share them between threads
///
/// Any raw struct type with no internal references will follow this requirement
#[cfg(feature = "serialize_serde")]
pub trait SerMsg: 'static + for<'a> Deserialize<'a> + Serialize + Send + Sync + Clone {}

#[cfg(feature = "serialize_capnp")]
pub trait SerMessage: 'static + Send + Clone {}

#[cfg(feature = "serialize_serde")]
pub trait NonSyncSerMsg: 'static + for<'a> Deserialize<'a> + Serialize + Clone + Send {}

/// Automatically implement the SerType trait for all types that implement the serde traits
/// which, since we do not require any function impls, is the only thing we require
#[cfg(feature = "serialize_serde")]
impl<T> SerMsg for T where T: 'static + for<'a> Deserialize<'a> + Serialize + Send + Sync + Clone {}

#[cfg(feature = "serialize_capnp")]
impl<T> SerMsg for T where T: 'static + Send + Clone {}

#[cfg(feature = "serialize_serde")]
impl<T> NonSyncSerMsg for T where T: 'static + for<'a> Deserialize<'a> + Serialize + Clone + Send {}

#[cfg(feature = "serialize_capnp")]
impl<T> NonSyncSerMsg for T where T: 'static + Clone {}