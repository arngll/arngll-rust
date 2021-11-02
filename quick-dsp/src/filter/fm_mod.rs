use super::*;
use std::marker::PhantomData;

pub struct Duplicator<T: OneToOne<In>, In> {
    pipeline: T,
    leftover: f32,
    phantom: PhantomData<In>,
}

impl<T: OneToOne<In>, In: Clone> Duplicator<T, In> {
    pub fn new(pipeline: T) -> Self {
        Duplicator {
            pipeline,
            leftover: 0.0,
            phantom: PhantomData::default(),
        }
    }

    pub fn wrap_iterator<'a, I: Iterator<Item = In>>(
        self,
        iter: I,
        duration: f32,
    ) -> impl Iterator<Item = T::Output> + 'a
    where
        I: Iterator<Item = In> + 'a,
        In: 'a,
        T: 'a,
    {
        struct DupeIter<I, T, In>
        where
            I: Iterator<Item = In>,
            T: OneToOne<In>,
            In: Clone,
        {
            inner: I,
            duplicator: Duplicator<T, In>,
            duration: f32,
        }
        impl<I, T, In> Iterator for DupeIter<I, T, In>
        where
            I: Iterator<Item = In>,
            T: OneToOne<In>,
            In: Clone,
        {
            type Item = std::vec::IntoIter<T::Output>;
            fn next(&mut self) -> Option<Self::Item> {
                if let Some(x) = self.inner.next() {
                    Some(self.duplicator.push_fill(x, self.duration).into_iter())
                } else {
                    None
                }
            }
        }

        DupeIter {
            inner: iter,
            duplicator: self,
            duration,
        }
        .flatten()
    }

    pub fn push_fill(&mut self, v: In, duration: f32) -> Vec<T::Output> {
        let mut ret = vec![];

        self.leftover += duration;
        while self.leftover > 1.0 {
            self.leftover -= 1.0;
            ret.push(self.pipeline.filter(v.clone()));
        }

        ret
    }
}

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

impl<T: Real> OneToOne<T> for FmMod<T> {
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
