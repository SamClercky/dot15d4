#![no_std]
//! Simple oneshot Channel implementation based on the one described in the book
//! of Mara Bos, but adapted to work as a signaling mechanism. Sending will
//! remain non-blocking and just overwrite the previous message.
use core::cell::RefCell;
use core::cell::UnsafeCell;
use core::future::poll_fn;
use core::future::Future;
use core::mem::MaybeUninit;
use core::pin::Pin;
use core::task::Context;
use core::task::Poll;
use core::task::Waker;

struct ChannelState {
    is_ready: bool, // We always stay in the same task/thread -> no atomic needed here
    waker: Option<Waker>,
}

pub struct Channel<T> {
    message: UnsafeCell<MaybeUninit<T>>,
    state: UnsafeCell<ChannelState>,
}

impl<T> Channel<T> {
    pub fn new() -> Self {
        Self {
            message: UnsafeCell::new(MaybeUninit::uninit()),
            state: UnsafeCell::new(ChannelState {
                is_ready: false,
                waker: None,
            }),
        }
    }

    pub fn split(&mut self) -> (Sender<'_, T>, Receiver<'_, T>) {
        *self = Self::new(); // Drop previous channel to reset state. We have exclusive access here
        (Sender { channel: self }, Receiver { channel: self })
    }
}

pub struct Sender<'a, T> {
    channel: &'a Channel<T>,
}

impl<T> Sender<'_, T> {
    /// Sends a message across the channel. Sending multiple messages before the
    /// Receiver can read them, results in overwriting the previous messages.
    /// Only the last one will be actually sent. This method returns whether or
    /// not the previous message was overwritten
    pub fn send(&mut self, message: T) -> bool {
        // If the channel is ready, make the message drop
        // Safety: The state is only accessed inside a function body and never across an await point. No concurrent access here (same task)
        let mut state = unsafe { &mut *self.channel.state.get() };
        let did_replace = if state.is_ready {
            unsafe {
                // Drop previous message
                // Safety: This is ok, as we are the only ones with access to the
                // Sender and there is no concurrent access from the reader
                let maybe_uninit = &mut *self.channel.message.get();
                core::ptr::drop_in_place(maybe_uninit.as_mut_ptr());

                // Store the new message
                maybe_uninit.as_mut_ptr().write(message);
                // The channel is already set to be ready -> keep it this way
            }

            // Wake the Receiver task
            if let Some(waker) = state.waker.take() {
                waker.wake()
            }

            // Signal that the channel has replaced something
            true
        } else {
            // The channel is not yet ready -> store the message and make it ready
            // Safety: We are the only one with access to the Sender and no concurrent access with the Receiver possible
            unsafe {
                let maybe_uninit = &mut *self.channel.message.get();
                maybe_uninit.as_mut_ptr().write(message);
            }

            // Signal that the channel was empty before
            false
        };
        // Wake the Receiver task
        state.is_ready = true;
        if let Some(waker) = state.waker.take() {
            waker.wake()
        }

        // Did we replace the inner message or not
        did_replace
    }
}

pub struct Receiver<'a, T> {
    channel: &'a Channel<T>,
}

impl<T> Receiver<'_, T> {
    pub async fn receive(&mut self) -> T {
        poll_fn(|cx| {
            // Safety: We only access the state in the bounds of this call and never across an await point
            let state = unsafe { &mut *self.channel.state.get() };

            if !state.is_ready {
                // Not yet ready, store/replace the context
                match &mut state.waker {
                    Some(waker) => waker.clone_from(cx.waker()),
                    waker @ None => *waker = Some(cx.waker().clone()),
                }

                Poll::Pending
            } else {
                // Safety: We have a message, and exclusive access to the channel as there is no concurrent access possible
                let message = unsafe {
                    let maybe_uninit = &mut *self.channel.message.get();
                    maybe_uninit.assume_init_read()
                };
                // Reset the state, such that we can send again
                state.is_ready = false;

                Poll::Ready(message)
            }
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use pollster::FutureExt as _;

    use crate::sync::{join::join, select::select, yield_now::yield_now};

    use super::Channel;

    #[test]
    pub fn test_channel_no_concurrency() {
        async {
            let mut channel = Channel::new();
            let (mut send, mut recv) = channel.split();
            send.send(1);
            assert_eq!(recv.receive().await, 1);
        }
        .block_on();
    }

    #[test]
    pub fn test_channel_join_concurrency() {
        async {
            let mut channel = Channel::new();
            let (mut send, mut recv) = channel.split();

            join(
                async {
                    for i in 0..10 {
                        send.send(i);
                        yield_now().await;
                    }
                },
                async {
                    for i in 0..10 {
                        assert_eq!(recv.receive().await, i);
                    }
                },
            )
            .await;
        }
        .block_on();
    }
}