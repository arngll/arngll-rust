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

/// FM Modulator.
#[derive(Clone, Debug)]
pub struct FmMod<T> {
    phase: T,
    amplitude: T,
}

impl<T: Real> FmMod<T> {
    pub fn new(amplitude: T) -> Self {
        FmMod {
            phase: T::ZERO,
            amplitude,
        }
    }
}

impl<T: Real> Filter<T> for FmMod<T> {
    type Output = T;

    fn filter(&mut self, sample: T) -> Self::Output {
        self.phase += sample * T::TAU;
        if self.phase > T::TAU {
            self.phase -= T::TAU;
        }
        self.phase.sin() * self.amplitude
    }
}

impl<T> Delay for FmMod<T> {
    fn delay(&self) -> usize {
        0
    }
}
