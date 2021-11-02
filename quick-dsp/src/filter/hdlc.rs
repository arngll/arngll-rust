use super::*;
use crc::*;
use std::mem::swap;

pub const X25: Crc<u16> = Crc::<u16>::new(&CRC_16_IBM_SDLC);

// bit-stuffing:
// * Applied to frames.
// * frames are prepended with some number of start-of-frame marker patterns: `01111110`
// * Least-significant-bit gets serialized first.
// * continuous runs of five 1 bits always have a single 0 bit appended.
//
// De-bit-stuffing:
// * We look for `01111110`, that marks start-of-frame.
// * Frames start with several frame start markers.
// * We are always looking for frame start markers
// * least-significant-bit gets deserialized first.
//
// * after frame start marker we look for any 5-bit continuous run of 1 bits.
// * After finding a 5-bit continuous run, we drop the next bit and keep decoding.

pub enum HdlcEncoderIter<I: Iterator<Item = bool>> {
    Prelude { inner: I, index: u32 },
    Body { inner: I, ones: u32 },
    Finishing { index: u32 },
    End,
}

impl<I: Iterator<Item = bool>> HdlcEncoderIter<I> {
    pub fn new(iter: I) -> Self {
        HdlcEncoderIter::Prelude {
            inner: iter,
            index: 0,
        }
    }
}

