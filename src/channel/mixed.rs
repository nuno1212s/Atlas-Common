use std::sync::Arc;
use std::time::Duration;
use crate::channel::{SendError, SendReturnError, RecvError, TrySendReturnError};
use crate::Err;

#[allow(dead_code)]

/**
Async and sync mixed channels (Allows us to connect async and sync environments together)
 */

use super::r#async::{ChannelTxFut, ChannelRxFut};

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
    pub fn recv_async(&mut self) -> ChannelRxFut<'_, T> {
        self.inner.recv().into()
    }

    #[inline]
    pub fn try_recv(&self) -> crate::error::Result<T> {
        self.inner.try_recv()
    }
}

impl<T> ChannelMixedTx<T> 
where T: 'static {
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn send_async(&self, value: T) -> ChannelTxFut<'_, T> {
        self.inner.send(value).into()
    }

    #[cfg(not(feature = "channel_mixed_kanal"))]
    #[inline]
    pub fn send_async_return(&self, value: T) -> ChannelTxFut<'_, T> {
        self.inner.send(value).into()
    }

    #[inline]
    pub fn send(&self, value: T) -> crate::error::Result<()> {
        Ok(self.inner.send_sync(value)?)
    }

    #[cfg(not(feature = "channel_mixed_kanal"))]
    #[inline]
    pub fn send_return(&self, value: T) -> Result<(), SendReturnError<T>> {
        self.inner.send_sync_return(value)
    }
    
    pub fn send_timeout(&self, value: T, timeout: Duration) -> crate::error::Result<()> {
        Ok(self.inner.send_timeout_sync(value, timeout)?)
    }

    #[cfg(not(feature = "channel_mixed_kanal"))]
    #[inline]
    pub fn send_timeout_return(
        &self,
        value: T,
        timeout: Duration,
    ) -> Result<(), TrySendReturnError<T>> {
        self.inner.send_timeout_sync_return(value, timeout)
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
