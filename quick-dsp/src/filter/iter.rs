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

use crate::filter::{Filter, HdlcEncoderIter, NrziEncode, ResampleNN};

/// Transforms an iterator over bytes into an iterator over bits,
/// most significant bit first.
pub struct MsbIterator<T> {
    iter: T,
    byte: u8,
    bit: u8,
}

impl<T: Iterator<Item = u8>> Iterator for MsbIterator<T> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bit == 0 {
            self.byte = self.iter.next()?;
            self.bit = 8;
        }
        self.bit -= 1;
        Some((self.byte & (1 << self.bit)) != 0)
    }
}

/// Transforms an iterator over bytes into an iterator over bits,
/// least significant bit first.
pub struct LsbIterator<T> {
    iter: T,
    byte: u8,
    bit: u8,
}

impl<T: Iterator<Item = u8>> Iterator for LsbIterator<T> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bit == 0 {
            self.byte = self.iter.next()?;
            self.bit = 8;
        }
        self.bit -= 1;
        Some((self.byte & (1 << (7 - self.bit))) != 0)
    }
}

pub struct OneToOneIter<T, F> {
    iter: T,
    filter: F,
}

impl<T: Iterator, F: Filter<T::Item>> Iterator for OneToOneIter<T, F> {
    type Item = F::Output;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(x) = self.iter.next() {
            Some(self.filter.filter(x))
        } else {
            None
        }
    }
}

pub enum CrcAppendIter<T> {
    Running(T, crc::Digest<'static, u16>),
    Finishing(u8),
    Finished,
}

impl<T> CrcAppendIter<T> {
    pub fn new(iter: T, crc: &'static crc::Crc<u16>) -> Self {
        Self::Running(iter, crc.digest())
    }
}
impl<T: Iterator<Item = u8>> Iterator for CrcAppendIter<T> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let mut this = Self::Finished;
        // TODO: Rewrite to not need this swap.
        std::mem::swap(&mut this, self);
        match this {
            CrcAppendIter::Running(mut iter, mut digest) => {
                if let Some(byte) = iter.next() {
                    digest.update(&[byte]);
                    *self = CrcAppendIter::Running(iter, digest);
                    Some(byte)
                } else {
                    let crc = digest.finalize().to_le_bytes();
                    *self = Self::Finishing(crc[1]);
                    Some(crc[0])
                }
            }
            CrcAppendIter::Finishing(last_byte) => {
                *self = CrcAppendIter::Finished;
                Some(last_byte)
            }
            CrcAppendIter::Finished => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            CrcAppendIter::Running(iter, ..) => {
                let hint = iter.size_hint();
                (hint.0 + 2, hint.1.map(|x| x + 2))
            }
            CrcAppendIter::Finishing(_) => (1, Some(1)),
            CrcAppendIter::Finished => (0, Some(0)),
        }
    }
}

pub trait IteratorExt: Iterator {
    fn append_crc(self, crc: &'static crc::Crc<u16>) -> CrcAppendIter<Self>
    where
        Self: std::marker::Sized + Iterator<Item = u8>,
    {
        CrcAppendIter::new(self, crc)
    }

    fn bits_msb(self) -> MsbIterator<Self>
    where
        Self: std::marker::Sized + Iterator<Item = u8>,
    {
        MsbIterator {
            iter: self,
            byte: 0,
            bit: 0,
        }
    }

    fn bits_lsb(self) -> LsbIterator<Self>
    where
        Self: std::marker::Sized + Iterator<Item = u8>,
    {
        LsbIterator {
            iter: self,
            byte: 0,
            bit: 0,
        }
    }

    fn hdlc_encode(self) -> HdlcEncoderIter<Self>
    where
        Self: std::marker::Sized + Iterator<Item = bool>,
    {
        HdlcEncoderIter::new(self)
    }

    /// Resample values, nearest-neighbor
    fn resample_nn(self, scale: f32) -> ResampleNN<Self>
    where
        Self: std::marker::Sized + Iterator,
        <Self as Iterator>::Item: Clone,
    {
        ResampleNN::new(self, scale)
    }

    fn apply_one_to_one<F>(self, filter: F) -> OneToOneIter<Self, F>
    where
        F: Filter<Self::Item>,
        Self: std::marker::Sized + Iterator,
    {
        OneToOneIter { iter: self, filter }
    }

    fn nrzi_encode(self) -> OneToOneIter<Self, NrziEncode>
    where
        Self: std::marker::Sized + Iterator<Item = bool>,
    {
        self.apply_one_to_one(NrziEncode::new())
    }
}

impl<T: Iterator> IteratorExt for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append_crc() {
        let vec_golden: Vec<u8> = hex::decode("82a0aa646a9ce0ae8270989a8c60ae92888a62406303f03e3230323333377a687474703a2f2f7761386c6d662e636f6d0df782").unwrap();

        let vec: Vec<u8> = hex::decode("82a0aa646a9ce0ae8270989a8c60ae92888a62406303f03e3230323333377a687474703a2f2f7761386c6d662e636f6d0d").unwrap();
        const X25: crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_IBM_SDLC);

        assert_eq!(
            vec_golden,
            vec.into_iter().append_crc(&X25).collect::<Vec<_>>()
        );
    }
    #[test]
    fn msb_bit_iterator() {
        let vec = vec![0x0Fu8, 0xF0u8];
        let iter = vec.into_iter();
        let mut bit_iter = iter.bits_msb();

        assert_eq!(bit_iter.next(), Some(false));
        assert_eq!(bit_iter.next(), Some(false));
        assert_eq!(bit_iter.next(), Some(false));
        assert_eq!(bit_iter.next(), Some(false));
        assert_eq!(bit_iter.next(), Some(true));
        assert_eq!(bit_iter.next(), Some(true));
        assert_eq!(bit_iter.next(), Some(true));
        assert_eq!(bit_iter.next(), Some(true));
        assert_eq!(bit_iter.next(), Some(true));
        assert_eq!(bit_iter.next(), Some(true));
        assert_eq!(bit_iter.next(), Some(true));
        assert_eq!(bit_iter.next(), Some(true));
        assert_eq!(bit_iter.next(), Some(false));
        assert_eq!(bit_iter.next(), Some(false));
        assert_eq!(bit_iter.next(), Some(false));
        assert_eq!(bit_iter.next(), Some(false));
        assert_eq!(bit_iter.next(), None);
    }

    #[test]
    fn lsb_bit_iterator() {
        let vec = vec![0xf0u8, 0x0fu8];
        let iter = vec.into_iter();
        let mut bit_iter = iter.bits_lsb();

        assert_eq!(bit_iter.next(), Some(false));
        assert_eq!(bit_iter.next(), Some(false));
        assert_eq!(bit_iter.next(), Some(false));
        assert_eq!(bit_iter.next(), Some(false));
        assert_eq!(bit_iter.next(), Some(true));
        assert_eq!(bit_iter.next(), Some(true));
        assert_eq!(bit_iter.next(), Some(true));
        assert_eq!(bit_iter.next(), Some(true));
        assert_eq!(bit_iter.next(), Some(true));
        assert_eq!(bit_iter.next(), Some(true));
        assert_eq!(bit_iter.next(), Some(true));
        assert_eq!(bit_iter.next(), Some(true));
        assert_eq!(bit_iter.next(), Some(false));
        assert_eq!(bit_iter.next(), Some(false));
        assert_eq!(bit_iter.next(), Some(false));
        assert_eq!(bit_iter.next(), Some(false));
        assert_eq!(bit_iter.next(), None);
    }
}
