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

fn calc_chebyshev<T: Real>(
    poles: usize,
    p: usize,
    cutoff1: T,
    _cutoff2: T,
    ripple: T,
    filter_type: FilterType,
) -> ([T; 3], [T; 3]) {
    let theta_p = T::ONE;

    // Calculate the pole location on the unit circle.
    //rp = -cos(M_PI/(poles*2.0) + (p-1.0)*M_PI/poles);
    //ip = sin(M_PI/(poles*2.0) + (p-1.0)*M_PI/poles);
    let mut rp = -(T::PI / T::from_usize(poles * 2)
        + T::from_usize(p - 1) * T::PI / T::from_usize(poles))
    .cos();
    let mut ip = (T::PI / T::from_usize(poles * 2)
        + T::from_usize(p - 1) * T::PI / T::from_usize(poles))
    .sin();

    let mut x = [T::ZERO, T::ZERO, T::ZERO];
    let mut y = [-T::ONE, T::ZERO, T::ZERO];

    if ripple > T::from_f64(0.0001) {
        // Warp from a circle into an elipse.

        let unripple = (T::from_f64(100.0) / (T::from_f64(100.0) - ripple)).powi(2);
        let es = (unripple - T::ONE).sqrt();
        let one_over_poles = T::ONE / T::from_usize(poles);
        let vx = one_over_poles * ((T::ONE / es) + (T::ONE / (es * es) + T::ONE).sqrt()).ln();
        let mut kx = one_over_poles * ((T::ONE / es) + (T::ONE / (es * es) - T::ONE).sqrt()).ln();
        kx = (T::E.powf(kx) + T::E.powf(-kx)) / T::TWO;

        rp *= ((T::E.powf(vx) - T::E.powf(-vx)) / T::TWO) / kx;
        ip *= ((T::E.powf(vx) + T::E.powf(-vx)) / T::TWO) / kx;
    }

    {
        // S-domain to Z-domain transformation.
        let t = T::from_f64(2.0f64 * (1.0f64 / 2.0f64).tan());
        let m = rp * rp + ip * ip;
        let d = T::from_usize(4) - T::from_usize(4) * rp * t + m * t * t;

        x[0] = t * t / d;
        x[1] = T::TWO * x[0];
        x[2] = x[0];

        y[1] = (T::from_usize(8) - T::TWO * m * t * t) / d;
        y[2] = (-T::from_usize(4) - T::from_usize(4) * rp * t - m * t * t) / d;
    }

    if filter_type.is_band() {
        todo!("Band filter not yet finished")

        // LP-to-BP or LP-to-BS transformation
        //let mu_p1 = T::TAU*cutoff1;
        //let mu_p2 = T::TAU*cutoff2;
        //
        // alpha = cos((mu_p2 + mu_p1)/2.0)/cos((mu_p2 - mu_p1)/2.0);
        //
        // if(type == DDDSP_BANDPASS) {
        //     k = tan(theta_p/2.0)/tan((mu_p2 - mu_p1)/2.0);
        // } else {
        //     k = tan(theta_p/2.0)*tan((mu_p2 - mu_p1)/2.0);
        // }
    } else {
        // LP-to-LP or LP-to-HP transformation
        let mu_p = T::TAU * cutoff1;

        y[0] = -T::ONE;

        let alpha = if filter_type.is_high_pass() {
            -((theta_p + mu_p) / T::TWO).cos() / ((theta_p - mu_p) / T::TWO).cos()
        } else {
            ((theta_p - mu_p) / T::TWO).sin() / ((theta_p + mu_p) / T::TWO).sin()
        };

        let d = T::ONE + y[1] * alpha - y[2] * alpha * alpha;

        let mut a = [T::ZERO; 3];
        let mut b = [T::ZERO; 3];

        a[0] = (x[0] - x[1] * alpha + x[2] * alpha * alpha) / d;
        a[1] = (x[1] - T::TWO * x[0] * alpha - T::TWO * x[2] * alpha + x[1] * alpha * alpha) / d;
        a[2] = (x[2] - x[1] * alpha + x[0] * alpha * alpha) / d;

        b[1] = (y[1] - T::TWO * y[0] * alpha - T::TWO * y[2] * alpha + y[1] * alpha * alpha) / d;
        b[2] = (y[2] - y[1] * alpha + y[0] * alpha * alpha) / d;

        if filter_type.is_high_pass() {
            a[1] = -a[1];
            b[1] = -b[1];
        }

        (a, b)
    }
}

