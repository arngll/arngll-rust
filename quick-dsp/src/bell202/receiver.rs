use super::{bell_202_decoder, BELL202_OPTIMAL_SAMPLE_RATE};
use crate::filter::{Downsampler, OneToOne};
use anyhow::{Context as _, Error, Result};
use cpal::traits::*;
use cpal::*;
use futures::channel::mpsc;
use futures::StreamExt;
use log::{debug, trace};
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct Bell202Receiver {
    input_audio_stream: cpal::Stream,
    recvframe_receiver: mpsc::Receiver<Vec<u8>>,
}

impl Bell202Receiver {
    pub fn new(device: &cpal::Device) -> Result<Bell202Receiver, Error> {
        let mut supported_stream_configs = device
            .supported_input_configs()
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
                    Err(err)
                }
            }
        }
    }

    pub fn new_with_config(
        device: &cpal::Device,
        supported_config: &StreamConfig,
    ) -> Result<Bell202Receiver, Error> {
        debug!("supported_config: {:?}", supported_config);
        let mut downsampler =
            Downsampler::<f32>::new(supported_config.sample_rate.0, BELL202_OPTIMAL_SAMPLE_RATE);

        let mut decoder = bell_202_decoder(BELL202_OPTIMAL_SAMPLE_RATE);
        let (mut recvframe_sender, recvframe_receiver) = mpsc::channel(10);
        let input_audio_stream = device.build_input_stream(
            supported_config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let iter = data.iter().filter_map(|x| downsampler.filter(*x));
                for sample in iter {
                    if let Some(frame) = decoder.filter(sample) {
                        const X25: crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_IBM_SDLC);
                        if X25.checksum(&frame) == 0x0f47 {
                            if recvframe_sender.try_send(frame).is_err() {
                                trace!("Dropped packet");
                            }
                        } else {
                            trace!("Bad CRC");
                        }
                    }
                }
            },
            move |err| {
                // react to errors here.
                panic!("err: {:?}", err);
            },
        )?;

        input_audio_stream.play()?;

        Ok(Bell202Receiver {
            input_audio_stream,
            recvframe_receiver,
        })
    }

    pub fn pause(&mut self) -> Result<(), Error> {
        self.input_audio_stream.pause()?;
        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), Error> {
        self.input_audio_stream.play()?;
        Ok(())
    }
}

impl Deref for Bell202Receiver {
    type Target = mpsc::Receiver<Vec<u8>>;

    fn deref(&self) -> &Self::Target {
        &self.recvframe_receiver
    }
}

impl DerefMut for Bell202Receiver {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.recvframe_receiver
    }
}

impl futures::stream::Stream for Bell202Receiver {
    type Item = Vec<u8>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.recvframe_receiver.poll_next_unpin(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on_stream;
    use log::info;

    #[test]
    #[ignore]
    fn test_bell_202_phy_receiver() {
        stderrlog::new().verbosity(10).init().unwrap();
        let device = cpal::default_host().default_input_device().unwrap();
        info!("device: {:?}", device.name());
        let receiver = Bell202Receiver::new(&device).unwrap();

        for frame in block_on_stream(receiver) {
            info!("Received: {:?}", hex::encode(frame));
        }
    }
}
