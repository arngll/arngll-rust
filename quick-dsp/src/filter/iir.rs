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

//! IIR filter. Not yet complete.

use super::*;

#[derive(Clone, Debug)]
pub struct FilterIirKernel<T> {
    a: Vec<T>,
    b: Vec<T>,
    delay: usize,
}

impl<T: Real> Kernel for FilterIirKernel<T> {
    type Filter = FilterIir<T>;

    fn into_filter(self) -> Self::Filter {
        FilterIir::from_kernel(self)
    }
}

impl<T> FilterIirKernel<T> {
    pub fn new(a: Vec<T>, b: Vec<T>, delay: usize) -> Self {
        FilterIirKernel { a, b, delay }
    }

    pub fn len(&self) -> usize {
        self.a.len()
    }

    pub fn poles(&self) -> usize {
        self.a.len() - 1
    }
}

impl<T: Real> FilterIirKernel<T> {
    pub fn low_pass(_poles: usize, _cutoff: f64, _window: Window) -> Self {
        todo!()
    }

    pub fn high_pass(_poles: usize, _cutoff: f64, _window: Window) -> Self {
        todo!()
    }

    pub fn band_pass(_poles: usize, _lcutoff: f64, _hcutoff: f64, _window: Window) -> Self {
        todo!()
    }
}

impl<T: Real> From<FilterIirKernel<T>> for FilterIir<T> {
    fn from(kernel: FilterIirKernel<T>) -> Self {
        FilterIir::from_kernel(kernel)
    }
}

#[derive(Clone, Debug)]
pub struct FilterIir<T> {
    kernel: FilterIirKernel<T>,
    x: CircularQueue<T>,
    y: CircularQueue<T>,
}

impl<T: Real> FilterIir<T> {
    pub fn from_kernel(kernel: FilterIirKernel<T>) -> Self {
        FilterIir {
            x: CircularQueue::with_capacity(kernel.len()),
            y: CircularQueue::with_capacity(kernel.len()),
            kernel,
        }
    }

    pub fn low_pass(poles: usize, cutoff: f64, window: Window) -> Self {
        FilterIirKernel::low_pass(poles, cutoff, window).into()
    }

    pub fn high_pass(poles: usize, cutoff: f64, window: Window) -> Self {
        FilterIirKernel::high_pass(poles, cutoff, window).into()
    }

    pub fn band_pass(poles: usize, lcutoff: f64, hcutoff: f64, window: Window) -> Self {
        FilterIirKernel::band_pass(poles, lcutoff, hcutoff, window).into()
    }
}

impl<T: Debug> Delay for FilterIir<T> {
    fn delay(&self) -> usize {
        self.kernel.delay
    }
}

impl<T> OneToOne<T> for FilterIir<T>
where
    T: Real,
{
    type Output = T;
    fn filter(&mut self, sample: T) -> T {
        if !sample.is_finite() {
            return sample;
        }

        self.x.push(sample);
        self.y.push(
            self.x
                .iter()
                .zip(self.kernel.a.iter())
                .map(|(x, a)| x.mul(*a))
                .sum(),
        );

        let output: T = self
            .y
            .iter()
            .zip(self.kernel.b.iter())
            .map(|(x, b)| x.mul(*b))
            .sum();

        if output.is_finite() {
            *self.y.iter_mut().next().unwrap() = output;
        }

        output
    }
}
