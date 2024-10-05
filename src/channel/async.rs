use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use futures::future::FusedFuture;
use crate::channel::{flume_mpmc, SendReturnError};

/**
 * ASYNCHRONOUS CHANNEL
 */

#[cfg(feature = "channel_flume_mpmc")]
type InnerAsyncChannelTx<T> = flume_mpmc::ChannelMixedTx<T>;
#[cfg(feature = "channel_flume_mpmc")]
type InnerAsyncChannelRx<T> = flume_mpmc::ChannelMixedRx<T>;

#[cfg(feature = "channel_async_kanal")]
type InnerAsyncChannelTx<T> = crate::channel::kanal::r#async::ChannelMixedTx<T>;

#[cfg(feature = "channel_async_kanal")]
type InnerAsyncChannelRx<T> = crate::channel::kanal::r#async::ChannelMixedRx<T>;

/// General purpose channel's sending half.
pub struct ChannelAsyncTx<T> {
    name: Option<Arc<str>>,
    inner: InnerAsyncChannelTx<T>,
}

/// General purpose channel's receiving half.
pub struct ChannelAsyncRx<T> {
    name: Option<Arc<str>>,
    inner: InnerAsyncChannelRx<T>
}


#[cfg(feature = "channel_flume_mpmc")]
type InnerChannelRxFut<'a, T> = flume_mpmc::ChannelRxFut<'a, T>;

#[cfg(feature = "channel_async_channel_mpmc")]
type InnerChannelRxFut<'a, T> = crate::channel::async_channel_mpmc::ChannelRxFut<'a, T>;

#[cfg(feature = "channel_async_kanal")]
type InnerChannelRxFut<'a, T> = crate::channel::kanal::r#async::ChannelRxFut<'a, T>;

/// Future for a general purpose channel's receiving operation.
pub struct ChannelRxFut<'a, T> {
    pub(crate) inner: InnerChannelRxFut<'a, T>
}

#[cfg(feature = "channel_flume_mpmc")]
type InnerChannelTxFut<'a, T> = flume_mpmc::ChannelTxFut<'a, T>;

#[cfg(feature = "channel_async_channel_mpmc")]
type InnerChannelTxFut<'a, T> = crate::channel::async_channel_mpmc::ChannelTxFut<'a, T>;

#[cfg(feature = "channel_async_kanal")]
type InnerChannelTxFut<'a, T> = crate::channel::kanal::r#async::ChannelTxFut<'a, T>;

pub struct ChannelTxFut<'a, T> {
    pub(crate) inner: InnerChannelTxFut<'a, T>
}

impl<T> Clone for ChannelAsyncTx<T> {
    #[inline]
    fn clone(&self) -> Self {
        let inner = self.inner.clone();
        Self { name: self.name.clone(), inner }
    }
}

impl<T> Clone for ChannelAsyncRx<T> {
    #[inline]
    fn clone(&self) -> Self {
        let inner = self.inner.clone();
        Self { name: self.name.clone(), inner }
    }
}

impl<T> ChannelAsyncTx<T> {
    //Can have length because future mpsc doesn't implement it

    //Asynchronously send message through channel
    #[inline]
    pub fn send(&mut self, message: T) -> ChannelTxFut<'_, T> {
        self.inner.send(message).into()
    }
}

impl<T> ChannelAsyncRx<T> {
    //Asynchronously recv message from channel
    #[inline]
    pub fn recv(&mut self) -> ChannelRxFut<'_, T> {
        self.inner.recv().into()
    }
}

impl<'a, T> Future for ChannelRxFut<'a, T> {
    type Output = crate::error::Result<T>;

    #[inline]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<crate::error::Result<T>> {
        Pin::new(&mut self.inner).poll(cx)
    }
}

impl<'a, T> FusedFuture for ChannelRxFut<'a, T> {
    #[inline]
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

impl<'a, T> Future for ChannelTxFut<'a, T> {
    type Output = Result<(), SendReturnError<T>>;

    #[inline]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx)
    }
}

impl<'a, T> FusedFuture for ChannelTxFut<'a, T> {
    #[inline]
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}


/// Creates a new general purpose channel that can queue up to
/// `bound` messages from different async senders.
#[inline]
pub fn new_bounded_async<T>(bound: usize, name: Option<impl Into<String>>) -> (ChannelAsyncTx<T>, ChannelAsyncRx<T>) {

    let name = name.map(|string| Arc::from(string.into()));

    let (tx, rx) = {
        #[cfg(feature = "channel_flume_mpmc")]
        {
            flume_mpmc::new_bounded(bound)
        }
        #[cfg(feature= "channel_mixed_kanal")]
        {
            crate::channel::kanal::new_bounded(bound)
        }
        #[cfg(feature = "channel_async_channel_mpmc")]
        {
            crate::channel::async_channel_mpmc::new_bounded(bound)
        }
    };

    let ttx = ChannelAsyncTx { name: name.clone(), inner: tx };

    let rrx = ChannelAsyncRx { name, inner: rx };

    (ttx, rrx)
}
