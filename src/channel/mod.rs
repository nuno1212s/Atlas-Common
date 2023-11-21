//! FIFO channels used to send messages between async tasks.

use std::fmt::{Debug, Display, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use futures::future::FusedFuture;
use log::error;
use thiserror::Error;
use crate::error::*;


#[cfg(feature = "channel_futures_mpsc")]
mod futures_mpsc;

#[cfg(feature = "channel_flume_mpmc")]
mod flume_mpmc;

#[cfg(feature = "channel_async_channel_mpmc")]
mod async_channel_mpmc;

#[cfg(feature = "channel_mult_custom_dump")]
mod custom_dump;

#[cfg(feature = "channel_sync_crossbeam")]
mod crossbeam;

mod oneshot_spsc;

/**
 * ASYNCHRONOUS CHANNEL
 */
/// General purpose channel's sending half.
pub struct ChannelAsyncTx<T> {
    #[cfg(feature = "channel_futures_mpsc")]
    inner: futures_mpsc::ChannelAsyncTx<T>,

    #[cfg(feature = "channel_flume_mpmc")]
    inner: flume_mpmc::ChannelMixedTx<T>,

    #[cfg(feature = "channel_async_channel_mpmc")]
    inner: async_channel_mpmc::ChannelAsyncTx<T>,
}

/// General purpose channel's receiving half.
pub struct ChannelAsyncRx<T> {
    #[cfg(feature = "channel_futures_mpsc")]
    inner: futures_mpsc::ChannelRx<T>,

    #[cfg(feature = "channel_flume_mpmc")]
    inner: flume_mpmc::ChannelMixedRx<T>,

    #[cfg(feature = "channel_async_channel_mpmc")]
    inner: async_channel_mpmc::ChannelAsyncRx<T>,
}

/// Future for a general purpose channel's receiving operation.
pub struct ChannelRxFut<'a, T> {
    #[cfg(feature = "channel_futures_mpsc")]
    inner: futures_mpsc::ChannelRxFut<'a, T>,

    #[cfg(feature = "channel_flume_mpmc")]
    inner: flume_mpmc::ChannelRxFut<'a, T>,

    #[cfg(feature = "channel_async_channel_mpmc")]
    inner: async_channel_mpmc::ChannelRxFut<'a, T>,
}

impl<T> Clone for ChannelAsyncTx<T> {
    #[inline]
    fn clone(&self) -> Self {
        let inner = self.inner.clone();
        Self { inner }
    }
}

impl<T> Clone for ChannelAsyncRx<T> {
    #[inline]
    fn clone(&self) -> Self {
        let inner = self.inner.clone();
        Self { inner }
    }
}

impl<T> ChannelAsyncTx<T> {

    //Can have length because future mpsc doesn't implement it

    //Asynchronously send message through channel
    #[inline]
    pub async fn send(&mut self, message: T) -> Result<()> {
        self.inner.send(message).await
    }
}

impl<T> ChannelAsyncRx<T> {
    //Asynchronously recv message from channel
    #[inline]
    pub fn recv<'a>(&'a mut self) -> ChannelRxFut<'a, T> {
        let inner = self.inner.recv();
        ChannelRxFut { inner }
    }
}

impl<'a, T> Future for ChannelRxFut<'a, T> {
    type Output = Result<T>;

    #[inline]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<T>> {
        Pin::new(&mut self.inner).poll(cx)
    }
}

impl<'a, T> FusedFuture for ChannelRxFut<'a, T> {
    #[inline]
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

/// Creates a new general purpose channel that can queue up to
/// `bound` messages from different async senders.
#[inline]
pub fn new_bounded_async<T>(bound: usize) -> (ChannelAsyncTx<T>, ChannelAsyncRx<T>) {
    let (tx, rx) = {
        #[cfg(feature = "channel_futures_mpsc")]
        { futures_mpsc::new_bounded(bound) }
        #[cfg(feature = "channel_flume_mpmc")]
        { flume_mpmc::new_bounded(bound) }
        #[cfg(feature = "channel_async_channel_mpmc")]
        { async_channel_mpmc::new_bounded(bound) }
    };

    let ttx = ChannelAsyncTx { inner: tx };

    let rrx = ChannelAsyncRx { inner: rx };

    (ttx, rrx)
}

/**
Sync channels
 */
pub struct ChannelSyncRx<T> {
    #[cfg(feature = "channel_sync_crossbeam")]
    inner: crossbeam::ChannelSyncRx<T>,
    #[cfg(feature = "channel_sync_flume")]
    inner: flume_mpmc::ChannelMixedRx<T>,
}

pub struct ChannelSyncTx<T> {
    channel_identifier: Option<String>,
    #[cfg(feature = "channel_sync_crossbeam")]
    inner: crossbeam::ChannelSyncTx<T>,
    #[cfg(feature = "channel_sync_flume")]
    inner: flume_mpmc::ChannelMixedTx<T>,
}

impl<T> ChannelSyncRx<T> {
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    pub fn try_recv(&self) -> std::result::Result<T, TryRecvError> {
        self.inner.try_recv()
    }

