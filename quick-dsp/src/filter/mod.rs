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

use circular_queue::CircularQueue;
use std::convert::TryFrom;
use std::fmt::{Debug, Display};
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub, SubAssign};

mod boxfilter;
mod decimator;
mod discriminator;
mod fir;
mod fm_mod;
mod fsk_demod;
mod hdlc;
mod iir;
mod iter;
mod nrzi;
mod qam;
mod resample;

pub use boxfilter::*;
pub use decimator::*;
pub use discriminator::*;
pub use fir::*;
pub use fm_mod::*;
pub use fsk_demod::*;
pub use hdlc::*;
pub use iir::*;
pub use iter::*;
pub use nrzi::*;
pub use qam::*;
pub use resample::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Window {
    Rectangular,
    Bartlett,
    Hann,
    Hamming,
    Blackman,
    Nuttall,
    BlackmanNuttall,
    BlackmanHarris,
}

pub trait WindowFunc {
    fn window_func(&self, t: f64, l: f64) -> f64;
}

impl WindowFunc for Window {
    fn window_func(&self, t: f64, l: f64) -> f64 {
        match self {
            Window::Rectangular => 1.0,
            Window::Bartlett => 2.0 * ((l / 2.0) + t.abs()) / l,
            Window::Hann => 0.5 - 0.5 * f64::cos((f64::PI * 2.0 * t) / l),
            Window::Hamming => 0.54 - 0.46 * f64::cos((f64::PI * 2.0 * t) / l),
            Window::Blackman => {
                0.42 - 0.5 * f64::cos((f64::PI * 2.0 * t) / l)
                    + 0.08 * f64::cos((f64::PI * 4.0 * t) / l)
            }
            Window::Nuttall => {
                0.355768 - 0.487396 * f64::cos((f64::PI * 2.0 * t) / l)
                    + 0.144232 * f64::cos((f64::PI * 4.0 * t) / l)
                    - 0.012604 * f64::cos((f64::PI * 6.0 * t) / l)
            }
            Window::BlackmanNuttall => {
                0.3635819 - 0.4891775 * f64::cos((f64::PI * 2.0 * t) / l)
                    + 0.1365995 * f64::cos((f64::PI * 4.0 * t) / l)
                    - 0.0106411 * f64::cos((f64::PI * 6.0 * t) / l)
            }
            Window::BlackmanHarris => {
                0.35875 - 0.48829 * f64::cos((f64::PI * 2.0 * t) / l)
                    + 0.14128 * f64::cos((f64::PI * 4.0 * t) / l)
                    - 0.01168 * f64::cos((f64::PI * 6.0 * t) / l)
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum FilterType {
    LowPass,
    HighPass,
    BandPass,
    BandStop,
}

impl FilterType {
    pub fn is_band(self) -> bool {
        match self {
            Self::BandPass | Self::BandStop => true,
            Self::LowPass | Self::HighPass => false,
        }
    }

    pub fn is_band_pass(self) -> bool {
        match self {
            Self::BandPass => true,
            _ => false,
        }
    }

    pub fn is_high_pass(self) -> bool {
        match self {
            Self::HighPass => true,
            _ => false,
        }
    }

    pub fn is_low_pass(self) -> bool {
        match self {
            Self::LowPass => true,
            _ => false,
        }
    }
}

pub trait IntoFilter<T> {
    type Filter: Filter<T>;
    fn into_filter(self) -> Self::Filter;
}

pub trait Filter<T> {
    type Output;
    fn filter(&mut self, sample: T) -> Self::Output;
}

pub trait FilterExt<T>: Filter<T> + Sized {
    fn chain<B: Filter<Self::Output>>(self, b: B) -> Chain<Self, B>
    where
        Self: Sized,
    {
        Chain(self, b)
    }

    fn optional(self) -> Optional<Self> {
        Optional(self)
    }

    fn inspect<F: Fn(&Self::Output)>(self, f: F) -> Inspect<Self, F> {
        Inspect(self, f)
    }

    fn boxed<'a>(self) -> BoxedFilter<'a, T, Self::Output>
    where
        Self: 'a + Sized,
    {
        Box::new(self) as BoxedFilter<T, Self::Output>
    }
}
impl<T: Filter<A>, A> FilterExt<A> for T {}

pub type BoxedFilter<'a, In, Out = In> = Box<dyn Filter<In, Output = Out> + 'a>;

impl<F, T> Filter<T> for Box<F>
where
    F: Filter<T>,
{
    type Output = F::Output;

    fn filter(&mut self, sample: T) -> Self::Output {
        self.as_mut().filter(sample)
    }
}

impl<F: Delay> Delay for Box<F> {
    fn delay(&self) -> usize {
        self.as_ref().delay()
    }
}

pub struct Optional<A: Sized>(A);

impl<T, A> Filter<Option<T>> for Optional<A>
where
    A: Filter<T>,
{
    type Output = Option<A::Output>;

    fn filter(&mut self, sample: Option<T>) -> Self::Output {
        if let Some(sample) = sample {
            Some(self.0.filter(sample))
        } else {
            None
        }
    }
}

impl<A: Delay> Delay for Optional<A> {
    fn delay(&self) -> usize {
        self.0.delay()
    }
}

impl<A: Reset> Reset for Optional<A> {
    fn reset(&mut self) {
        self.0.reset();
    }
}

pub struct Inspect<T, F>(T, F);
impl<T, X, F> Filter<X> for Inspect<T, F>
where
    T: Filter<X>,
    F: Fn(&T::Output),
{
    type Output = T::Output;

    fn filter(&mut self, sample: X) -> Self::Output {
        let ret = self.0.filter(sample);
        self.1(&ret);
        ret
    }
}

impl<A: Delay, F> Delay for Inspect<A, F> {
    fn delay(&self) -> usize {
        self.0.delay()
    }
}

impl<A: Reset, F> Reset for Inspect<A, F> {
    fn reset(&mut self) {
        self.0.reset();
    }
}

pub struct Chain<A, B>(A, B);

impl<T, A, B> Filter<T> for Chain<A, B>
where
    A: Filter<T>,
    B: Filter<A::Output>,
{
    type Output = B::Output;

    fn filter(&mut self, sample: T) -> Self::Output {
        self.1.filter(self.0.filter(sample))
    }
}

impl<A: Delay, B: Delay> Delay for Chain<A, B> {
    fn delay(&self) -> usize {
        self.0.delay() + self.1.delay()
    }
}

impl<A: Reset, B: Reset> Reset for Chain<A, B> {
    fn reset(&mut self) {
        self.0.reset();
        self.1.reset();
    }
}

pub trait Delay {
    /// The amount of delay this filter adds, measured in samples.
    fn delay(&self) -> usize;
}

pub trait Reset {
    fn reset(&mut self);
}

pub trait Real:
    Debug
    + Default
    + num::Float
    + Copy
    + Display
    + std::cmp::PartialEq
    + Div<Output = Self>
    + Sub<Output = Self>
    + Add<Output = Self>
    + Mul<Output = Self>
    + Neg<Output = Self>
    + PartialOrd
    + AddAssign
    + SubAssign
    + MulAssign
    + std::iter::Sum<<Self as std::ops::Mul>::Output>
    + Into<f64>
{
    const NAN: Self;
    const TAU: Self;
    const PI: Self;
    const E: Self;
    const ZERO: Self;
    const ONE: Self;
    const HALF: Self;
    const TWO: Self;
    const FORTH: Self;

    fn from_f64(v: f64) -> Self;
    fn from_usize(v: usize) -> Self {
        Self::from_f64(v as f64)
    }
}

impl Real for f64 {
    const NAN: Self = Self::ZERO / Self::ZERO;
    const TAU: Self = 6.28318530717958647692528676655900577_f64;
    const PI: Self = Self::TAU / 2.0;
    const E: Self = 2.71828182845904523536028747135266250_f64;
    const ZERO: Self = 0.0f64;
    const ONE: Self = 1.0f64;
    const HALF: Self = 0.5f64;
    const TWO: Self = 2.0f64;
    const FORTH: Self = 0.25f64;

    fn from_f64(v: f64) -> Self {
        v as Self
    }
}

impl Real for f32 {
    const NAN: Self = Self::ZERO / Self::ZERO;
    const TAU: Self = 6.28318530717958647692528676655900577_f32;
    const PI: Self = Self::TAU / 2.0;
    const E: Self = 2.71828182845904523536028747135266250_f32;
    const ZERO: Self = 0.0f32;
    const ONE: Self = 1.0f32;
    const HALF: Self = 0.5f32;
    const TWO: Self = 2.0f32;
    const FORTH: Self = 0.25f32;

    fn from_f64(v: f64) -> Self {
        v as Self
    }
}

pub fn calc_dbs<T: Real>(zero: T, x: T) -> T {
    (x / zero).log10() * T::from_usize(10)
}

pub fn calc_gain<T: Real, F: Filter<T, Output = T> + Delay>(mut filter: F, freq: T) -> T {
    let phase_delta = T::TAU * freq;
    let mut phase = T::ZERO;
    for _ in 0..(filter.delay() * 4 + 1000) {
        filter.filter(phase.cos());
        phase += phase_delta;
    }

    let mut max_signal = T::ZERO;
    for _ in 0..(filter.delay() * 4 + 1000) {
        let x = filter.filter(phase.cos()).abs();
        phase += phase_delta;
        if x > max_signal {
            max_signal = x;
        }
    }

    calc_dbs(T::ONE, max_signal)
}
