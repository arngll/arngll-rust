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

use quick_dsp::bell202::*;
use quick_dsp::filter::*;
use std::path::Path;

fn run_benchmark<P: AsRef<Path>>(path: P) -> u32 {
    use std::fs::File;
    const X25: crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_IBM_SDLC);

    let mut inp_file = File::open(path.as_ref()).unwrap();
    let (header, data) = wav::read(&mut inp_file).unwrap();

    let mut decoder = bell_202_decoder(BELL202_OPTIMAL_SAMPLE_RATE);
    let mut downsampler =
        Downsampler::<f32>::new(header.sampling_rate, BELL202_OPTIMAL_SAMPLE_RATE);

    let mut framecount = 0u32;
    let mut badframecount = 0u32;
    let mut drop = false;

    match data {
        wav::BitDepth::Sixteen(vec) => {
            for sample in vec {
                // Remove the stereo
                if header.channel_count == 2 && drop {
                    drop = false;
                    continue;
                } else {
                    drop = true;
                }

                // Convert to floating point
                let sample = sample as f32 / (std::i16::MAX as f32 / 4.0 * 3.0);

                // Downsample
                let sample = if let Some(sample) = downsampler.filter(sample) {
                    sample
                } else {
                    continue;
                };

                // Decode
                let out = decoder.filter(sample);

                if let Some(frame) = out {
                    if frame.len() < 7 {
                        continue;
                    }

                    if X25.checksum(&frame) != 0x0f47 {
                        if Ax25Debug(&frame).is_ax25() {
                            badframecount += 1;
                        }
                    } else {
                        framecount += 1;
                    }
                }
            }
        }
        _ => panic!("bad data"),
    }
    println!(
        "{}: Success:{} Bad-CRC:{}, Total:{}",
        path.as_ref().to_str().unwrap(),
        framecount,
        badframecount,
        framecount + badframecount
    );
    framecount
}

fn get_path(file: &str) -> String {
    format!("../contrib/TNCTestCD/{}", file)
}

#[test]
fn benchmark_testcd01() {
    let filename = "testcd01.wav";
    let path_str = get_path(filename);
    let path = Path::new(&path_str);
    if !path.exists() {
        eprintln!("File {:?} doesn't exist, skipping test.", path);
        return;
    }
    assert!(run_benchmark(path) >= 950);
}

#[test]
fn benchmark_testcd02() {
    let filename = "testcd02.wav";
    let path_str = get_path(filename);
    let path = Path::new(&path_str);
    if !path.exists() {
        eprintln!("File {:?} doesn't exist, skipping test.", path);
        return;
    }
    assert!(run_benchmark(path) >= 940);
}

#[test]
fn benchmark_testcd03() {
    let filename = "testcd03.wav";
    let path_str = get_path(filename);
    let path = Path::new(&path_str);
    if !path.exists() {
        eprintln!("File {:?} doesn't exist, skipping test.", path);
        return;
    }
    assert_eq!(run_benchmark(path), 100);
}

#[test]
fn benchmark_testcd04() {
    let filename = "testcd04.wav";
    let path_str = get_path(filename);
    let path = Path::new(&path_str);
    if !path.exists() {
        eprintln!("File {:?} doesn't exist, skipping test.", path);
        return;
    }
    assert!(run_benchmark(path) >= 87);
}
