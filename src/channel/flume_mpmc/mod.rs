use std::future::Future;
use std::pin::Pin;

use flume::RecvTimeoutError;
use std::task::{Context, Poll};
use std::time::Duration;

use crate::channel::{RecvError, SendReturnError, TryRecvError};
use crate::error::*;
use crate::Err;
use futures::future::FusedFuture;

/**
Mixed channels
 */
pub struct ChannelMixedRx<T> {
    inner: flume::Receiver<T>,
}

pub struct ChannelMixedTx<T> {
    inner: ::flume::Sender<T>,
}

pub struct ChannelRxFut<'a, T> {
    inner: ::flume::r#async::RecvFut<'a, T>,
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
    pub async fn send(&self, message: T) -> std::result::Result<(), SendReturnError<T>> {
        match self.inner.send_async(message).await {
            Ok(_) => Ok(()),
            Err(err) => Err(SendReturnError::FailedToSend(err.into_inner())),
        }
    }

    #[inline]
    pub fn send_sync(&self, message: T) -> std::result::Result<(), SendReturnError<T>> {
        match self.inner.send(message) {
            Ok(_) => Ok(()),
            Err(err) => Err(SendReturnError::FailedToSend(err.into_inner())),
        }
    }

    #[inline]
    pub fn send_timeout(
        &self,
        message: T,
        timeout: Duration,
    ) -> std::result::Result<(), SendReturnError<T>> {
        match self.inner.send_timeout(message, timeout) {
            Ok(_) => Ok(()),
            Err(err) => Err(SendReturnError::FailedToSend(err.into_inner())),
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
        let inner = self.inner.recv_async();
        ChannelRxFut { inner }
    }

    #[inline]
    pub fn recv_sync(&self) -> Result<T> {
        match self.inner.recv() {
            Ok(elem) => Ok(elem),
            Err(_) => {
                Err!(RecvError::ChannelDc)
            }
        }
    }

    #[inline]
    pub fn recv_timeout(&self, timeout: Duration) -> Result<T> {
        match self.inner.recv_timeout(timeout) {
            Ok(elem) => Ok(elem),
            Err(err) => match err {
                RecvTimeoutError::Timeout => {
                    Err!(TryRecvError::Timeout)
                }
                RecvTimeoutError::Disconnected => {
                    Err!(TryRecvError::ChannelDc)
                }
            },
        }
    }

    #[inline]
    pub fn try_recv(&self) -> Result<T> {
        match self.inner.try_recv() {
            Ok(ele) => Ok(ele),
            Err(err) => match err {
                flume::TryRecvError::Empty => {
                    Err!(TryRecvError::ChannelEmpty)
                }
                flume::TryRecvError::Disconnected => {
                    Err!(TryRecvError::ChannelDc)
                }
            },
        }
    }
}

impl<'a, T> Future for ChannelRxFut<'a, T> {
    type Output = Result<T>;

    #[inline]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<T>> {
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

pub fn new_bounded<T>(bound: usize) -> (ChannelMixedTx<T>, ChannelMixedRx<T>) {
    let (tx, rx) = flume::bounded(bound);

    (ChannelMixedTx { inner: tx }, ChannelMixedRx { inner: rx })
}

/// Generate a new unbounded channel from flume, wrap it in our own channel and return it
pub fn new_unbounded<T>() -> (ChannelMixedTx<T>, ChannelMixedRx<T>) {
    let (tx, rx) = flume::unbounded();

    (ChannelMixedTx { inner: tx }, ChannelMixedRx { inner: rx })
}
