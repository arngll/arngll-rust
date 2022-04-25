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

#[derive(Clone, Debug)]
pub struct Decimator<F, I> {
    offset: F,
    scale: F,
    _error: F,
    nanvalue: I,
}

impl<F, I> Delay for Decimator<F, I> {
    fn delay(&self) -> usize {
        0
    }
}

impl Default for Decimator<f32, f32> {
    fn default() -> Self {
        Decimator {
            offset: 0.0,
            scale: 0.0,
            _error: 0.0,
            nanvalue: 0.0,
        }
    }
}

impl Filter<f32> for Decimator<f32, f32> {
    type Output = f32;

    fn filter(&mut self, sample: f32) -> Self::Output {
        sample
    }
}

impl<F: Real> Default for Decimator<F, i8> {
    fn default() -> Self {
        Self::new(-F::ONE, F::ONE)
    }
}
impl<F: Real> Decimator<F, i8> {
    pub fn new(min: F, max: F) -> Self {
        Decimator {
            offset: -(max + min) / F::TWO,
            scale: F::from_f64(255.0) / (max - min),
            _error: F::ZERO,
            nanvalue: 0,
        }
    }
}
impl<F: Real> Filter<F> for Decimator<F, i8> {
    type Output = i8;

    fn filter(&mut self, sample: F) -> Self::Output {
        if sample.is_finite() {
            num::clamp(
                (sample + self.offset) * self.scale,
                F::from_f64(-128.0),
                F::from_f64(127.0),
            )
            .to_i8()
            .unwrap()
        } else {
            self.nanvalue
        }
    }
}

impl<F: Real> Default for Decimator<F, u8> {
    fn default() -> Self {
        Self::new(-F::ONE, F::ONE)
    }
}
impl<F: Real> Decimator<F, u8> {
    pub fn new(min: F, max: F) -> Self {
        Decimator {
            offset: -min,
            scale: F::from_f64(255.0) / (max - min),
            _error: F::ZERO,
            nanvalue: 128,
        }
    }
}
impl<F: Real> Filter<F> for Decimator<F, u8> {
    type Output = u8;

    fn filter(&mut self, sample: F) -> Self::Output {
        if sample.is_finite() {
            num::clamp(
                (sample + self.offset) * self.scale,
                F::from_f64(0.0),
                F::from_f64(255.0),
            )
            .to_u8()
            .unwrap()
        } else {
            self.nanvalue
        }
    }
}

impl<F: Real> Default for Decimator<F, i16> {
    fn default() -> Self {
        Self::new(-F::ONE, F::ONE)
    }
}
impl<F: Real> Decimator<F, i16> {
    pub fn new(min: F, max: F) -> Self {
        Decimator {
            offset: -(max + min) / F::TWO,
            scale: F::from_f64(65535.0) / (max - min),
            _error: F::ZERO,
            nanvalue: 0,
        }
    }
}
impl<F: Real> Filter<F> for Decimator<F, i16> {
    type Output = i16;

    fn filter(&mut self, sample: F) -> Self::Output {
        if sample.is_finite() {
            num::clamp(
                (sample + self.offset) * self.scale,
                F::from_f64(-32768.0),
                F::from_f64(32767.0),
            )
            .to_i16()
            .unwrap()
        } else {
            self.nanvalue
        }
    }
}

impl<F: Real> Default for Decimator<F, u16> {
    fn default() -> Self {
        Self::new(-F::ONE, F::ONE)
    }
}
impl<F: Real> Decimator<F, u16> {
    pub fn new(min: F, max: F) -> Self {
        Decimator {
            offset: -min,
            scale: F::from_f64(65535.0) / (max - min),
            _error: F::ZERO,
            nanvalue: 32768,
        }
    }
}
impl<F: Real> Filter<F> for Decimator<F, u16> {
    type Output = u16;

    fn filter(&mut self, sample: F) -> Self::Output {
        if sample.is_finite() {
            num::clamp(
                (sample + self.offset) * self.scale,
                F::from_f64(0.0),
                F::from_f64(65535.0),
            )
            .to_u16()
            .unwrap()
        } else {
            self.nanvalue
        }
    }
}
