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

#[derive(Debug, Clone, Default)]
pub struct QamDiscriminatorFast<T> {
    v_i: T,
    v_q: T,
    last: T,
}

impl<T> Delay for QamDiscriminatorFast<T> {
    fn delay(&self) -> usize {
        0
    }
}

impl<T:Real> Filter<(T, T)> for QamDiscriminatorFast<T>
{
    type Output = (T, T); // (angle, magnitude_squared)

    fn filter(&mut self, sample: (T,T)) -> Self::Output {
        if !sample.0.is_finite() || !sample.1.is_finite() {
            return (T::NAN, T::NAN);
        }

        let (v_i, v_q) = (sample.0 * T::TWO, sample.1 * T::TWO);

        let mag_sq = v_i * v_i + v_q * v_q;

        self.last = if mag_sq.eq(&T::ZERO) {
            // If the magnitude is zero then simply repeat the last value.
            self.last
        } else {
            let ret = (v_q * self.v_i - v_i * self.v_q) / mag_sq;
            self.v_i = v_i;
            self.v_q = v_q;
            ret
        };

        let carrier: T = T::FORTH;
        let inv_carrier: T = T::ONE / carrier;
        let neg_recip_tau:T = -inv_carrier / T::TAU;

        (
            (self.last * neg_recip_tau + T::ONE) * carrier,
            mag_sq,
        )
    }
}



#[derive(Debug, Clone, Default)]
pub struct QamDiscriminatorAccurate<T> {
    last_angle: T,
    last: T,
}

impl<T> Delay for QamDiscriminatorAccurate<T> {
    fn delay(&self) -> usize {
        0
    }
}

impl<T:Real> Filter<(T, T)> for QamDiscriminatorAccurate<T>
{
    type Output = (T, T); // (angle, magnitude_squared)

    fn filter(&mut self, sample: (T,T)) -> Self::Output {
        if !sample.0.is_finite() || !sample.1.is_finite() {
            return (T::NAN, T::NAN);
        }

        let (v_i, v_q) = (sample.0 * T::TWO, sample.1 * T::TWO);

        let mag_sq = v_i * v_i + v_q * v_q;

        self.last = if mag_sq.eq(&T::ZERO) {
            // If the magnitude is zero then simply repeat the last value.
            self.last
        } else {
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
        };

        let carrier: T = T::FORTH;
        let inv_carrier: T = T::ONE / carrier;
        let neg_recip_tau:T = -inv_carrier / T::TAU;

        (
            (self.last * neg_recip_tau + T::ONE) * carrier,
            mag_sq,
        )
    }
}

/// Carrier is assumed to be 0.25 (freq/4)
///
/// Output is (angle, magnitude_squared)
#[derive(Clone, Debug)]
pub struct Discriminator<T, FIQ=(), FOUT=()> {
    qam: QamSplitFixed<T,FIQ>,
    disc: QamDiscriminatorAccurate<T>,
    filter_out: FOUT,
}

impl<T, FIQ: Delay, FOUT: Delay> Delay for Discriminator<T, FIQ, FOUT> {
    fn delay(&self) -> usize {
        self.filter_out.delay() + self.qam.delay() + self.disc.delay()
    }
}

impl<T: Real, FIQ, FOUT> Discriminator<T, FIQ, FOUT> {
    pub fn digital_default() -> Discriminator<T, FilterFir<T>, FilterFir<T>> {
        Self::new(
            FilterFirKernel::<T>::low_pass(15, 0.1, Window::Blackman).into_filter(),
            FilterFirKernel::<T>::low_pass(15, 0.1, Window::Blackman).into_filter(),
        )
    }

    pub fn analog_default() -> Discriminator<T, FilterFir<T>, FilterFir<T>> {
        Self::new(
            FilterFirKernel::<T>::low_pass(21, 0.25, Window::Blackman).into_filter(),
            FilterFirKernel::<T>::low_pass(9, 0.25, Window::Blackman).into_filter(),
        )
    }

    pub fn new<KIQ: Filter<T> + Clone, KOUT: Filter<T>>(
        kiq: KIQ,
        kout: KOUT,
    ) -> Discriminator<T, KIQ, KOUT> {
        Discriminator {
            qam: QamSplitFixed::<_>::new(kiq),
            disc: Default::default(),
            filter_out: kout,
        }
    }
}

impl<T, FIQ: Filter<T, Output = T>, FOUT: Filter<T, Output = T>> Filter<T> for Discriminator<T, FIQ, FOUT>
where
    T: Real,
{
    type Output = (T, T); // (angle, magnitude_squared)

    fn filter(&mut self, sample: T) -> Self::Output {
        if !sample.is_finite() {
            return (T::NAN, T::NAN);
        }

        let sample = self.disc.filter(self.qam.filter(sample));

        (self.filter_out.filter(sample.0), sample.1)
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
        let mut disc = disc.chain(Discriminator::<_>::analog_default());

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
        let mut disc = disc.chain(Discriminator::<_>::analog_default());

        for f in &[0.1, 0.15, 0.2, 0.25, 0.3, 0.35, 0.4] {
            // Flush the filters.
            for _i in 0..disc.delay() {
                let _ = disc.filter(*f);
            }

            // Value should settle 10 samples after the delay is flushed.
            for _i in 0..10 {
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
