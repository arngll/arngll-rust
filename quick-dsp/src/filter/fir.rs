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

#[derive(Clone, Debug)]
pub struct FilterFirKernel<T> {
    a: Vec<T>,
    delay: usize,
}

impl<T: Real> Kernel for FilterFirKernel<T> {
    type Filter = FilterFir<T>;

    fn into_filter(self) -> Self::Filter {
        FilterFir::from_kernel(self)
    }
}

impl<T> FilterFirKernel<T> {
    pub fn new(a: Vec<T>, delay: usize) -> Self {
        FilterFirKernel { a, delay }
    }

    pub fn len(&self) -> usize {
        self.a.len()
    }

    pub fn poles(&self) -> usize {
        self.a.len() - 1
    }
}

impl<T> FilterFirKernel<T>
where
    T: Real,
{
    pub fn low_pass(poles: usize, cutoff: f64, window: Window) -> Self {
        let taps = poles + 1;
        let mut ret = Vec::with_capacity(taps);
        let mut sum = 0.0;

        for i in 0..taps {
            let n: i32 = if (taps & 1) != 0 {
                i32::try_from(i).unwrap() - i32::try_from(poles).unwrap() / 2
            } else {
                i32::try_from(i).unwrap() - i32::try_from(taps).unwrap() / 2
            };
            let tn = f64::from(n);
            let ti = f64::from(u32::try_from(i).unwrap());
            let ttaps = f64::from(u32::try_from(taps).unwrap());
            let tau = std::f64::consts::PI * 2.0;

            let val = if n != 0 {
                f64::sin(cutoff * tau * tn) / tn
            } else {
                cutoff * tau
            } * match window {
                Window::Hanning => 0.5 - 0.5 * f64::cos((tau * ti) / ttaps),
                Window::Hamming => 0.54 - 0.46 * f64::cos((tau * ti) / ttaps),
                Window::Blackman => {
                    0.42 - 0.5 * f64::cos((tau * ti) / ttaps)
                        + 0.08 * f64::cos((2.0 * tau * ti) / ttaps)
                }
                Window::Rectangular => 1.0,
            };

            sum += val;

            ret.push(T::from_f64(val));
        }

        let recip_sum = T::from_f64(1.0 / sum);

        ret.iter_mut().for_each(|x: &mut T| x.mul_assign(recip_sum));

        Self::new(ret, poles / 2)
    }

    pub fn high_pass(poles: usize, cutoff: f64, window: Window) -> Self {
        let mut ret = Self::low_pass(poles, cutoff, window);
        let n = poles + 1;
        (0..n).into_iter().zip(ret.a.iter_mut()).for_each(|(i, x)| {
            let y = f64::from((i == n / 2) as i32);
            *x = T::from_f64(y) - *x;
        });
        ret
    }

    pub fn band_pass(poles: usize, lcutoff: f64, hcutoff: f64, window: Window) -> Self {
        let mut a = Self::low_pass(poles, lcutoff, window);
        let b = Self::low_pass(poles, hcutoff, window);

        a.a.iter_mut()
            .zip(b.a.iter())
            .for_each(|(a, b)| *a = *b - *a);

        a
    }
}

impl<T> From<FilterFirKernel<T>> for FilterFir<T>
where
    T: Real,
{
    fn from(kernel: FilterFirKernel<T>) -> Self {
        FilterFir::from_kernel(kernel)
    }
}

#[derive(Clone, Debug)]
pub struct FilterFir<T> {
    kernel: FilterFirKernel<T>,
    x: CircularQueue<T>,
}

impl<T> Delay for FilterFir<T> {
    fn delay(&self) -> usize {
        self.kernel.delay
    }
}

impl<T: Real> FilterFir<T> {
    pub fn from_kernel(kernel: FilterFirKernel<T>) -> Self {
        FilterFir {
            x: CircularQueue::with_capacity(kernel.len()),
            kernel,
        }
    }

    pub fn low_pass(poles: usize, cutoff: f64, window: Window) -> Self {
        FilterFirKernel::low_pass(poles, cutoff, window).into()
    }

    pub fn high_pass(poles: usize, cutoff: f64, window: Window) -> Self {
        FilterFirKernel::high_pass(poles, cutoff, window).into()
    }

    pub fn band_pass(poles: usize, lcutoff: f64, hcutoff: f64, window: Window) -> Self {
        FilterFirKernel::band_pass(poles, lcutoff, hcutoff, window).into()
    }
}

impl<T: Real> OneToOne<T> for FilterFir<T> {
    type Output = T;

