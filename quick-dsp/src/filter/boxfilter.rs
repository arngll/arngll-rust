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

//! FIR Filter.

use super::*;
use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct FilterBox<T>(CircularQueue<T>);

impl<T> Delay for FilterBox<T> {
    fn delay(&self) -> usize {
        self.0.len() / 2
    }
}

impl<T: Real> FilterBox<T> {
    pub fn new(len: usize) -> Self {
        FilterBox(CircularQueue::with_capacity(len))
    }
}

impl<T: Real> Filter<T> for FilterBox<T> {
    type Output = T;

    fn filter(&mut self, sample: T) -> T {
        self.0.push(sample);

        let ret: T = self.0.iter().copied().sum();

        ret / T::from_f64(self.0.len() as f64)
    }
}

#[derive(Clone, Debug)]
pub struct FilterMedian<T, const N: usize>([T; N]);

impl<T, const N: usize> Delay for FilterMedian<T, N> {
    fn delay(&self) -> usize {
        N / 2
    }
}

impl<T: Default, const N: usize> Default for FilterMedian<T, N>
where
    [T; N]: Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: Default, const N: usize> FilterMedian<T, N>
where
    [T; N]: Default,
{
    pub fn new() -> Self {
        Default::default()
    }
}

impl<T: Default + Copy + PartialOrd, const N: usize> Filter<T> for FilterMedian<T, N> {
    type Output = T;

    fn filter(&mut self, sample: T) -> T {
        // Not super fast, but it works.
        self.0.copy_within(0..N - 1, 1);
        self.0[0] = sample;
        let mut x = self.0.clone();
        x.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        x[N / 2]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_box2() {
        let mut filter = FilterBox::new(2);

        println!("Filter = {:?}", filter);
        assert_eq!(filter.filter(0.0), 0.0);
        assert_eq!(filter.filter(1.0), 0.5);
        assert_eq!(filter.filter(0.0), 0.5);
        assert_eq!(filter.filter(1.0), 0.5);
        assert_eq!(filter.filter(0.0), 0.5);
        assert_eq!(filter.filter(1.0), 0.5);
        assert_eq!(filter.filter(0.0), 0.5);
        assert_eq!(filter.filter(1.0), 0.5);
        assert_eq!(filter.filter(0.0), 0.5);
    }

    #[test]
    fn filter_box3() {
        let mut filter = FilterBox::new(3);

        println!("Filter = {:?}", filter);
        assert_eq!(filter.filter(0.0), 0.0);
        assert_eq!(filter.filter(1.0), 0.5);
        assert_eq!(filter.filter(0.0), 1.0 / 3.0);
        assert_eq!(filter.filter(0.0), 1.0 / 3.0);
        assert_eq!(filter.filter(1.0), 1.0 / 3.0);
        assert_eq!(filter.filter(0.0), 1.0 / 3.0);
        assert_eq!(filter.filter(0.0), 1.0 / 3.0);
        assert_eq!(filter.filter(1.0), 1.0 / 3.0);
        assert_eq!(filter.filter(0.0), 1.0 / 3.0);
        assert_eq!(filter.filter(0.0), 1.0 / 3.0);
        assert_eq!(filter.filter(1.0), 1.0 / 3.0);
    }
}
