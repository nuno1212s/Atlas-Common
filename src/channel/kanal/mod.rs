pub(super) mod r#async;

use crate::channel::{RecvError, TryRecvError};
use crate::error::*;
use crate::Err;
use kanal::*;
use std::ops::Deref;
use std::time::Duration;

pub struct ChannelSyncRx<T> {
    inner: Receiver<T>,
}

pub struct ChannelSyncTx<T> {
    inner: Sender<T>,
}

impl<T> Clone for ChannelSyncTx<T> {
    fn clone(&self) -> Self {
        ChannelSyncTx {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Clone for ChannelSyncRx<T> {
    fn clone(&self) -> Self {
        ChannelSyncRx {
            inner: self.inner.clone(),
        }
    }
}

impl<T> ChannelSyncTx<T> {
    #[inline]
    pub fn send(&self, value: T) -> std::result::Result<(), super::SendError> {
        match self.inner.send(value) {
            Ok(_) => Ok(()),
            Err(_) => Err(super::SendError::FailedToSend),
        }
    }

    #[inline]
    pub fn send_timeout(
        &self,
        value: T,
        timeout: Duration,
    ) -> std::result::Result<(), super::TrySendError> {
        match self.inner.send_timeout(value, timeout) {
            Ok(_) => Ok(()),
            Err(err) => match err {
                SendErrorTimeout::Timeout => Err(super::TrySendError::Timeout),
                SendErrorTimeout::Closed | SendErrorTimeout::ReceiveClosed => {
                    Err(super::TrySendError::Disconnected)
                }
            },
        }
    }

    #[inline]
    pub fn try_send(&self, value: T) -> std::result::Result<(), super::TrySendError> {
        match self.inner.try_send(value) {
            Ok(true) => Ok(()),
            Ok(false) => Err(super::TrySendError::Full),
            Err(err) => match err {
                SendError::ReceiveClosed | SendError::Closed => {
                    Err(super::TrySendError::Disconnected)
                }
            },
        }
    }
}

impl<T> ChannelSyncTx<T>
where
    T: Clone,
{
    pub fn try_send_return(
        &self,
        value: T,
    ) -> std::result::Result<(), super::TrySendReturnError<T>> {
        let value_clone = value.clone();

        match self.inner.try_send(value) {
            Ok(true) => Ok(()),
            Ok(false) => Err(super::TrySendReturnError::Full(value_clone)),
            Err(err) => match err {
                SendError::ReceiveClosed | SendError::Closed => {
                    Err(super::TrySendReturnError::Disconnected(value_clone))
                }
            },
        }
    }
}

impl<T> ChannelSyncRx<T> {
    #[inline]
    pub fn try_recv(&self) -> std::result::Result<T, TryRecvError> {
        match self.inner.try_recv() {
            Ok(Some(res)) => Ok(res),
            Ok(None) => Err(TryRecvError::ChannelEmpty),
            Err(err) => match err {
                ReceiveError::Closed | ReceiveError::SendClosed => Err(TryRecvError::ChannelDc),
            },
        }
    }

    #[inline]
    pub fn recv(&self) -> Result<T> {
        match self.inner.recv() {
            Ok(res) => Ok(res),
            Err(_err) => {
                Err!(RecvError::ChannelDc)
            }
        }
    }

    #[inline]
    pub fn recv_timeout(&self, timeout: Duration) -> std::result::Result<T, TryRecvError> {
        match self.inner.recv_timeout(timeout) {
            Ok(result) => Ok(result),
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
}

impl<T> Deref for ChannelSyncRx<T> {
    type Target = Receiver<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> Deref for ChannelSyncTx<T> {
    type Target = Sender<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[inline]
pub(super) fn new_bounded<T>(bound: usize) -> (ChannelSyncTx<T>, ChannelSyncRx<T>) {
    let (tx, rx) = bounded(bound);

    (ChannelSyncTx { inner: tx }, ChannelSyncRx { inner: rx })
}

#[inline]
pub(super) fn new_unbounded<T>() -> (ChannelSyncTx<T>, ChannelSyncRx<T>) {
    let (tx, rx) = unbounded();

    (ChannelSyncTx { inner: tx }, ChannelSyncRx { inner: rx })
}

impl<T> From<Sender<T>> for ChannelSyncTx<T> {
    fn from(value: Sender<T>) -> Self {
        Self { inner: value }
    }
}

impl<T> From<Receiver<T>> for ChannelSyncRx<T> {
    fn from(value: Receiver<T>) -> Self {
        Self { inner: value }
    }
}

impl<T> From<ChannelSyncTx<T>> for Sender<T> {
    fn from(value: ChannelSyncTx<T>) -> Self {
        value.inner
    }
}

impl<T> From<ChannelSyncRx<T>> for Receiver<T> {
    fn from(value: ChannelSyncRx<T>) -> Self {
        value.inner
    }
}

impl<T> AsRef<Sender<T>> for ChannelSyncTx<T> {
    fn as_ref(&self) -> &Sender<T> {
        &self.inner
    }
}

impl<T> AsRef<Receiver<T>> for ChannelSyncRx<T> {
    fn as_ref(&self) -> &Receiver<T> {
        &self.inner
    }
}