impl<I: Iterator<Item = bool>> Iterator for HdlcEncoderIter<I> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        let mut this = Self::End;
        // TODO: Rewrite to not need this swap.
        std::mem::swap(&mut this, self);
        match this {
            Self::Prelude { inner, mut index } => {
                let ret = !matches!(index & 7, 0 | 7);

                index += 1;
                *self = if index >= 8 * 15 {
                    Self::Body { inner, ones: 0 }
                } else {
                    Self::Prelude { inner, index }
                };
                Some(ret)
            }
            Self::Body {
                mut inner,
                mut ones,
            } => {
                if ones == 5 {
                    ones = 0;
                    *self = Self::Body { inner, ones };
                    Some(false)
                } else if let Some(x) = inner.next() {
                    if x {
                        ones += 1;
                    } else {
                        ones = 0;
                    }

                    *self = Self::Body { inner, ones };
                    Some(x)
                } else {
                    *self = Self::Finishing { index: 1 };
                    Some(false)
                }
            }
            Self::Finishing { mut index } => {
                let ret = !matches!(index & 7, 0 | 7);
                index += 1;
                *self = if index >= 16 {
                    Self::End
                } else {
                    Self::Finishing { index }
                };
                Some(ret)
            }
            Self::End => None,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FrameSignal {
    Octet(u8),
    FrameMarker,
    DecodeError,
}

/// HDLC Bitstream Decoder.
///
/// De-stuffs bits into bytes, separates frames.
/// Output is Option<FrameSignal>
#[derive(Clone, Default, Debug)]
pub struct HdlcDecode {
    accum: u8,
    bit: u8,
    ones: u8,
    skip_next_zero: bool,
    reset_next: bool,
    is_running: bool,
    empty_bits: u8,
}

impl Delay for HdlcDecode {
    fn delay(&self) -> usize {
        8
    }
}

impl Reset for HdlcDecode {
    fn reset(&mut self) {
        self.skip_next_zero = false;
        self.reset_next = false;
        self.bit = 0;
        self.accum = 0;
        self.ones = 0;
        self.is_running = false;
        self.empty_bits = 0;
    }
}

impl OneToOne<Option<bool>> for HdlcDecode {
    type Output = Option<FrameSignal>;

    fn filter(&mut self, sample: Option<bool>) -> Self::Output {
        if let Some(sample) = sample {
            self.empty_bits = 0;
            self.filter(sample)
        } else {
            self.empty_bits += 1;
            if self.empty_bits > 20 {
                self.reset();
            }
            None
        }
    }
}

impl OneToOne<bool> for HdlcDecode {
    type Output = Option<FrameSignal>;

    fn filter(&mut self, sample: bool) -> Self::Output {
        if self.reset_next {
            // The last check simply makes sure we are aligned on a byte boundary.
            return if !sample && (!self.is_running || self.bit == 6 || self.bit == 5) {
                self.reset();
                self.is_running = true;
                Some(FrameSignal::FrameMarker)
            } else if self.is_running {
                self.reset();
                self.is_running = false;
                Some(FrameSignal::DecodeError)
            } else {
                self.reset();
                None
            };
        }

        if self.skip_next_zero {
            self.skip_next_zero = false;
            self.reset_next = sample;
            return None;
        }

        // Decode least-significant bit first
        self.accum = (self.accum >> 1) | ((sample as u8) << 7);

        if sample {
            self.ones += 1;
        } else {
            // Reset ones counter
            self.ones = 0;
        }

        if self.ones == 5 {
            self.skip_next_zero = true;
            self.ones = 0;
        }

        self.bit += 1;
        if self.bit >= 8 {
            self.bit = 0;
            if self.is_running {
                return Some(FrameSignal::Octet(self.accum));
            }
        }

        None
    }
}

#[derive(Clone, Default, Debug)]
pub struct FrameCollector {
    frame: Vec<u8>,
}

impl Reset for FrameCollector {
    fn reset(&mut self) {
        self.frame.clear();
    }
}

impl Delay for FrameCollector {
    fn delay(&self) -> usize {
        0
    }
}

impl OneToOne<Option<FrameSignal>> for FrameCollector {
    type Output = Option<Vec<u8>>;

    fn filter(&mut self, sample: Option<FrameSignal>) -> Self::Output {
        match sample {
            Some(FrameSignal::Octet(x)) => {
                self.frame.push(x);
                None
            }
            Some(FrameSignal::FrameMarker) if !self.frame.is_empty() => {
                let mut x = vec![];
                swap(&mut x, &mut self.frame);

                Some(x)
            }
            Some(FrameSignal::DecodeError) => {
                self.reset();
                None
            }
            _ => None,
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct BitExtractor {
    sample_rate: u32,
    bit_rate: u32,
    accumulator: u32,
    last_bit: bool,
}

impl BitExtractor {
    pub fn new(sample_rate: u32, bit_rate: u32) -> BitExtractor {
        BitExtractor {
            sample_rate,
            bit_rate,
            ..Default::default()
        }
    }
}

impl Delay for BitExtractor {
    fn delay(&self) -> usize {
        0
    }
}

impl Reset for BitExtractor {
    fn reset(&mut self) {
        self.accumulator = 0;
        self.last_bit = false;
    }
}

impl OneToOne<Option<bool>> for BitExtractor {
    type Output = Option<bool>;

    fn filter(&mut self, sample: Option<bool>) -> Self::Output {
        if let Some(sample) = sample {
            if self.last_bit == sample {
                if self.accumulator < self.bit_rate {
                    self.accumulator += self.sample_rate - self.bit_rate;
                    Some(sample)
                } else {
                    self.accumulator -= self.bit_rate;
                    None
                }
            } else {
                self.accumulator = self.sample_rate / 2;
                self.last_bit = sample;
                None
            }
        } else {
            self.reset();
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hdlc_decode() {
        let mut decode = HdlcDecode::default();

        // Random bits at the start
        assert_eq!(decode.filter(false), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(false), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(false), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);

        // Send some frame markers
        assert_eq!(decode.filter(false), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(false), Some(FrameSignal::FrameMarker));
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(false), Some(FrameSignal::FrameMarker));
        assert_eq!(decode.filter(false), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(false), Some(FrameSignal::FrameMarker));

        // Send a 0x0F
        assert_eq!(decode.filter(false), None);
        assert_eq!(decode.filter(false), None);
        assert_eq!(decode.filter(false), None);
        assert_eq!(decode.filter(false), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), Some(FrameSignal::Octet(0xF0)));

        // Send a 0xF0
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(false), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(false), None);
        assert_eq!(decode.filter(false), None);
        assert_eq!(decode.filter(false), None);
        assert_eq!(decode.filter(false), Some(FrameSignal::Octet(0x0F)));

        // Send a frame marker
        assert_eq!(decode.filter(false), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(false), Some(FrameSignal::FrameMarker));

        // Trigger a decode error
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), Some(FrameSignal::DecodeError));

        // Random bits at the end
        assert_eq!(decode.filter(false), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(false), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(false), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
        assert_eq!(decode.filter(true), None);
    }

    #[test]
    fn bit_extractor_decode() {
        let mut decode = BitExtractor::new(20, 10);

        assert_eq!(decode.filter(Some(false)), Some(false));
        assert_eq!(decode.filter(Some(false)), None);
        assert_eq!(decode.filter(Some(false)), Some(false));
        assert_eq!(decode.filter(Some(false)), None);
        assert_eq!(decode.filter(Some(false)), Some(false));

        let mut decode = BitExtractor::new(30, 10);

        assert_eq!(decode.filter(None), None);
        assert_eq!(decode.filter(None), None);
        assert_eq!(decode.filter(Some(false)), Some(false));
        assert_eq!(decode.filter(Some(false)), None);
        assert_eq!(decode.filter(Some(false)), None);
        assert_eq!(decode.filter(Some(false)), Some(false));
        assert_eq!(decode.filter(Some(false)), None);
        assert_eq!(decode.filter(Some(false)), None);
        assert_eq!(decode.filter(Some(false)), Some(false));
        assert_eq!(decode.filter(Some(true)), None);
        assert_eq!(decode.filter(Some(true)), None);
        assert_eq!(decode.filter(Some(true)), Some(true));
        assert_eq!(decode.filter(Some(true)), None);
        assert_eq!(decode.filter(Some(true)), None);
        assert_eq!(decode.filter(Some(true)), Some(true));
        assert_eq!(decode.filter(Some(true)), None);
        assert_eq!(decode.filter(Some(true)), None);
        assert_eq!(decode.filter(Some(true)), Some(true));
    }
}