    fn filter(&mut self, sample: T) -> T {
        self.x.push(sample);

        return self
            .x
            .iter()
            .zip(self.kernel.a.iter())
            .map(|(x, a)| x.mul(*a))
            .sum();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rasciigraph as rag;

    fn rag_config() -> rag::Config {
        rag::Config::default()
            .with_offset(10)
            .with_height(10)
            .with_width(70)
    }

    #[test]
    fn filter_fir_low_pass_histogram_12_pole() {
        let kernel = FilterFirKernel::low_pass(12, 0.25f64, Window::Blackman);

        let fresponse = (0..=50)
            .into_iter()
            .map(|i| calc_gain(kernel.clone().into_filter(), (i as f64) / 100f64))
            .collect::<Vec<_>>();

        let histogram = rag::plot(
            fresponse,
            rag_config().with_caption("filter_fir_low_pass_histogram_12_pole".to_string()),
        );

        println!("{}", histogram);
    }

    #[test]
    fn filter_fir_low_pass_histogram_24_pole() {
        let kernel = FilterFirKernel::low_pass(24, 0.25f64, Window::Blackman);

        let fresponse = (0..=50)
            .into_iter()
            .map(|i| calc_gain(kernel.clone().into_filter(), (i as f64) / 100f64))
            .collect::<Vec<_>>();

        let histogram = rag::plot(
            fresponse,
            rag_config().with_caption("filter_fir_low_pass_histogram_24_pole".to_string()),
        );

        println!("{}", histogram);
    }

    #[test]
    fn filter_fir_low_pass_histogram_100_pole() {
        let kernel = FilterFirKernel::low_pass(100, 0.25f64, Window::Blackman);

        let fresponse = (0..=50)
            .into_iter()
            .map(|i| calc_gain(kernel.clone().into_filter(), (i as f64) / 100f64))
            .collect::<Vec<_>>();

        let histogram = rag::plot(
            fresponse,
            rag_config().with_caption("filter_fir_low_pass_histogram_100_pole".to_string()),
        );

        println!("{}", histogram);
    }

    #[test]
    fn filter_fir_high_pass_histogram_24_pole() {
        let kernel = FilterFirKernel::high_pass(24, 0.25f64, Window::Blackman);

        let fresponse = (1..=50)
            .into_iter()
            .map(|i| calc_gain(kernel.clone().into_filter(), (i as f64) / 100f64))
            .collect::<Vec<_>>();

        let histogram = rag::plot(
            fresponse,
            rag_config().with_caption("filter_fir_high_pass_histogram_24_pole".to_string()),
        );

        println!("{}", histogram);
    }

    #[test]
    fn filter_fir_band_pass_histogram_24_pole() {
        let kernel = FilterFirKernel::band_pass(24, 0.1666f64, 0.33333f64, Window::Blackman);

        let fresponse = (1..=50)
            .into_iter()
            .map(|i| calc_gain(kernel.clone().into_filter(), (i as f64) / 100f64))
            .collect::<Vec<_>>();

        let histogram = rag::plot(
            fresponse,
            rag_config().with_caption("filter_fir_band_pass_histogram_24_pole".to_string()),
        );

        println!("{}", histogram);
    }

    #[test]
    fn filter_fir_low_pass_12_pole() {
        let gain_h = calc_gain(FilterFir::low_pass(12, 0.25f64, Window::Blackman), 0.35f64);
        println!("12-pole gain_h: {:.2}dB", gain_h);
        assert!(gain_h < -10.0);

        let gain_l = calc_gain(FilterFir::low_pass(12, 0.25f64, Window::Blackman), 0.15f64);
        println!("12-pole gain_l: {:.2}dB", gain_l);
        assert!(gain_l > -0.5);
        assert!(gain_l < 0.01);
    }

    #[test]
    fn filter_fir_low_pass_24_pole() {
        let gain_h = calc_gain(FilterFir::low_pass(24, 0.25f64, Window::Blackman), 0.35f64);
        println!("24-pole gain_h: {:.2}dB", gain_h);
        assert!(gain_h < -25.0);

        let gain_l = calc_gain(FilterFir::low_pass(24, 0.25f64, Window::Blackman), 0.15f64);
        println!("24-pole gain_l: {:.2}dB", gain_l);
        assert!(gain_l > -0.5);
        assert!(gain_l < 0.01);
    }

    #[test]
    fn filter_fir_low_pass_100_pole() {
        let gain_h = calc_gain(FilterFir::low_pass(100, 0.25f64, Window::Blackman), 0.45f64);
        println!("50-pole gain_h: {:.2}dB", gain_h);
        assert!(gain_h < -57.0);

        let gain_l = calc_gain(FilterFir::low_pass(50, 0.25f64, Window::Blackman), 0.15f64);
        println!("50-pole gain_l: {:.2}dB", gain_l);
        assert!(gain_l > -0.5);
        assert!(gain_l < 0.01);
    }

    #[test]
    fn filter_fir_high_pass_12_pole() {
        let gain_h = calc_gain(FilterFir::high_pass(12, 0.25f64, Window::Blackman), 0.35f64);
        println!("12-pole gain_h: {:.2}dB", gain_h);
        assert!(gain_h > -0.5);
        assert!(gain_h < 0.01);

        let gain_l = calc_gain(FilterFir::high_pass(12, 0.25f64, Window::Blackman), 0.15f64);
        println!("12-pole gain_l: {:.2}dB", gain_l);
        assert!(gain_l < -10.0);
    }

    #[test]
    fn filter_fir_high_pass_24_pole() {
        let gain_h = calc_gain(FilterFir::high_pass(24, 0.25f64, Window::Blackman), 0.35f64);
        println!("24-pole gain_h: {:.2}dB", gain_h);
        assert!(gain_h > -0.5);
        assert!(gain_h < 0.01);

        let gain_l = calc_gain(FilterFir::high_pass(24, 0.25f64, Window::Blackman), 0.15f64);
        println!("24-pole gain_l: {:.2}dB", gain_l);
        assert!(gain_l < -29.0);
    }
}
