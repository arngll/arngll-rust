use super::bell_202_encode;
use anyhow::{Context as _, Error, Result};
use cpal::traits::*;
use cpal::*;
use futures::channel::mpsc;
use futures::SinkExt;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct Bell202Sender {
    output_audio_stream: cpal::Stream,
    sendframe_sender: mpsc::Sender<Vec<u8>>,
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

        let sample_rate = supported_config.sample_rate.0;

        let mut encoder = bell_202_encode(vec![].into_iter(), sample_rate, 0.75);

        let (sendframe_sender, mut sendframe_receiver) = mpsc::channel::<Vec<u8>>(3);

        let output_audio_stream = device.build_output_stream(
            &supported_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                for sample in data.iter_mut() {
                    if let Some(value) = encoder.next() {
                        *sample = value;
                    } else if let Ok(Some(vec)) = sendframe_receiver.try_next() {
                        // Set up the next frame.
                        encoder = bell_202_encode::<f32, _>(vec.into_iter(), sample_rate, 0.75);
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
        })
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
        self.sendframe_sender
            .poll_ready_unpin(cx)
            .map_err(anyhow::Error::from)
    }

    fn start_send(mut self: Pin<&mut Self>, item: Vec<u8>) -> std::result::Result<(), Self::Error> {
        self.sendframe_sender
            .start_send_unpin(item)
            .map_err(anyhow::Error::from)
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
