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

/// Carrier is assumed to be 0.25 (freq/4)
///
/// Output is (angle, magnitude_squared)
#[derive(Clone, Debug)]
pub struct Discriminator<T, F> {
    current_step: u8,
    v_i: T,
    v_q: T,
    last: T,
    last_angle: T,
    filter_i: F,
    filter_q: F,
    filter_out: F,
}

impl<T, F: Delay> Delay for Discriminator<T, F> {
    fn delay(&self) -> usize {
        self.filter_out.delay() + (self.filter_i.delay() + self.filter_q.delay()) / 2
    }
}

impl<T: Real, F> Discriminator<T, F> {
    pub fn digital_default() -> Discriminator<T, FilterFir<T>> {
        Self::new(10, 0.10, 15, 0.10)
    }

    pub fn analog_default() -> Discriminator<T, FilterFir<T>> {
        Self::new(21, 0.25, 9, 0.25)
    }

    pub fn new(
        iq_filter_poles: usize,
        iq_filter_cutoff: f64,
        out_filter_poles: usize,
        out_filter_cutoff: f64,
    ) -> Discriminator<T, FilterFir<T>> {
        let window = Window::Blackman;
        Discriminator {
            current_step: 0,
            v_i: T::ZERO,
            v_q: T::ZERO,
            last: T::ZERO,
            last_angle: T::ZERO,
            filter_i: FilterFir::<T>::low_pass(iq_filter_poles, iq_filter_cutoff, window),
            filter_q: FilterFir::<T>::low_pass(iq_filter_poles, iq_filter_cutoff, window),
            filter_out: FilterFir::<T>::low_pass(out_filter_poles, out_filter_cutoff, window),
        }
    }
}

impl<T, F: OneToOne<T, Output = T>> OneToOne<T> for Discriminator<T, F>
where
    T: Real,
{
    type Output = (T, T); // (angle, magnitude_squared)

    fn filter(&mut self, sample: T) -> Self::Output {
        if !sample.is_finite() {
            return (T::NAN, T::NAN);
        }

        let (v_i, v_q) = match self.current_step {
            0 => (sample, T::ZERO),
            1 => (T::ZERO, sample),
            2 => (-sample, T::ZERO),
            _ => (T::ZERO, -sample),
        };
        self.current_step = (self.current_step + 1) & 3;

        let v_i = self.filter_i.filter(v_i) * T::from_f64(2.0);
        let v_q = self.filter_q.filter(v_q) * T::from_f64(2.0);

        let carrier = T::from_f64(0.25f64);
        let inv_carrier = T::ONE / carrier;

        let neg_recip_tau = -inv_carrier / T::TAU;

        let mag_sq = v_i * v_i + v_q * v_q;

        self.last = if mag_sq.eq(&T::ZERO) {
            self.last
        } else if false {
            // high-quality
            let ret = -self.last_angle;
            self.last_angle = v_q.atan2(v_i);
            let ret = ret + self.last_angle;
            if ret > T::PI {
                ret - T::TAU
            } else if ret < -T::PI {
                ret + T::TAU
            } else {
                ret
            }
        } else {
            // low-quality
            let ret = (v_q * self.v_i - v_i * self.v_q) / mag_sq;
            self.v_i = v_i;
            self.v_q = v_q;
            ret
        };

        (
            self.filter_out
                .filter((self.last * neg_recip_tau + T::ONE) * carrier),
            mag_sq,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discriminator_f32() {
        let mut phase_err;
        let mut amplitude_err;
        let amplitude = 1.0;

        let disc = FmMod::<f32>::new(amplitude);
        let mut disc = disc.chain(Discriminator::<f32, ()>::analog_default());

        for f in &[0.1, 0.15, 0.2, 0.25, 0.3, 0.35, 0.4] {
            // Flush the filters.
            for _i in 0..disc.delay() {
                let _ = disc.filter(*f);
            }

            // Value should settle 3 samples after the delay is flushed.
            for _i in 0..3 {
                let _ = disc.filter(*f);
            }

            phase_err = 0.0;
            amplitude_err = 0.0;

            for _i in 0..100 {
                let result = disc.filter(*f);
                let perr = (result.0 - f).abs();
                let aerr = (result.1 - amplitude).abs();
                phase_err += perr;
                amplitude_err += aerr;
                assert!(perr < 0.05, "bad phase error {}", perr);
                assert!(aerr < 0.02, "bad amplitude error {}", aerr);
            }

            phase_err /= 100.0;
            amplitude_err /= 100.0;
            println!(
                "discriminator_f32: [{}] phase_err:{} amp_err:{}",
                f, phase_err, amplitude_err
            );
            assert!(phase_err < 0.03, "bad average phase error {}", phase_err);
            assert!(
                amplitude_err < 0.01,
                "bad average amplitude error {}",
                amplitude_err
            );
        }
    }

    #[test]
    fn discriminator_f64() {
        let mut phase_err;
        let mut amplitude_err;
        let amplitude = 1.0;

        let disc = FmMod::<f64>::new(amplitude);
        let mut disc = disc.chain(Discriminator::<f64, ()>::analog_default());

        for f in &[0.1, 0.15, 0.2, 0.25, 0.3, 0.35, 0.4] {
            // Flush the filters.
            for _i in 0..disc.delay() {
                let _ = disc.filter(*f);
            }

            // Value should settle 3 samples after the delay is flushed.
            for _i in 0..3 {
                let _ = disc.filter(*f);
            }

            phase_err = 0.0;
            amplitude_err = 0.0;

            for _i in 0..100 {
                let result = disc.filter(*f);
                let perr = (result.0 - f).abs();
                let aerr = (result.1 - amplitude).abs();
                phase_err += perr;
                amplitude_err += aerr;
                assert!(perr < 0.05, "bad phase error {}", perr);
                assert!(aerr < 0.02);
            }

            phase_err /= 100.0;
            amplitude_err /= 100.0;
            println!(
                "discriminator_f64: [{}] phase_err:{} amp_err:{}",
                f, phase_err, amplitude_err
            );
            assert!(phase_err < 0.03, "bad average phase error {}", phase_err);
            assert!(
                amplitude_err < 0.01,
                "bad average amplitude error {}",
                amplitude_err
            );
        }
    }
}
