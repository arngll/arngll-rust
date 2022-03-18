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

mod receiver;
mod sender;

use crate::filter::*;
pub use receiver::*;
pub use sender::*;
use std::fmt::{Debug, Formatter};

pub const BELL202_RATE: u32 = 1200;
pub const BELL202_MARK: u32 = 1200;
pub const BELL202_SPACE: u32 = 2200;
pub const BELL202_OPTIMAL_SAMPLE_RATE: u32 = 7500;

/// Bell 202 decoder.
///
/// Feed in samples into the returned filter and it will
/// occasionally spit out a frame. Does not check CRC.
///
/// Theoretical ideal sample rate is 7500. Maximum usable sample rate
/// is around 10000. If your sample rate is too high, you will need
/// to downsample first.
pub fn bell_202_decoder(sample_rate: u32) -> impl OneToOne<f32, Output = Option<Vec<u8>>> {
    #[cfg(not(test))]
    assert!(
        sample_rate <= 14000,
        "max sample rate:14000, given: {}",
        sample_rate
    );

    let space = (BELL202_SPACE as f32) / (sample_rate as f32);
    let mark = (BELL202_MARK as f32) / (sample_rate as f32);

    Discriminator::<f32, ()>::digital_default()
        .chain(FskDemod::new(space, mark))
        .chain(BitSampler::new(sample_rate, BELL202_RATE))
        .chain(NrziDecode::new().optional())
        .chain(HdlcDecode::default())
        // .inspect(|x| {
        //     if let Some(x) = x {
        //         println!("{:?}", x);
        //     }
        // })
        .chain(FrameCollector::default())
}

/// Bell 202 encoder.
///
/// Encodes a single frame of octets. Does not add CRC.
/// Input is an iterator of octets. Output is an iterator
/// samples at the given sample rate, with a preamble.
pub fn bell_202_encode<'a, Out, InIterator: Iterator<Item = u8> + 'a>(
    iter: InIterator,
    sample_rate: u32,
    amplitude: f32,
) -> impl Iterator<Item = <Decimator<f32, Out> as OneToOne<f32>>::Output> + 'a
where
    Decimator<f32, Out>: Default + OneToOne<f32>,
    Out: 'a,
{
    let samples_per_bit = (sample_rate as f32) / (BELL202_RATE as f32);
    let mark_freq = (BELL202_MARK as f32) / (sample_rate as f32);
    let space_freq = (BELL202_SPACE as f32) / (sample_rate as f32);

    iter.bits_lsb()
        .hdlc_encode()
        .nrzi_encode()
        .resample_nn(samples_per_bit)
        .map(move |x| match x {
            true => mark_freq,
            false => space_freq,
        })
        .apply_one_to_one(FmMod::new(amplitude))
        .apply_one_to_one(Decimator::<f32, Out>::default())
}

pub struct Ax25Debug<'a>(pub &'a [u8]);

impl<'a> Ax25Debug<'a> {
    /// Returns the length of the address field
    pub fn addr_len(&self) -> usize {
        std::cmp::min(
            self.0.len(),
            self.0.iter().take_while(|&x| x & 1 != 1).count() + 1,
        )
    }

    pub fn addr_bytes(&self) -> &'a [u8] {
        &self.0[..self.addr_len()]
    }

    pub fn addr_escaped_ascii(&self) -> impl Iterator<Item = u8> + 'a {
        self.addr_bytes()
            .iter()
            .map(|x| x >> 1)
            .map(|x| if x.is_ascii() && x > 31 { x } else { b'.' })
            .flat_map(std::ascii::escape_default)
    }

    pub fn payload_bytes(&self) -> &'a [u8] {
        &self.0[self.addr_len()..]
    }

    pub fn payload_escaped_ascii(&self) -> impl Iterator<Item = u8> + 'a {
        self.payload_bytes()
            .iter()
            .map(|&x| if x.is_ascii() && x > 31 { x } else { b'.' })
            .flat_map(std::ascii::escape_default)
    }

    /// Returns true if this looks like a AX25 packet
    pub fn is_ax25(&self) -> bool {
        // AX.25 packets always have an address field that is a multiple of 7 bytes.
        (self.addr_len() % 7) == 0
    }
}

impl<'a> Debug for Ax25Debug<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use core::str::from_utf8;

        let addr_escaped = self.addr_escaped_ascii().collect::<Vec<_>>();
        let addr_str = from_utf8(&addr_escaped).unwrap();

        let payload_escaped = self.payload_escaped_ascii().collect::<Vec<_>>();
        let payload_str = from_utf8(&payload_escaped).unwrap();

        write!(f, "[{}]{}", addr_str, payload_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filter::Downsampler;

    #[test]
    fn test_ax25_debug_decode() {
        let vec: Vec<u8> = hex::decode("82a0aa646a9ce0ae8270989a8c60ae92888a62406303f03e3230323333377a687474703a2f2f7761386c6d662e636f6d0df782").unwrap();
        assert!(Ax25Debug(&vec).is_ax25());
        assert_eq!(
            format!("{:?}", Ax25Debug(&vec)),
            "[APU25NpWA8LMF0WIDE1 1]..>202337zhttp://wa8lmf.com..."
        );
    }

    #[test]
    fn test_bell_202_encode_decode() {
        for sample_rate in (6000u32..14900).step_by(100) {
            test_bell_202_encode_decode_at(sample_rate);
        }
    }

    fn test_bell_202_encode_decode_at(sample_rate: u32) {
        let vec: Vec<u8> = hex::decode("82a0aa646a9ce0ae8270989a8c60ae92888a62406303f03e3230323333377a687474703a2f2f7761386c6d662e636f6d0df782").unwrap();

        let iter = bell_202_encode::<f32, _>(vec.into_iter(), sample_rate, 0.75);

        let mut decoder = bell_202_decoder(sample_rate);

        for x in iter {
            if let Some(_x) = decoder.filter(x) {
                //println!("decoded: {:?}", hex::encode(_x));
                return;
            }
        }

        panic!("Unable to decode at {}", sample_rate);
    }

    #[test]
    fn test_bell_202_encode_decode_resample() {
        let vec: Vec<u8> = hex::decode("82a0aa646a9ce0ae8270989a8c60ae92888a62406303f03e3230323333377a687474703a2f2f7761386c6d662e636f6d0df782").unwrap();
        let in_sample_rate = 44100;
        let iter = bell_202_encode::<f32, _>(vec.clone().into_iter(), in_sample_rate, 0.75);

        let mut decoder = bell_202_decoder(BELL202_OPTIMAL_SAMPLE_RATE);
        let mut resampler = Downsampler::new(in_sample_rate, BELL202_OPTIMAL_SAMPLE_RATE);

        for x in iter {
            if let Some(x) = resampler.filter(x) {
                if let Some(x) = decoder.filter(x) {
                    println!("decoded: {:?}", hex::encode(&x));
                    assert_eq!(vec, x);
                    return;
                }
            }
        }
        panic!("Unable to decode");
    }
}
