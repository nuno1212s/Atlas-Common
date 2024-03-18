use rayon::ThreadPoolBuilder;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct ThreadPool {
    inner: rayon::ThreadPool,
}

impl ThreadPool {
    pub fn execute<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.inner.spawn(job)
    }

    pub fn join(&self) {
        // no-op
    }
}

pub struct Builder {
    threads: Option<usize>,
}

impl Builder {
    pub fn new() -> Builder {
        Builder { threads: None }
    }

    pub fn build(self) -> ThreadPool {
        let mut builder = ThreadPoolBuilder::new();

        builder = builder.thread_name(|t| format!("Atlas-CPU-Worker-{}", t));

        let inner = match self.threads {
            Some(n) => builder.num_threads(n),
            None => builder,
        }
        .build()
        .unwrap();

        ThreadPool { inner }
    }

    pub fn num_threads(mut self, num_threads: usize) -> Self {
        self.threads = Some(num_threads);
        self
    }
}
