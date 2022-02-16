use crate::filter::{HdlcEncoderIter, NrziEncode, OneToOne, ResampleNN};

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

impl<T: Iterator, F: OneToOne<T::Item>> Iterator for OneToOneIter<T, F> {
    type Item = F::Output;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(x) = self.iter.next() {
            Some(self.filter.filter(x))
        } else {
            None
        }
    }
}

pub trait IteratorExt: Iterator {
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

    fn resample_nn(self, scale: f32) -> ResampleNN<Self>
    where
        Self: std::marker::Sized + Iterator,
        <Self as Iterator>::Item: Clone,
    {
        ResampleNN::new(self, scale)
    }

    fn apply_one_to_one<F>(self, filter: F) -> OneToOneIter<Self, F>
    where
        F: OneToOne<Self::Item>,
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