    #[inline]
    pub fn recv(&self) -> Result<T> {
        self.inner.recv()
    }

    #[inline]
    pub fn recv_timeout(&self, timeout: Duration) -> std::result::Result<T, TryRecvError> {
        self.inner.recv_timeout(timeout)
    }
}

impl<T> ChannelSyncTx<T> {
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    pub fn send(&self, value: T) -> Result<()> {
        let value = match self.inner.try_send(value) {
            Ok(_) => {
                return Ok(())
            }
            Err(err) => {
                match err {
                    TrySendError::Full(value) => {
                        error!("Failed to insert into channel. Channel is full and could not directly insert, blocking. {:?}", self.channel_identifier);

                        value
                    }
                    TrySendError::Disconnected(value) => {
                        error!("Channel is disconnected");

                        value
                    }
                    TrySendError::Timeout(value) => {
                        value
                    }
                }
            }
        };

        self.inner.send(value)
    }

    #[inline]
    pub fn send_timeout(&self, value: T, timeout: Duration) -> Result<()> {
        self.inner.send_timeout(value, timeout)
    }

    #[inline]
    pub fn try_send(&self, value: T) -> Result<()> {
        self.inner.try_send(value)
    }
}

impl<T> Clone for ChannelSyncTx<T> {
    fn clone(&self) -> Self {
        ChannelSyncTx {
            channel_identifier: self.channel_identifier.clone(),
            inner: self.inner.clone()
        }
    }
}

impl<T> Clone for ChannelSyncRx<T> {
    fn clone(&self) -> Self {
        ChannelSyncRx {
            inner: self.inner.clone()
        }
    }
}

#[inline]
pub fn new_bounded_sync<T>(bound: usize, name: Option<&str>) -> (ChannelSyncTx<T>, ChannelSyncRx<T>) {

    let name = name.map(|string|  String::from(string));

    #[cfg(feature = "channel_sync_crossbeam")]
    {
        let (tx, rx) = crossbeam::new_bounded(bound);

        (ChannelSyncTx { channel_identifier: name, inner: tx }, ChannelSyncRx { inner: rx })
    }

    #[cfg(feature = "channel_sync_flume")]
    {
        let (tx, rx) = flume_mpmc::new_bounded(bound);

        (ChannelSyncTx { channel_identifier: name, inner: tx }, ChannelSyncRx { inner: rx })
    }
}


#[inline]
pub fn new_unbounded_sync<T>(name: Option<&str>) -> (ChannelSyncTx<T>, ChannelSyncRx<T>) {

    let name = name.map(|string| String::from(string));

    #[cfg(feature = "channel_sync_crossbeam")]
    {
        let (tx, rx) = crossbeam::new_unbounded();

        (ChannelSyncTx { channel_identifier: name, inner: tx }, ChannelSyncRx { inner: rx })
    }

    #[cfg(feature = "channel_sync_flume")]
    {
        let (tx, rx) = flume_mpmc::new_unbounded();

        (ChannelSyncTx { channel_identifier: name, inner: tx }, ChannelSyncRx { inner: rx })
    }
}

/**
Async and sync mixed channels (Allows us to connect async and sync environments together)
 */
pub struct ChannelMixedRx<T> {
    #[cfg(feature = "channel_mixed_flume")]
    inner: flume_mpmc::ChannelMixedRx<T>,
}

pub struct ChannelMixedTx<T> {
    #[cfg(feature = "channel_mixed_flume")]
    inner: flume_mpmc::ChannelMixedTx<T>,
}

impl<T> ChannelMixedRx<T> {
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    pub fn recv(&self) -> Result<T> {
        match self.inner.recv_sync() {
            Ok(res) => {
                Ok(res)
            }
            Err(_err) => {
                Err(RecvError::ChannelDc)
            }
        }
    }

    #[inline]
    pub fn recv_timeout(&self, timeout: Duration) -> Result<T> {
        match self.inner.recv_timeout(timeout) {
            Ok(result) => {
                Ok(result)
            }
            Err(_err) => {
                Err(RecvError::ChannelDc)
            }
        }
    }

    #[inline]
    pub async fn recv_async(&mut self) -> Result<T> {
        match self.inner.recv().await {
            Ok(val) => {
                Ok(val)
            }
            Err(_err) => {
                Err(RecvError::ChannelDc)
            }
        }
    }

