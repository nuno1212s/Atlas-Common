use std::sync::Arc;
use std::time::Duration;
use crate::channel::{SendError, SendReturnError, RecvError};
use crate::Err;

#[allow(dead_code)]

/**
Async and sync mixed channels (Allows us to connect async and sync environments together)
 */

#[cfg(feature = "channel_mixed_flume")]
type InnerChannelMixedRx<T> = super::flume_mpmc::ChannelMixedRx<T>;

#[cfg(feature = "channel_mixed_kanal")]
type InnerChannelMixedTx<T> = super::kanal::r#async::ChannelMixedTx<T>;

#[cfg(feature = "channel_mixed_flume")]
type InnerChannelMixedTx<T> = super::flume_mpmc::ChannelMixedTx<T>;

#[cfg(feature = "channel_mixed_kanal")]
type InnerChannelMixedRx<T> = super::kanal::r#async::ChannelMixedRx<T>;

pub struct ChannelMixedRx<T> {
    channel_identifier: Option<Arc<str>>,
    inner: InnerChannelMixedRx<T>
}

pub struct ChannelMixedTx<T> {
    channel_identifier: Option<Arc<str>>,
    inner: InnerChannelMixedTx<T>
}

impl<T> ChannelMixedRx<T> {
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn recv(&self) -> crate::error::Result<T> {
        match self.inner.recv_sync() {
            Ok(res) => Ok(res),
            Err(_err) => {
                Err!(RecvError::ChannelDc)
            }
        }
    }

    #[inline]
    pub fn recv_timeout(&self, timeout: Duration) -> crate::error::Result<T> {
        match self.inner.recv_timeout(timeout) {
            Ok(result) => Ok(result),
            Err(_err) => {
                Err!(RecvError::ChannelDc)
            }
        }
    }

    #[inline]
    pub async fn recv_async(&mut self) -> crate::error::Result<T> {
        match self.inner.recv().await {
            Ok(val) => Ok(val),
            Err(_err) => {
                Err!(RecvError::ChannelDc)
            }
        }
    }

    #[inline]
    pub fn try_recv(&self) -> crate::error::Result<T> {
        self.inner.try_recv()
    }
}

impl<T> ChannelMixedTx<T> {
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub async fn send_async(&self, value: T) -> crate::error::Result<()> {
        self.send_async_return(value)
            .await
            .map_err(SendError::from)
            .map_err(anyhow::Error::from)
    }

    #[inline]
    pub async fn send_async_return(&self, value: T) -> Result<(), SendReturnError<T>> {
        self.inner.send(value).await
    }

    #[inline]
    pub fn send(&self, value: T) -> crate::error::Result<()> {
        self.send_return(value)
            .map_err(SendError::from)
            .map_err(anyhow::Error::from)
    }

    #[inline]
    pub fn send_return(&self, value: T) -> Result<(), SendReturnError<T>> {
        self.inner.send_sync(value)
    }

    #[inline]
    pub fn send_timeout(
        &self,
        value: T,
        timeout: Duration,
    ) -> Result<(), SendReturnError<T>> {
        self.inner.send_timeout(value, timeout)
    }
}

impl<T> Clone for ChannelMixedTx<T> {
    fn clone(&self) -> Self {
        ChannelMixedTx {
            channel_identifier: self.channel_identifier.clone(),
            inner: self.inner.clone(),
        }
    }
}

impl<T> Clone for ChannelMixedRx<T> {
    fn clone(&self) -> Self {
        ChannelMixedRx {
            channel_identifier: self.channel_identifier.clone(),
            inner: self.inner.clone(),
        }
    }
}

pub fn new_bounded_mixed<T>(
    bound: usize,
    name: Option<impl Into<String>>,
) -> (ChannelMixedTx<T>, ChannelMixedRx<T>) {
    let name = name.map(|string| Arc::from(string.into()));

    let (tx, rx) = {
        #[cfg(feature = "channel_mixed_flume")]
        {
            super::flume_mpmc::new_bounded(bound)
        }
        #[cfg(feature = "channel_mixed_kanal")]
        {
            super::kanal::r#async::new_bounded(bound)
        }
    };

    (
        ChannelMixedTx {
            channel_identifier: name.clone(),
            inner: tx,
        },
        ChannelMixedRx {
            channel_identifier: name,
            inner: rx,
        },
    )
}
