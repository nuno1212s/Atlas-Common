use crate::error::*;
use anyhow::Context;
use std::sync::atomic::{AtomicUsize, Ordering};

pub type JoinHandle<T> = ::tokio::task::JoinHandle<T>;

// FIXME: if users call `tokio::spawn`, nothing will happen!
// we need to include an `EnterGuard<'static>` in the returned
// runtime type
pub type Runtime = ::tokio::runtime::Runtime;

pub fn init(num_threads: usize) -> Result<Runtime> {
    let result = ::tokio::runtime::Builder::new_multi_thread()
        .worker_threads(num_threads)
        .thread_name_fn(|| {
            static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);

            let id = ATOMIC_ID.fetch_add(1, Ordering::SeqCst);

            format!("Atlas-IO-Worker-{}", id)
        })
        .enable_all()
        .build()
        .context("Failed to build tokio runtime")?;

    let _handle = result.handle();

    Ok(result)
}
