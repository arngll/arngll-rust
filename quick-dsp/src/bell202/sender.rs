// Copyright (c) 2022, The ARNGLL-Rust Authors.
//
// Permission is hereby granted, free of charge, to any person obtaining
// a copy of this software and associated documentation files (the
// "Software"), to deal in the Software without restriction, including
// without limitation the rights to use, copy, modify, merge, publish,
// distribute, sublicense, and/or sell copies of the Software, and to
// permit persons to whom the Software is furnished to do so, subject to
// the following conditions:
//
// The above copyright notice and this permission notice shall be
// included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
// IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
// TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
// SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

use super::bell_202_encode;
use anyhow::{format_err, Context as _, Error, Result};
use async_timer::oneshot::{Oneshot, Timer};
use cpal::traits::*;
use cpal::*;
use futures::channel::mpsc;
use futures::task::noop_waker;
use futures::SinkExt;
use log::debug;
use rand::Rng;
use std::cell::Cell;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll, Waker};

pub struct Bell202Sender {
    output_audio_stream: cpal::Stream,
    sendframe_sender: mpsc::Sender<Vec<u8>>,
    is_channel_clear: AtomicBool,
    channel_clear_waker: Cell<Waker>,
    cca_backoff_timer: Option<Timer>,
}

impl Bell202Sender {
    pub fn new(device: &cpal::Device) -> Result<Bell202Sender, Error> {
        let mut supported_stream_configs = device
            .supported_output_configs()
            .context("error while querying configs")?;

        let supported_config_range = supported_stream_configs
            .next()
            .expect("no supported config?!");

        let mut supported_config: StreamConfig =
            supported_config_range.with_max_sample_rate().into();

        // We only care about a single channel.
        supported_config.channels = 1;

        match Self::new_with_config(device, &supported_config) {
            Ok(ret) => Ok(ret),
            Err(err) => {
                // Try a different sample rate.
                supported_config.sample_rate = SampleRate(11025);
                if let Ok(ret) = Self::new_with_config(device, &supported_config) {
                    Ok(ret)
                } else {
                    // Last try.
                    supported_config.sample_rate = SampleRate(8000);
                    if let Ok(ret) = Self::new_with_config(device, &supported_config) {
                        Ok(ret)
                    } else {
                        Err(err)
                    }
                }
            }
        }
    }

    pub fn new_with_config(
        device: &cpal::Device,
        supported_config: &StreamConfig,
    ) -> Result<Bell202Sender, Error> {
        let sample_rate = supported_config.sample_rate.0;

        // We are just using this to make sure we get the type right
        // for the output func. It should play as silence.
        let mut encoder = bell_202_encode::<f32, _>(vec![].into_iter(), sample_rate, 0.0);

        let (sendframe_sender, mut sendframe_receiver) = mpsc::channel::<Vec<u8>>(1);

        debug!("Sender stream config: {:?}", supported_config);

        let output_audio_stream = device.build_output_stream(
            &supported_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                for sample in data.iter_mut() {
                    if let Some(value) = encoder.next() {
                        *sample = value;
                    } else if let Ok(Some(vec)) = sendframe_receiver.try_next() {
                        // Set up the next frame.
                        encoder = bell_202_encode(vec.into_iter(), sample_rate, 0.75);
                        *sample = encoder.next().unwrap();
                    } else {
                        *sample = 0.0;
                    }
                }
            },
            move |err| {
                // react to errors here.
                panic!("err: {:?}", err);
            },
        )?;

        output_audio_stream.play()?;

        Ok(Bell202Sender {
            output_audio_stream,
            sendframe_sender,
            is_channel_clear: AtomicBool::new(true),
            channel_clear_waker: Cell::new(noop_waker()),
            cca_backoff_timer: None,
        })
    }

    /// Sets channel clear indicator. This should be set to false
    /// when there is a signal on the channel, true if no signal is detected.
    pub fn set_channel_clear(&self, is_channel_clear: bool) {
        debug!("CCA: is_channel_clear={:?}", is_channel_clear);
        self.is_channel_clear
            .store(is_channel_clear, Ordering::Relaxed);
        self.channel_clear_waker.replace(noop_waker()).wake()
    }

    pub fn pause(&mut self) -> Result<(), Error> {
        self.output_audio_stream.pause()?;
        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), Error> {
        self.output_audio_stream.play()?;
        Ok(())
    }
}

impl futures::sink::Sink<Vec<u8>> for Bell202Sender {
    type Error = anyhow::Error;

    fn poll_ready(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), Self::Error>> {
        if let Some(timer) = self.cca_backoff_timer.as_mut() {
            if Pin::new(timer).poll(cx).is_pending() {
                return Poll::Pending;
            }
        }

        self.cca_backoff_timer = None;

        if self.is_channel_clear.load(Ordering::Relaxed) {
            self.sendframe_sender
                .poll_ready_unpin(cx)
                .map_err(anyhow::Error::from)
        } else {
            self.cca_backoff_timer = Some(Timer::new(std::time::Duration::from_millis(
                rand::thread_rng().gen_range(5..50),
            )));
            self.channel_clear_waker.replace(cx.waker().clone());
            Poll::Pending
        }
    }

    fn start_send(mut self: Pin<&mut Self>, item: Vec<u8>) -> std::result::Result<(), Self::Error> {
        if self.is_channel_clear.load(Ordering::Relaxed) {
            self.sendframe_sender
                .start_send_unpin(item)
                .map_err(anyhow::Error::from)
        } else {
            Err(format_err!("Channel not clear"))
        }
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), Self::Error>> {
        self.sendframe_sender
            .poll_flush_unpin(cx)
            .map_err(anyhow::Error::from)
    }

    fn poll_close(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), Self::Error>> {
        self.sendframe_sender
            .poll_close_unpin(cx)
            .map_err(anyhow::Error::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;

    #[test]
    #[ignore]
    fn test_bell_202_phy_sender() {
        let device = cpal::default_host().default_output_device().unwrap();
        let mut sender = Bell202Sender::new(&device).unwrap();

        let frame = hex::decode("82a0aa646a9ce0ae8270989a8c60ae92888a62406303f03e3230323333377a687474703a2f2f7761386c6d662e636f6d0df782").unwrap();

        block_on(sender.send(frame)).unwrap();

        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
