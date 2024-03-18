//! This crate, `febft-common` takes advantage of the feature flags
//! in `Cargo.toml` to provide a super flexible, modular API.
//!
//! # Feature flags
//!
//! At the moment, a user is able to customize:
//!
//! - The asynchronous runtime used by this crate:
//!     + E.g. To use `tokio`, enter the feature flag `async_runtime_tokio`.
//! - The thread pool used to execute CPU intensive tasks:
//!     + E.g. `threadpool_cthpool`.
//! - The sockets library used to communicate with other nodes:
//!     + E.g. `socket_async_std_tcp`.
//! - If the serialization of wire messages is possible with `serde`:
//!     + With `serialize_serde`.
//! - The crypto library used to perform public key crypto operations:
//!     + E.g. `crypto_signature_ring_ed25519`.
//! - The crypto library used to calculate hash digests of messages:
//!     + E.g. `crypto_hash_ring_sha2`.
//!
//! However, for convenience, some sane default feature flags are already
//! configured, which should perform well under any environment. Mind you,
//! the user, that this is a BFT library, so software variation is encouraged;
//! in a typical system setup, you would probably employ different backend
//! libraries performing identical duties.
#![feature(type_alias_impl_trait)]

use crate::error::*;
use crate::globals::Flag;
use log::debug;

pub mod async_runtime;
pub mod channel;
pub mod collections;
pub mod config_utils;
pub mod crypto;
pub mod error;
pub mod globals;
pub mod maybe_vec;
pub mod mem_pool;
pub mod node_id;
pub mod ordering;
pub mod peer_addr;
pub mod persistentdb;
pub mod prng;
pub mod serialization_helper;
pub mod socket;
pub mod system_params;
pub mod threadpool;

static INITIALIZED: Flag = Flag::new();

/// Configure the init process of the library.
pub struct InitConfig {
    /// Number of threads used by the async runtime.
    pub async_threads: usize,
    /// Number of threads used by the thread pool.
    pub threadpool_threads: usize,

    ///Unique ID, used to specify the log file this replica should use
    pub id: Option<String>,
}

/// Handle to the global data.
///
/// When dropped, the data is deinitialized.
#[repr(transparent)]
pub struct InitGuard;

/// Initializes global data.
///
/// Should always be called before other methods, otherwise runtime
/// panics may ensue.
pub unsafe fn init(c: InitConfig) -> Result<Option<InitGuard>> {
    if INITIALIZED.test() {
        return Ok(None);
    }

    threadpool::init(c.threadpool_threads)?;
    async_runtime::init(c.async_threads)?;

    debug!(
        "Async threads {}, sync threads {}",
        c.async_threads, c.threadpool_threads
    );

    socket::init()?;
    INITIALIZED.set();
    Ok(Some(InitGuard))
}

impl Drop for InitGuard {
    fn drop(&mut self) {
        unsafe { drop().unwrap() }
    }
}

unsafe fn drop() -> Result<()> {
    INITIALIZED.unset();
    threadpool::drop()?;
    async_runtime::drop()?;
    socket::drop()?;
    Ok(())
}
