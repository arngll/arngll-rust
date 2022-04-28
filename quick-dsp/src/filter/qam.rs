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
use std::marker::PhantomData;

/// QAM Splitter.
///
/// Carrier is fixed at 0.25 (freq/4)
///
/// Output is (i, q)
#[derive(Clone, Debug)]
pub struct QamSplitFixed<T, F = ()> {
    current_step: u8,
    filter_i: F,
    filter_q: F,
    _t: PhantomData<T>,
}

impl<T, F: Delay> Delay for QamSplitFixed<T, F> {
    fn delay(&self) -> usize {
        (self.filter_i.delay() + self.filter_q.delay()) / 2
    }
}

impl<T: Real, F> QamSplitFixed<T, F> {
    pub fn digital_default() -> QamSplitFixed<T, FilterFir<T>> {
        Self::new(FilterFirKernel::<T>::low_pass(15, 0.1, Window::Blackman).into_filter())
    }

    pub fn analog_default() -> QamSplitFixed<T, FilterFir<T>> {
        Self::new(FilterFirKernel::<T>::low_pass(21, 0.25, Window::Blackman).into_filter())
    }

    pub fn new<K: Filter<T> + Clone>(kiq: K) -> QamSplitFixed<T, K> {
        QamSplitFixed {
            current_step: 0,
            filter_i: kiq.clone(),
            filter_q: kiq,
            _t: Default::default(),
        }
    }
}

impl<T, F: Filter<T, Output = T>> Filter<T> for QamSplitFixed<T, F>
where
    T: Real,
{
    type Output = (T, T); // (i,q)

    fn filter(&mut self, sample: T) -> Self::Output {
        if !sample.is_finite() {
            return (T::NAN, T::NAN);
        }

        let (v_i, v_q) = match self.current_step & 3 {
            0 => (sample, T::ZERO),
            1 => (T::ZERO, sample),
            2 => (-sample, T::ZERO),
            _ => (T::ZERO, -sample),
        };
        self.current_step = self.current_step.wrapping_add(1);

        (self.filter_i.filter(v_i), self.filter_q.filter(v_q))
    }
}

#[cfg(test)]
mod tests {
    //use super::*;
}
