use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use futures::future::FusedFuture;
use kanal::{ReceiveError, ReceiveErrorTimeout, ReceiveFuture, SendErrorTimeout, SendFuture};
use crate::channel::{SendError, TrySendError, RecvError, TryRecvError};
use crate::Err;

/**
Mixed channels
 */
pub struct ChannelMixedRx<T> {
    inner: kanal::AsyncReceiver<T>,
}

pub struct ChannelMixedTx<T> {
    inner: kanal::AsyncSender<T>,
}

pub struct ChannelRxFut<'a, T> {
    inner: ReceiveFuture<'a, T>,
}

pub struct ChannelTxFut<'a, T> {
    inner: SendFuture<'a, T>,
}

impl<T> Clone for ChannelMixedTx<T> {
    fn clone(&self) -> Self {
        ChannelMixedTx {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Clone for ChannelMixedRx<T> {
    fn clone(&self) -> Self {
        ChannelMixedRx {
            inner: self.inner.clone(),
        }
    }
}

impl<T> ChannelMixedTx<T> {
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_dc(&self) -> bool {
        self.inner.is_disconnected()
    }

    #[inline]
    pub fn send(&self, message: T) -> ChannelTxFut<'_, T> {
        self.inner.send(message).into()
    }

    #[inline]
    pub fn send_sync(&self, message: T) -> Result<(), SendError> {
        match self.inner.as_sync().send(message) {
            Ok(_) => Ok(()),
            Err(_) => Err(SendError::FailedToSend),
        }
    }

    #[inline]
    pub fn send_timeout_sync(
        &self,
        message: T,
        timeout: Duration,
    ) -> std::result::Result<(), TrySendError> {
        match self.inner.as_sync().send_timeout(message, timeout) {
            Ok(_) => Ok(()),
            Err(err) => {
                match err {
                    SendErrorTimeout::Closed  | SendErrorTimeout::ReceiveClosed => Err(TrySendError::Disconnected),
                    SendErrorTimeout::Timeout => Err(TrySendError::Timeout),
                }
            },
        }
    }
}

impl<T> ChannelMixedRx<T> {
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_dc(&self) -> bool {
        self.inner.is_disconnected()
    }

    #[inline]
    pub fn recv(&mut self) -> ChannelRxFut<'_, T> {
        self.inner.recv().into()
    }

    #[inline]
    pub fn recv_sync(&self) -> crate::error::Result<T> {
        match self.inner.as_sync().recv() {
            Ok(elem) => Ok(elem),
            Err(_) => {
                Err!(RecvError::ChannelDc)
            }
        }
    }

    #[inline]
    pub fn recv_timeout(&self, timeout: Duration) -> crate::error::Result<T> {
        match self.inner.as_sync().recv_timeout(timeout) {
            Ok(elem) => Ok(elem),
            Err(err) => match err {
                ReceiveErrorTimeout::Timeout => {
                    Err!(TryRecvError::Timeout)
                }
                ReceiveErrorTimeout::Closed | ReceiveErrorTimeout::SendClosed => {
                    Err!(TryRecvError::ChannelDc)
                }
            },
        }
    }

    #[inline]
    pub fn try_recv(&self) -> crate::error::Result<T> {
        match self.inner.as_sync().try_recv() {
            Ok(Some(ele)) => Ok(ele),
            Ok(None) => Err!(TryRecvError::ChannelEmpty),
            Err(err) => match err { 
                ReceiveError::Closed | ReceiveError::SendClosed => {
                    Err!(TryRecvError::ChannelDc)
                }
            },
        }
    }
}

impl<'a, T> Future for ChannelRxFut<'a, T> {
    type Output = crate::error::Result<T>;

    #[inline]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<crate::error::Result<T>> {
        Pin::new(&mut self.inner).poll(cx).map(|r| match r {
            Ok(res) => Ok(res),
            Err(_) => {
                Err!(RecvError::ChannelDc)
            }
        })
    }
}

impl<'a, T> FusedFuture for ChannelRxFut<'a, T> {
    #[inline]
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

impl<'a, T> From<ReceiveFuture<'a, T>> for ChannelRxFut<'a, T> {
    fn from(inner: ReceiveFuture<'a, T>) -> Self {
        Self { inner }
    }
}

impl<'a, T> From<SendFuture<'a, T>> for ChannelTxFut<'a, T> {
    fn from(inner: SendFuture<'a, T>) -> Self {
        Self { inner }
    }
}

pub fn new_bounded<T>(bound: usize) -> (ChannelMixedTx<T>, ChannelMixedRx<T>) {
    let (tx, rx) = kanal::bounded_async(bound);

    (ChannelMixedTx { inner: tx }, ChannelMixedRx { inner: rx})
}

/// Generate a new unbounded channel from flume, wrap it in our own channel and return it
pub fn new_unbounded<T>() -> (ChannelMixedTx<T>, ChannelMixedRx<T>) {
    let (tx, rx) = kanal::unbounded_async();

    (ChannelMixedTx { inner: tx }, ChannelMixedRx { inner: rx })
}
