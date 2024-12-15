//! FIFO channels used to send messages between async tasks.

use std::fmt::{Debug, Formatter};

use thiserror::Error;

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

#[cfg(any(feature = "channel_sync_kanal", feature = "channel_mixed_kanal", feature = "channel_async_kanal"))]
mod kanal;

mod oneshot_spsc;

pub mod sync;
pub mod r#async;
pub mod mixed;
pub mod oneshot;
pub mod mult;

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

#[derive(Error)]
pub enum TrySendReturnError<T> {
    #[error("Channel has disconnected")]
    Disconnected(T),
    #[error("Send operation has timed out")]
    Timeout(T),
    #[error("Channel is full")]
    Full(T),
}

#[derive(Error, Debug)]
pub enum SendError {
    #[error("Failed to send message")]
    FailedToSend,
}

#[derive(Error, Debug)]
pub enum TrySendError {
    #[error("Channel has disconnected")]
    Disconnected,
    #[error("Send operation has timed out")]
    Timeout,
    #[error("Channel is full")]
    Full,
}

#[derive(Error)]
pub enum SendReturnError<T> {
    #[error("Failed to send message, channel disconnected")]
    FailedToSend(T),
}

unsafe impl<T> Send for SendReturnError<T> {}

unsafe impl<T> Sync for SendReturnError<T> {}

unsafe impl<T> Send for TrySendReturnError<T> {}

unsafe impl<T> Sync for TrySendReturnError<T> {}

impl<T> Debug for SendReturnError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to send message")
    }
}

impl<T> Debug for TrySendReturnError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to send message")
    }
}

impl<T> From<SendReturnError<T>> for SendError {
    fn from(value: SendReturnError<T>) -> Self {
        match value {
            SendReturnError::FailedToSend(_) => SendError::FailedToSend,
        }
    }
}

impl<T> From<TrySendReturnError<T>> for TrySendError {
    fn from(value: TrySendReturnError<T>) -> Self {
        match value {
            TrySendReturnError::Disconnected(_) => TrySendError::Disconnected,
            TrySendReturnError::Timeout(_) => TrySendError::Timeout,
            TrySendReturnError::Full(_) => TrySendError::Full,
        }
    }
}
