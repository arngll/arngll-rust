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

use anyhow::format_err;
use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait};
use futures::executor::{block_on, block_on_stream};
use futures::prelude::*;
use hamaddr::HamAddr;
use log::info;
use quick_dsp::bell202::{Ax25Debug, Bell202Receiver, Bell202Sender};
use quick_dsp::filter::IteratorExt as _;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Opt {
    /// Silence all output
    #[clap(short, long)]
    quiet: bool,

    /// Verbose mode (-v, -vv, -vvv, etc)
    #[clap(short, long, parse(from_occurrences))]
    verbose: usize,

    /// List audio devices
    #[clap(long)]
    list_devices: bool,

    #[clap(short, long)]
    callsign: Option<HamAddr>,

    #[clap(long)]
    input_audio_device: Option<String>,

    #[clap(long)]
    output_audio_device: Option<String>,
}

fn find_device<I: IntoIterator<Item = cpal::Device>>(
    devices: I,
    name: &str,
) -> Option<cpal::Device> {
    let lc_name = name.to_lowercase();
    let mut devs = devices
        .into_iter()
        .filter_map(|d| Some((d.name().ok()?.to_lowercase(), d)))
        .collect::<Vec<_>>();

    // Try exact
    if let Some((i, _)) = devs.iter().enumerate().find(|(_, (n, _))| n == &lc_name) {
        return Some(devs.remove(i).1);
    }

    // Try prefix
    if let Some((i, _)) = devs
        .iter()
        .enumerate()
        .find(|(_, (n, _))| n.starts_with(&lc_name))
    {
        return Some(devs.remove(i).1);
    }

    // Any substring match
    if let Some((i, _)) = devs
        .iter()
        .enumerate()
        .find(|(_, (n, _))| n.contains(&lc_name))
    {
        return Some(devs.remove(i).1);
    }

    None
}

impl Opt {
    fn get_output_device(&self) -> Result<cpal::Device, anyhow::Error> {
        let host = cpal::default_host();

        if let Some(name) = self.output_audio_device.as_ref() {
            return find_device(host.output_devices()?, name.as_str())
                .ok_or_else(|| format_err!("Cannot find output device matching {:?}", name));
        }

        host.default_output_device()
            .ok_or_else(|| format_err!("no default output device"))
    }

    #[allow(dead_code)]
    fn get_input_device(&self) -> Result<cpal::Device, anyhow::Error> {
        let host = cpal::default_host();

        if let Some(name) = self.input_audio_device.as_ref() {
            return find_device(host.input_devices()?, name.as_str())
                .ok_or_else(|| format_err!("Cannot find input device matching {:?}", name));
        }

        host.default_input_device()
            .ok_or_else(|| format_err!("no default input device"))
    }

    #[allow(dead_code)]
    fn get_packet_stream(&self) -> Result<Bell202Receiver, anyhow::Error> {
        let device = self.get_input_device()?;
        info!("Using input device {:?}", device.name());
        let receiver = Bell202Receiver::new(&device)?;

        Ok(receiver)
    }

    fn get_packet_sink(&self) -> Result<Bell202Sender, anyhow::Error> {
        let device = self.get_output_device()?;
        info!("Using output device {:?}", device.name());
        let sender = Bell202Sender::new(&device)?;

        Ok(sender)
    }
}

fn main() {
    let opt = Opt::parse();

    if opt.list_devices {
        let host = cpal::default_host();
        println!("Input Devices:");
        for device in host.input_devices().expect("Unable to list input devices") {
            if let Some(name) = device.name().ok() {
                println!("\t{}", name);
            }
        }
        println!("Output Devices:");
        for device in host
            .output_devices()
            .expect("Unable to list output devices")
        {
            if let Some(name) = device.name().ok() {
                println!("\t{}", name);
            }
        }
        return;
    }

    stderrlog::new()
        .quiet(opt.quiet)
        .verbosity(opt.verbose)
        .init()
        .unwrap();

    println!("Callsign: {}", opt.callsign.expect("Missing callsign"));
    println!("opt = {:?}", opt);

    const X25: crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_IBM_SDLC);

    let frame_string = format!(
        "\x01\x00de {}: TEST: This is a test frame of ASCII text.",
        opt.callsign.unwrap()
    );
    let frame = frame_string.bytes().append_crc(&X25);

    let mut packet_sink = opt.get_packet_sink().unwrap();

    println!("Sending test frame...");

    // Play the test packet.
    block_on(packet_sink.send(frame.collect())).unwrap();

    println!("Listening for packets...");

    let packet_stream = opt.get_packet_stream().unwrap();

    for frame in block_on_stream(packet_stream) {
        let debug = Ax25Debug(&frame);
        if debug.is_ax25() {
            info!("Received AX25: {:?}", debug);
        } else {
            info!("Received: {:?}", hex::encode(frame));
        }
    }
}
