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
pub struct Downsampler<T> {
    filter: FilterFir<T>,
    skip: bool,
    out_sample_rate: u32,
    accumulator: u32,
    inter_sample_rate: u32,
    inter_factor: u32,
}

impl<T: Real> Downsampler<T> {
    pub fn new(in_sample_rate: u32, out_sample_rate: u32) -> Downsampler<T> {
        assert!(
            in_sample_rate >= out_sample_rate,
            "Downsampler output rate must be smaller than input rate"
        );

        if in_sample_rate == out_sample_rate {
            // Return a special downsampler that doesn't do anything.
            return Downsampler {
                filter: FilterFir::<T>::low_pass(1, 0.5, Window::Rectangular),
                skip: true,
                out_sample_rate,
                accumulator: 0,
                inter_sample_rate: 0,
                inter_factor: 0,
            };
        }

        let inter_factor = std::cmp::max(1, 6 * out_sample_rate / in_sample_rate);
        let inter_sample_rate = in_sample_rate * inter_factor;
        let cutoff =
            0.5f64 / (inter_factor as f64) * (out_sample_rate as f64) / (in_sample_rate as f64);
        Downsampler {
            filter: FilterFir::<T>::low_pass(50, cutoff, Window::Blackman),
            skip: false,
            out_sample_rate,
            accumulator: 0,
            inter_sample_rate,
            inter_factor,
        }
    }
}

impl<T: Real> Filter<T> for Downsampler<T> {
    type Output = Option<T>;

    fn filter(&mut self, mut sample: T) -> Self::Output {
        if self.skip {
            return Some(sample);
        }

        let mut ret = None;
        let mult = T::from_f64(self.inter_factor as f64);

        for i in 0..self.inter_factor {
            if i != 0 {
                sample = T::ZERO;
            }
            let v = self.filter.filter(sample * mult);
            self.accumulator += self.out_sample_rate;
            if self.accumulator > self.inter_sample_rate {
                self.accumulator -= self.inter_sample_rate;
                ret = Some(v);
            }
        }

        ret
    }
}

/// Resampling nearest-neighbor iterator
pub struct ResampleNN<I: Iterator> {
    inner: I,
    curr: Option<I::Item>,
    leftover: f32,
    scale: f32,
}

impl<I: Iterator> ResampleNN<I> {
    pub fn new(inner: I, scale: f32) -> Self {
        ResampleNN {
            inner,
            curr: None,
            leftover: 0.0,
            scale,
        }
    }
}

impl<I: Iterator> Iterator for ResampleNN<I>
where
    I::Item: Clone,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        while self.leftover < 1.0 {
            self.leftover += self.scale;
            self.curr = self.inner.next();
            if self.curr.is_none() {
                break;
            }
        }
        self.leftover -= 1.0;
        self.curr.clone()
    }
}
