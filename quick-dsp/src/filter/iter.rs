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

pub trait IteratorExt: Iterator {
    fn bits_msb(self) -> MsbIterator<Self>
    where
        Self: std::marker::Sized;
    fn bits_lsb(self) -> LsbIterator<Self>
    where
        Self: std::marker::Sized;
}

impl<T: Iterator<Item = u8>> IteratorExt for T {
    fn bits_msb(self) -> MsbIterator<Self>
    where
        Self: std::marker::Sized,
    {
        MsbIterator {
            iter: self,
            byte: 0,
            bit: 0,
        }
    }
    fn bits_lsb(self) -> LsbIterator<Self>
    where
        Self: std::marker::Sized,
    {
        LsbIterator {
            iter: self,
            byte: 0,
            bit: 0,
        }
    }
}

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
