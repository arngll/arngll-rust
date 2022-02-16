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