fn calc_gain_low<T: Real>(a: &[T], b: &[T]) -> T {
    let mut sa = T::ZERO;
    let mut sb = T::ZERO;
    for (&a, &b) in a.iter().zip(b.iter()) {
        sa += a;
        sb += b;
    }
    return sa / (T::ONE - sb);
}

fn calc_gain_high<T: Real>(a: &[T], b: &[T]) -> T {
    let mut sa = T::ZERO;
    let mut sb = T::ZERO;
    for (i, (&a, &b)) in a.iter().zip(b.iter()).enumerate() {
        let x = T::TWO * (T::from_f64((i & 1) as f64));
        sa += a * (T::ONE - x);
        sb += b * (T::ONE - x);
    }
    return sa / (T::ONE - sb);
}

fn adjust_gain<T: Real>(a: &mut [T], x: T) {
    for a in a.iter_mut() {
        *a *= x;
    }
}

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

impl<T: Real> FilterIirKernel<T> {
    pub fn new(a: Vec<T>, b: Vec<T>, delay: usize) -> Self {
        FilterIirKernel { a, b, delay }
    }

    fn gain_low(&self) -> T {
        calc_gain_low(&self.a, &self.b)
    }

    fn gain_high(&self) -> T {
        calc_gain_high(&self.a, &self.b)
    }

    fn adjust_gain(&mut self, gain: T) {
        adjust_gain(&mut self.a, gain)
    }

    pub fn len(&self) -> usize {
        self.a.len()
    }

    pub fn poles(&self) -> usize {
        self.a.len() - 1
    }
}

impl<T: Real> FilterIirKernel<T> {
    fn chebyshev(
        poles: usize,
        cutoff1: T,
        _cutoff2: T,
        ripple: T,
        filter_type: FilterType,
    ) -> Self {
        let mut ret = Self {
            a: vec![],
            b: vec![],
            delay: poles / 2,
        };
        ret.a.resize(poles + 1, T::ZERO);
        ret.b.resize(poles + 1, T::ZERO);
        ret.a[0] = T::ONE;
        ret.b[0] = T::ONE;

        if filter_type.is_band_pass() {
            todo!();
        } else {
            for p in 1..=(poles / 2) {
                let mut ta = ret.a.clone();
                let mut tb = ret.b.clone();
                ta.insert(0, T::ZERO);
                ta.insert(0, T::ZERO);
                tb.insert(0, T::ZERO);
                tb.insert(0, T::ZERO);

                let (a_x, b_x) = calc_chebyshev(poles, p, cutoff1, cutoff1, ripple, filter_type);
                for (i, (a, b)) in ret.a.iter_mut().zip(ret.b.iter_mut()).enumerate() {
                    *a = a_x[0] * ta[i + 2] + a_x[1] * ta[i + 1] + a_x[2] * ta[i + 0];
                    *b = tb[i + 2] - b_x[1] * tb[i + 1] - b_x[2] * tb[i + 0];
                }
            }
        }

        ret.b[0] = T::ZERO;

        // Finish combining coefficients
        for b in ret.b.iter_mut() {
            *b = -*b;
        }

        // Normalize the gain on the coefficients.
        match filter_type {
            FilterType::LowPass => ret.adjust_gain(T::ONE / ret.gain_low()),
            FilterType::HighPass => ret.adjust_gain(T::ONE / ret.gain_high()),
            _ => (),
        }

        return ret;
    }

