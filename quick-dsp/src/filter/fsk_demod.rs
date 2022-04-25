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

use super::*;

/// One-bit decimator. Used for converting frequency data into a bitstream.
#[derive(Clone, Debug)]
pub struct FskDemod<T> {
    pub offset: T,
    pub scale: T,
    pub last_mag: T,
}

impl<T: Real> FskDemod<T> {
    pub fn new(zero: T, one: T) -> Self {
        FskDemod {
            offset: (zero + one) * T::from_f64(0.5),
            scale: T::from_f64(1.0) / (one - zero) * T::from_f64(2.0),
            last_mag: T::ZERO,
        }
    }
}

impl<T: Real> Filter<(T, T)> for FskDemod<T> {
    type Output = Option<bool>;

    fn filter(&mut self, sample: (T, T)) -> Self::Output {
        if !sample.0.is_finite() || sample.0 <= T::ZERO {
            return None;
        }

        // After this, v should be between -1.0 and 1.0
        let v = (sample.0 - self.offset) * self.scale;

        Some(v > T::ZERO)
    }
}

impl<T> Delay for FskDemod<T> {
    fn delay(&self) -> usize {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fsk_demod_f32() {
        let disc = Discriminator::<f32, (), ()>::digital_default();
        let mut disc = disc.chain(FskDemod::new(0.2, 0.3));

        let mut modulator = FmMod::<f32>::new(1.0);

        for _i in 0..100 {
            let sample = modulator.filter(0.2);
            let result = disc.filter(sample);
            println!("fsk_demod_f32(0.20) = {:?}", result);
        }

        for _i in 0..100 {
            let sample = modulator.filter(0.3);
            let result = disc.filter(sample);
            println!("fsk_demod_f32(0.30) = {:?}", result);
        }

        for _i in 0..100 {
            let sample = modulator.filter(0.24);
            let result = disc.filter(sample);
            println!("fsk_demod_f32(0.24) = {:?}", result);
        }
    }
}
