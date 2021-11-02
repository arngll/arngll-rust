use circular_queue::CircularQueue;
use std::convert::TryFrom;
use std::fmt::{Debug, Display};
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub, SubAssign};

mod decimator;
mod discriminator;
mod downsampler;
mod fir;
mod fm_mod;
mod fsk_demod;
mod hdlc;
mod iir;
mod iter;
mod nrzi;

pub use decimator::*;
pub use discriminator::*;
pub use downsampler::*;
pub use fir::*;
pub use fm_mod::*;
pub use fsk_demod::*;
pub use hdlc::*;
pub use iir::*;
pub use iter::*;
pub use nrzi::*;

#[derive(Copy, Clone, Debug)]
pub enum Window {
    Hanning,
    Hamming,
    Blackman,
    Rectangular,
}

pub trait Kernel {
    type Filter;
    fn into_filter(self) -> Self::Filter;
}

pub trait OneToOne<T> {
    type Output;
    fn filter(&mut self, sample: T) -> Self::Output;
}

pub trait OneToOneExt<T>: OneToOne<T> + Sized {
    fn chain<B: OneToOne<Self::Output>>(self, b: B) -> OneToOneChain<Self, B>
    where
        Self: Sized,
    {
        OneToOneChain(self, b)
    }

    fn optional(self) -> OneToOneOptional<Self> {
        OneToOneOptional(self)
    }

    fn inspect<F: Fn(&Self::Output)>(self, f: F) -> OneToOneInspect<Self, F> {
        OneToOneInspect(self, f)
    }

    fn boxed<'a>(self) -> BoxOneToOne<'a, T, Self::Output>
    where
        Self: 'a + Sized,
    {
        Box::new(self) as BoxOneToOne<T, Self::Output>
    }
}
impl<T: OneToOne<A>, A> OneToOneExt<A> for T {}

pub type BoxOneToOne<'a, In, Out> = Box<dyn OneToOne<In, Output = Out> + 'a>;

pub struct OneToOneOptional<A: Sized>(A);

impl<T, A> OneToOne<Option<T>> for OneToOneOptional<A>
where
    A: OneToOne<T>,
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

impl<A: Delay> Delay for OneToOneOptional<A> {
    fn delay(&self) -> usize {
        self.0.delay()
    }
}

impl<A: Reset> Reset for OneToOneOptional<A> {
    fn reset(&mut self) {
        self.0.reset();
    }
}

pub struct OneToOneInspect<T, F>(T, F);
impl<T, X, F> OneToOne<X> for OneToOneInspect<T, F>
where
    T: OneToOne<X>,
    F: Fn(&T::Output),
{
    type Output = T::Output;

    fn filter(&mut self, sample: X) -> Self::Output {
        let ret = self.0.filter(sample);
        self.1(&ret);
        ret
    }
}

impl<A: Delay, F> Delay for OneToOneInspect<A, F> {
    fn delay(&self) -> usize {
        self.0.delay()
    }
}

impl<A: Reset, F> Reset for OneToOneInspect<A, F> {
    fn reset(&mut self) {
        self.0.reset();
    }
}

pub struct OneToOneChain<A, B>(A, B);

impl<T, A, B> OneToOne<T> for OneToOneChain<A, B>
where
    A: OneToOne<T>,
    B: OneToOne<A::Output>,
{
    type Output = B::Output;

    fn filter(&mut self, sample: T) -> Self::Output {
        self.1.filter(self.0.filter(sample))
    }
}

impl<A: Delay, B: Delay> Delay for OneToOneChain<A, B> {
    fn delay(&self) -> usize {
        self.0.delay() + self.1.delay()
    }
}

impl<A: Reset, B: Reset> Reset for OneToOneChain<A, B> {
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
    const ZERO: Self;
    const ONE: Self;
    const HALF: Self;
    const TWO: Self;

    fn from_f64(v: f64) -> Self;
}

impl Real for f64 {
    const NAN: Self = Self::ZERO / Self::ZERO;
    const TAU: Self = 6.28318530717958647692528676655900577_f64;
    const PI: Self = Self::TAU / 2.0;
    const ZERO: Self = 0.0f64;
    const ONE: Self = 1.0f64;
    const HALF: Self = 0.5f64;
    const TWO: Self = 2.0f64;

    fn from_f64(v: f64) -> Self {
        v as Self
    }
}

impl Real for f32 {
    const NAN: Self = Self::ZERO / Self::ZERO;
    const TAU: Self = 6.28318530717958647692528676655900577_f32;
    const PI: Self = Self::TAU / 2.0;
    const ZERO: Self = 0.0f32;
    const ONE: Self = 1.0f32;
    const HALF: Self = 0.5f32;
    const TWO: Self = 2.0f32;

    fn from_f64(v: f64) -> Self {
        v as Self
    }
}