    pub fn low_pass(poles: usize, cutoff: T, ripple: T) -> Self {
        Self::chebyshev(poles, cutoff, T::ZERO, ripple, FilterType::LowPass)
    }

    pub fn high_pass(poles: usize, cutoff: T, ripple: T) -> Self {
        Self::chebyshev(poles, cutoff, T::ZERO, ripple, FilterType::HighPass)
    }

    pub fn band_pass(poles: usize, lcutoff: T, hcutoff: T, ripple: T) -> Self {
        Self::chebyshev(poles, lcutoff, hcutoff, ripple, FilterType::BandPass)
    }

    pub fn band_stop(poles: usize, lcutoff: T, hcutoff: T, ripple: T) -> Self {
        Self::chebyshev(poles, lcutoff, hcutoff, ripple, FilterType::BandStop)
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

    pub fn low_pass(poles: usize, cutoff: T, ripple: T) -> Self {
        FilterIirKernel::low_pass(poles, cutoff, ripple).into()
    }

    pub fn high_pass(poles: usize, cutoff: T, ripple: T) -> Self {
        FilterIirKernel::high_pass(poles, cutoff, ripple).into()
    }

    pub fn band_pass(poles: usize, lcutoff: T, hcutoff: T, ripple: T) -> Self {
        FilterIirKernel::band_pass(poles, lcutoff, hcutoff, ripple).into()
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
        self.y.push(T::ZERO);

        let output = self
            .x
            .iter()
            .zip(self.kernel.a.iter())
            .map(|(x, a)| x.mul(*a))
            .sum::<T>()
            + self
                .y
                .iter()
                .skip(1)
                .zip(self.kernel.b.iter().skip(1))
                .map(|(y, b)| y.mul(*b))
                .sum::<T>();

        if output.is_finite() {
            *self.y.iter_mut().next().unwrap() = output;
        }

        output
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
    fn filter_iir_dataset_test1() {
        let (a, b) = calc_chebyshev(4, 1, 0.1f64, 0.0f64, 0.0f64, FilterType::LowPass);
        println!("filter_iir_dataset_test1: a={:?}", a);
        println!("filter_iir_dataset_test1: b={:?}", b);
        assert!((a[0] - 0.061885).abs() < 0.00001);
        assert!((a[1] - 0.123770).abs() < 0.00001);
        assert!((a[2] - 0.061885).abs() < 0.00001);
        assert!((b[1] - 1.048600).abs() < 0.00001);
        assert!((b[2] + 0.296140).abs() < 0.00001);
    }

    #[test]
    fn filter_iir_dataset_test2() {
        let (a, b) = calc_chebyshev(4, 2, 0.1f64, 0.0f64, 10.0f64, FilterType::HighPass);
        println!("filter_iir_dataset_test2: a={:?}", a);
        println!("filter_iir_dataset_test2: b={:?}", b);
        assert!((a[0] - 0.922919).abs() < 0.00001);
        assert!((a[1] + 1.845840).abs() < 0.00001);
        assert!((a[2] - 0.922919).abs() < 0.00001);
        assert!((b[1] - 1.446913).abs() < 0.00001);
        assert!((b[2] + 0.836653).abs() < 0.00001);
    }

    #[test]
    fn filter_iir_dataset_test3() {
        let filter = FilterIir::low_pass(4, 0.25f64, 0.5f64);
        println!("filter_iir_dataset_test3: {:#?}", filter);

        assert!((filter.kernel.a[0] - 0.07015301).abs() < 0.00001);
        assert!((filter.kernel.a[1] - 0.2806120).abs() < 0.00001);
        assert!((filter.kernel.a[2] - 0.4209180).abs() < 0.00001);
        assert!((filter.kernel.a[3] - 0.2806120).abs() < 0.00001);
        assert!((filter.kernel.a[4] - 0.07015301).abs() < 0.00001);

        assert!((filter.kernel.b[1] - 0.4541481).abs() < 0.00001);
        assert!((filter.kernel.b[2] + 0.7417536).abs() < 0.00001);
        assert!((filter.kernel.b[3] - 0.2361222).abs() < 0.00001);
        assert!((filter.kernel.b[4] + 0.07096476).abs() < 0.00001);
    }

    #[test]
    fn filter_iir_low_pass_histogram_2_pole() {
        let kernel = FilterIirKernel::low_pass(2, 0.25f64, 0.5f64);

        let fresponse = (0..50)
            .into_iter()
            .map(|i| calc_gain(kernel.clone().into_filter(), (i as f64) / 100f64))
            .collect::<Vec<_>>();

        let histogram = rag::plot(
            fresponse,
            rag_config().with_caption("filter_iir_low_pass_histogram_2_pole".to_string()),
        );

        println!("{}", histogram);
    }

    #[test]
    fn filter_iir_low_pass_histogram_4_pole() {
        let kernel = FilterIirKernel::low_pass(4, 0.25f64, 0.5f64);

        let fresponse = (0..50)
            .into_iter()
            .map(|i| calc_gain(kernel.clone().into_filter(), (i as f64) / 100f64))
            .collect::<Vec<_>>();

        let histogram = rag::plot(
            fresponse,
            rag_config().with_caption("filter_iir_low_pass_histogram_4_pole".to_string()),
        );

        println!("{}", histogram);
    }

    #[test]
    fn filter_iir_low_pass_histogram_6_pole() {
        let kernel = FilterIirKernel::low_pass(6, 0.25f64, 0.5f64);

        let fresponse = (0..50)
            .into_iter()
            .map(|i| calc_gain(kernel.clone().into_filter(), (i as f64) / 100f64))
            .collect::<Vec<_>>();

        let histogram = rag::plot(
            fresponse,
            rag_config().with_caption("filter_iir_low_pass_histogram_4_pole".to_string()),
        );

        println!("{}", histogram);
    }

    #[test]
    fn filter_iir_high_pass_histogram_2_pole() {
        let kernel = FilterIirKernel::high_pass(2, 0.25f64, 0.5f64);

        let fresponse = (1..=50)
            .into_iter()
            .map(|i| calc_gain(kernel.clone().into_filter(), (i as f64) / 100f64))
            .collect::<Vec<_>>();

        let histogram = rag::plot(
            fresponse,
            rag_config().with_caption("filter_iir_high_pass_histogram_2_pole".to_string()),
        );

        println!("{}", histogram);
    }

    #[test]
    fn filter_iir_high_pass_histogram_4_pole() {
        let kernel = FilterIirKernel::high_pass(4, 0.25f64, 0.5f64);

        let fresponse = (1..=50)
            .into_iter()
            .map(|i| calc_gain(kernel.clone().into_filter(), (i as f64) / 100f64))
            .collect::<Vec<_>>();

        let histogram = rag::plot(
            fresponse,
            rag_config().with_caption("filter_iir_high_pass_histogram_4_pole".to_string()),
        );

        println!("{}", histogram);
    }

    #[test]
    fn filter_iir_high_pass_histogram_6_pole() {
        let kernel = FilterIirKernel::high_pass(6, 0.25f64, 0.5f64);

        let fresponse = (1..=50)
            .into_iter()
            .map(|i| calc_gain(kernel.clone().into_filter(), (i as f64) / 100f64))
            .collect::<Vec<_>>();

        let histogram = rag::plot(
            fresponse,
            rag_config().with_caption("filter_iir_high_pass_histogram_6_pole".to_string()),
        );

        println!("{}", histogram);
    }

    #[test]
    fn filter_iir_low_pass_performance_2pole() {
        let gain_h = calc_gain(FilterIir::low_pass(2, 0.25f64, 0.5f64), 0.45f64);
        println!("filter_iir_low_pass: 2-pole gain_h: {:.2}dB", gain_h);
        assert!(gain_h < -15.0);

        let gain_l = calc_gain(FilterIir::low_pass(2, 0.25f64, 0.5f64), 0.05f64);
        println!("filter_iir_low_pass: 2-pole gain_l: {:.2}dB", gain_l);
        assert!(gain_l > -0.5);
        assert!(gain_l < 0.01);
    }

    #[test]
    fn filter_iir_low_pass_performance_4pole() {
        let gain_h = calc_gain(FilterIir::low_pass(4, 0.25f64, 0.5f64), 0.45f64);
        println!("filter_iir_low_pass: 4-pole gain_h: {:.2}dB", gain_h);
        assert!(gain_h < -35.0);

        let gain_l = calc_gain(FilterIir::low_pass(4, 0.25f64, 0.5f64), 0.05f64);
        println!("filter_iir_low_pass: 4-pole gain_l: {:.2}dB", gain_l);
        assert!(gain_l > -0.5);
        assert!(gain_l < 0.01);
    }

    #[test]
    fn filter_iir_low_pass_performance_6pole() {
        let gain_h = calc_gain(FilterIir::low_pass(6, 0.25f64, 0.5f64), 0.45f64);
        println!("filter_iir_low_pass: 6-pole gain_h: {:.2}dB", gain_h);
        assert!(gain_h < -55.0);

        let gain_l = calc_gain(FilterIir::low_pass(6, 0.25f64, 0.5f64), 0.05f64);
        println!("filter_iir_low_pass: 6-pole gain_l: {:.2}dB", gain_l);
        assert!(gain_l > -0.5);
        assert!(gain_l < 0.01);
    }

    #[test]
    //    #[ignore] // Currently failing.
    fn filter_iir_high_pass_performance_2pole() {
        let gain_l = calc_gain(FilterIir::high_pass(2, 0.25f64, 0.5f64), 0.05f64);
        println!("filter_iir_high_pass: 2-pole gain_l: {:.2}dB", gain_l);
        assert!(gain_l < -16.0);

        let gain_h = calc_gain(FilterIir::high_pass(2, 0.25f64, 0.5f64), 0.5f64);
        println!("filter_iir_high_pass: 2-pole gain_h: {:.2}dB", gain_h);
        assert!(gain_h > -0.5);
        assert!(gain_h < 0.01);
    }

    #[test]
    //    #[ignore] // Currently failing.
    fn filter_iir_high_pass_performance_4pole() {
        let gain_l = calc_gain(FilterIir::high_pass(4, 0.25f64, 0.5f64), 0.05f64);
        println!("filter_iir_high_pass: 4-pole gain_l: {:.2}dB", gain_l);
        assert!(gain_l < -35.0);

        let gain_h = calc_gain(FilterIir::high_pass(4, 0.25f64, 0.5f64), 0.5f64);
        println!("filter_iir_high_pass: 4-pole gain_h: {:.2}dB", gain_h);
        assert!(gain_h > -0.5);
        assert!(gain_h < 0.01);
    }

    #[test]
    //    #[ignore] // Currently failing.
    fn filter_iir_high_pass_performance_6pole() {
        let gain_l = calc_gain(FilterIir::high_pass(6, 0.25f64, 0.5f64), 0.05f64);
        println!("filter_iir_high_pass: 6-pole gain_l: {:.2}dB", gain_l);
        assert!(gain_l < -52.0);

        let gain_h = calc_gain(FilterIir::high_pass(6, 0.25f64, 0.5f64), 0.5f64);
        println!("filter_iir_high_pass: 6-pole gain_h: {:.2}dB", gain_h);
        assert!(gain_h > -0.5);
        assert!(gain_h < 0.01);
    }
}