    #[inline]
    pub fn try_recv(&self) -> Result<T> {
        self.inner.try_recv()
    }
}

impl<T> ChannelMixedTx<T> {
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    pub async fn send_async(&self, value: T) -> Result<()> {
        self.inner.send(value).await
    }

    #[inline]
    pub fn send(&self, value: T) -> Result<()> {
        self.inner.send_sync(value)
    }

    #[inline]
    pub fn send_timeout(&self, value: T, timeout: Duration) -> Result<()> {
        self.inner.send_timeout(value, timeout)
    }
}


impl<T> Clone for ChannelMixedTx<T> {
    fn clone(&self) -> Self {
        ChannelMixedTx {
            inner: self.inner.clone()
        }
    }
}

impl<T> Clone for ChannelMixedRx<T> {
    fn clone(&self) -> Self {
        ChannelMixedRx {
            inner: self.inner.clone()
        }
    }
}

pub fn new_bounded_mixed<T>(bound: usize) -> (ChannelMixedTx<T>, ChannelMixedRx<T>) {
    let (tx, rx) = {
        #[cfg(feature = "channel_mixed_flume")]
        {
            flume_mpmc::new_bounded(bound)
        }
    };

    (ChannelMixedTx { inner: tx }, ChannelMixedRx { inner: rx })
}

/**
Channel with capability of dumping multiple members in a couple of CAS operations
 */

pub struct ChannelMultTx<T> {
    #[cfg(feature = "channel_mult_custom_dump")]
    inner: custom_dump::ChannelTx<T>,
}

pub struct ChannelMultRx<T> {
    #[cfg(feature = "channel_mult_custom_dump")]
    inner: custom_dump::ChannelRxMult<T>,
}

impl<T> ChannelMultTx<T> {
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_dc(&self) -> bool {
        self.inner.is_dc()
    }

    #[inline]
    pub async fn send_async(&self, value: T) -> Result<()> {
        self.inner.send(value).await
    }

    #[inline]
    pub fn send(&self, value: T) -> Result<()> {
        self.inner.send_blk(value)
    }
}

impl<T> ChannelMultRx<T> {

    pub fn is_dc(&self) -> bool {
        self.inner.is_dc()
    }

    pub async fn recv_mult(&mut self) -> Result<Vec<T>> {
        self.inner.recv().await
    }

    pub fn recv_mult_sync(&self, dest: &mut Vec<T>) -> Result<usize> {
        self.inner.recv_sync(dest)
    }

    pub fn try_recv_mult(&self, dest: &mut Vec<T>, rq_bound: usize) -> Result<usize> {
        self.inner.try_recv_mult(dest, rq_bound)
    }
}

impl<T> Clone for ChannelMultRx<T> {
    fn clone(&self) -> Self {
        ChannelMultRx {
            inner: self.inner.clone()
        }
    }
}

impl<T> Clone for ChannelMultTx<T> {
    fn clone(&self) -> Self {
        ChannelMultTx {
            inner: self.inner.clone()
        }
    }
}

#[inline]
pub fn new_bounded_mult<T>(bound: usize) -> (ChannelMultTx<T>, ChannelMultRx<T>) {
    let (tx, rx) = custom_dump::bounded_mult_channel(bound);

    (ChannelMultTx { inner: tx }, ChannelMultRx { inner: rx })
}

/*
  One shot channels
  @{
 */

pub type OneShotTx<T> = oneshot_spsc::OneShotTx<T>;

pub type OneShotRx<T> = oneshot_spsc::OneShotRx<T>;

#[inline]
pub fn new_oneshot_channel<T>() -> (OneShotTx<T>, OneShotRx<T>) {
    oneshot_spsc::new_oneshot()
}

/**
Errors
 **/

#[derive(Error, Debug)]
pub enum RecvMultError {
    #[error("Failed receive, channel is disconnected")]
    ChannelDc,
    #[error("The input vec to place received messages is malformed")]
    MalformedInputVec,
    #[error("Unsupported operation")]
    Unsupported,
}

#[derive(Error, Debug)]
pub enum TryRecvError {
    #[error("Channel has disconnected")]
    ChannelDc,
    #[error("Channel is empty")]
    ChannelEmpty,
    #[error("Receive operation timed out")]
    Timeout,
}

#[derive(Error, Debug)]
pub enum RecvError {
    #[error("Channel has disconnected")]
    ChannelDc,
}

#[derive(Error, Debug)]
pub enum TrySendError<T> {
    #[error("Channel has disconnected")]
    Disconnected(T),
    #[error("Send operation has timed out")]
    Timeout(T),
    #[error("Channel is full")]
    Full(T)
}

#[derive(Error, Debug)]
pub enum SendError<T> {
    FailedToSend(#[from] T)
}