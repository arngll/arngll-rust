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

fn calc_chebyshev(
    poles: usize,
    p: usize,
    cutoff1: f64,
    _cutoff2: f64,
    ripple: f64,
    filter_type: FilterType,
) -> ([f64; 3], [f64; 3]) {
    let theta_p = 1.0;

    // Calculate the pole location on the unit circle.
    //rp = -cos(M_PI/(poles*2.0) + (p-1.0)*M_PI/poles);
    //ip = sin(M_PI/(poles*2.0) + (p-1.0)*M_PI/poles);
    let mut rp = -(f64::PI / f64::from_usize(poles * 2)
        + f64::from_usize(p - 1) * f64::PI / f64::from_usize(poles))
    .cos();
    let mut ip = (f64::PI / f64::from_usize(poles * 2)
        + f64::from_usize(p - 1) * f64::PI / f64::from_usize(poles))
    .sin();

    let mut x = [0.0, 0.0, 0.0];
    let mut y = [-1.0, 0.0, 0.0];

    if ripple > 0.0001 {
        // Warp from a circle into an elipse.

        let unripple = (100.0 / (100.0 - ripple)).powi(2);
        let es = (unripple - 1.0).sqrt();
        let one_over_poles = 1.0 / f64::from_usize(poles);
        let vx = one_over_poles * ((1.0 / es) + (1.0 / (es * es) + 1.0).sqrt()).ln();
        let mut kx = one_over_poles * ((1.0 / es) + (1.0 / (es * es) - 1.0).sqrt()).ln();
        kx = (f64::E.powf(kx) + f64::E.powf(-kx)) / 2.0;

        rp *= ((f64::E.powf(vx) - f64::E.powf(-vx)) / 2.0) / kx;
        ip *= ((f64::E.powf(vx) + f64::E.powf(-vx)) / 2.0) / kx;
    }

    {
        // S-domain to Z-domain transformation.
        let t = 2.0f64 * (1.0f64 / 2.0f64).tan();
        let m = rp * rp + ip * ip;
        let d = 4.0 - 4.0 * rp * t + m * t * t;

        x[0] = t * t / d;
        x[1] = 2.0 * x[0];
        x[2] = x[0];

        y[1] = (8.0 - 2.0 * m * t * t) / d;
        y[2] = (-4.0 - 4.0 * rp * t - m * t * t) / d;
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
        let mu_p = f64::TAU * cutoff1;

        y[0] = -1.0;

        let alpha = if filter_type.is_high_pass() {
            -((theta_p + mu_p) / 2.0).cos() / ((theta_p - mu_p) / 2.0).cos()
        } else {
            ((theta_p - mu_p) / 2.0).sin() / ((theta_p + mu_p) / 2.0).sin()
        };

        let d = 1.0 + y[1] * alpha - y[2] * alpha * alpha;

        let mut a = [0.0; 3];
        let mut b = [0.0; 3];

        a[0] = (x[0] - x[1] * alpha + x[2] * alpha * alpha) / d;
        a[1] = (x[1] - 2.0 * x[0] * alpha - 2.0 * x[2] * alpha + x[1] * alpha * alpha) / d;
        a[2] = (x[2] - x[1] * alpha + x[0] * alpha * alpha) / d;

        b[1] = (y[1] - 2.0 * y[0] * alpha - 2.0 * y[2] * alpha + y[1] * alpha * alpha) / d;
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

pub trait FilterIirKernel {
    type Sample: Real;
    const A_TAPS: usize;
    const B_TAPS: usize;

    fn a(&self) -> &[Self::Sample];
    fn b(&self) -> &[Self::Sample];

    fn gain_low(&self) -> Self::Sample {
        calc_gain_low(self.a(), self.b())
    }

    fn gain_high(&self) -> Self::Sample {
        calc_gain_high(self.a(), self.b())
    }
}

#[derive(Clone, Debug)]
pub struct ChebyshevKernel<T, const TAPS: usize> {
    a: [T; TAPS],
    b: [T; TAPS],
    delay: usize,
}

impl<T, const TAPS: usize> Delay for ChebyshevKernel<T, TAPS> {
    fn delay(&self) -> usize {
        self.delay
    }
}

impl<T: Real, const TAPS: usize> FilterIirKernel for ChebyshevKernel<T, TAPS> {
    type Sample = T;
    const A_TAPS: usize = TAPS;
    const B_TAPS: usize = TAPS;

    fn a(&self) -> &[T] {
        self.a.as_slice()
    }

    fn b(&self) -> &[T] {
        self.b.as_slice()
    }
}

impl<T: Real, const TAPS: usize> IntoFilter<T> for ChebyshevKernel<T, TAPS> {
    type Filter = FilterIir<Self>;
    fn into_filter(self) -> Self::Filter {
        FilterIir::from_kernel(self)
    }
}

impl<T: Real, const TAPS: usize> ChebyshevKernel<T, TAPS> {
    fn adjust_gain(&mut self, gain: T) {
        adjust_gain(&mut self.a, gain)
    }
}

impl<T: Real, const TAPS: usize> ChebyshevKernel<T, TAPS> {
    fn chebyshev(cutoff1: f64, cutoff2: f64, ripple: f64, filter_type: FilterType) -> Self {
        let poles = (TAPS - 1) as usize;
        let mut ret = Self {
            a: [T::ZERO; TAPS],
            b: [T::ZERO; TAPS],
            delay: poles / 2,
        };
        ret.a[0] = T::ONE;
        ret.b[0] = T::ONE;

        if filter_type.is_band_pass() {
            todo!();
        } else {
            for p in 1..=(poles / 2) {
                let mut ta = ret.a().to_vec().clone();
                let mut tb = ret.b().to_vec().clone();
                ta.insert(0, T::ZERO);
                ta.insert(0, T::ZERO);
                tb.insert(0, T::ZERO);
                tb.insert(0, T::ZERO);

                let (a_x, b_x) = calc_chebyshev(poles, p, cutoff1, cutoff2, ripple, filter_type);
                let a_x = [
                    T::from_f64(a_x[0]),
                    T::from_f64(a_x[1]),
                    T::from_f64(a_x[2]),
                ];
                let b_x = [
                    T::from_f64(b_x[0]),
                    T::from_f64(b_x[1]),
                    T::from_f64(b_x[2]),
                ];
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

    pub fn low_pass(cutoff: f64, ripple: f64) -> Self {
        Self::chebyshev(cutoff, 0.0, ripple, FilterType::LowPass)
    }

    pub fn high_pass(cutoff: f64, ripple: f64) -> Self {
        Self::chebyshev(cutoff, 0.0, ripple, FilterType::HighPass)
    }

    pub fn band_pass(lcutoff: f64, hcutoff: f64, ripple: f64) -> Self {
        Self::chebyshev(lcutoff, hcutoff, ripple, FilterType::BandPass)
    }

    pub fn band_stop(lcutoff: f64, hcutoff: f64, ripple: f64) -> Self {
        Self::chebyshev(lcutoff, hcutoff, ripple, FilterType::BandStop)
    }
}

impl<T: Real, const TAPS: usize> From<ChebyshevKernel<T, TAPS>>
    for FilterIir<ChebyshevKernel<T, TAPS>>
{
    fn from(kernel: ChebyshevKernel<T, TAPS>) -> Self {
        FilterIir::from_kernel(kernel)
    }
}

#[derive(Clone, Debug)]
pub struct FilterIir<K: FilterIirKernel> {
    kernel: K,
    x: CircularQueue<K::Sample>,
    y: CircularQueue<K::Sample>,
}

impl<K: FilterIirKernel> FilterIir<K> {
    pub fn from_kernel(kernel: K) -> Self {
        FilterIir {
            x: CircularQueue::with_capacity(K::A_TAPS),
            y: CircularQueue::with_capacity(K::B_TAPS),
            kernel,
        }
    }
}

impl<T: Real, const TAPS: usize> FilterIir<ChebyshevKernel<T, TAPS>> {
    pub fn low_pass(cutoff: f64, ripple: f64) -> Self {
        ChebyshevKernel::low_pass(cutoff, ripple).into()
    }

    pub fn high_pass(cutoff: f64, ripple: f64) -> Self {
        ChebyshevKernel::high_pass(cutoff, ripple).into()
    }

    pub fn band_pass(lcutoff: f64, hcutoff: f64, ripple: f64) -> Self {
        ChebyshevKernel::band_pass(lcutoff, hcutoff, ripple).into()
    }
}

impl<K: FilterIirKernel + Delay> Delay for FilterIir<K> {
    fn delay(&self) -> usize {
        self.kernel.delay()
    }
}

impl<K: FilterIirKernel> Filter<K::Sample> for FilterIir<K>
where
    K::Sample: Real,
{
    type Output = K::Sample;
    fn filter(&mut self, sample: K::Sample) -> Self::Output {
        use num::Float;
        if !sample.is_finite() {
            return sample;
        }

        self.x.push(sample);
        self.y.push(K::Sample::ZERO);

        let output = self
            .x
            .iter()
            .zip(self.kernel.a().iter())
            .map(|(x, a)| x.mul(*a))
            .sum::<K::Sample>()
            + self
                .y
                .iter()
                .skip(1)
                .zip(self.kernel.b().iter().skip(1))
                .map(|(y, b)| y.mul(*b))
                .sum::<K::Sample>();

        if output.is_finite() {
            *self.y.iter_mut().next().unwrap() = output;
        }

        output
    }
}

pub type FilterChebyshev<T, const TAPS: usize> = FilterIir<ChebyshevKernel<T, TAPS>>;

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
        let (a, b) = calc_chebyshev(4, 2, 0.1, 0.0, 10.0, FilterType::HighPass);
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
        let filter = FilterChebyshev::<f64, 5>::low_pass(0.25, 0.5);
        println!("filter_iir_dataset_test3: {:#?}", filter);

        assert!((filter.kernel.a()[0] - 0.07015301).abs() < 0.00001);
        assert!((filter.kernel.a()[1] - 0.2806120).abs() < 0.00001);
        assert!((filter.kernel.a()[2] - 0.4209180).abs() < 0.00001);
        assert!((filter.kernel.a()[3] - 0.2806120).abs() < 0.00001);
        assert!((filter.kernel.a()[4] - 0.07015301).abs() < 0.00001);

        assert!((filter.kernel.b()[1] - 0.4541481).abs() < 0.00001);
        assert!((filter.kernel.b()[2] + 0.7417536).abs() < 0.00001);
        assert!((filter.kernel.b()[3] - 0.2361222).abs() < 0.00001);
        assert!((filter.kernel.b()[4] + 0.07096476).abs() < 0.00001);
    }

    #[test]
    fn filter_iir_low_pass_histogram_2_pole() {
        let kernel = ChebyshevKernel::<_, 3>::low_pass(0.25f64, 0.5f64);

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
        let kernel = ChebyshevKernel::<_, 5>::low_pass(0.25f64, 0.5f64);

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
        let kernel = ChebyshevKernel::<_, 7>::low_pass(0.25f64, 0.5f64);

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
        let kernel = ChebyshevKernel::<_, 3>::high_pass(0.25f64, 0.5f64);

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
        let kernel = ChebyshevKernel::<_, 5>::high_pass(0.25f64, 0.5f64);

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
        let kernel = ChebyshevKernel::<_, 7>::high_pass(0.25f64, 0.5f64);

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
        let gain_h = calc_gain(FilterChebyshev::<_, 3>::low_pass(0.25f64, 0.5f64), 0.45f64);
        println!("filter_iir_low_pass: 2-pole gain_h: {:.2}dB", gain_h);
        assert!(gain_h < -15.0);

        let gain_l = calc_gain(FilterChebyshev::<_, 3>::low_pass(0.25f64, 0.5f64), 0.05f64);
        println!("filter_iir_low_pass: 2-pole gain_l: {:.2}dB", gain_l);
        assert!(gain_l > -0.5);
        assert!(gain_l < 0.01);
    }

    #[test]
    fn filter_iir_low_pass_performance_4pole() {
        let gain_h = calc_gain(FilterChebyshev::<_, 5>::low_pass(0.25f64, 0.5f64), 0.45f64);
        println!("filter_iir_low_pass: 4-pole gain_h: {:.2}dB", gain_h);
        assert!(gain_h < -35.0);

        let gain_l = calc_gain(FilterChebyshev::<_, 5>::low_pass(0.25f64, 0.5f64), 0.05f64);
        println!("filter_iir_low_pass: 4-pole gain_l: {:.2}dB", gain_l);
        assert!(gain_l > -0.5);
        assert!(gain_l < 0.01);
    }

    #[test]
    fn filter_iir_low_pass_performance_6pole() {
        let gain_h = calc_gain(FilterChebyshev::<_, 7>::low_pass(0.25f64, 0.5f64), 0.45f64);
        println!("filter_iir_low_pass: 6-pole gain_h: {:.2}dB", gain_h);
        assert!(gain_h < -55.0);

        let gain_l = calc_gain(FilterChebyshev::<_, 7>::low_pass(0.25f64, 0.5f64), 0.05f64);
        println!("filter_iir_low_pass: 6-pole gain_l: {:.2}dB", gain_l);
        assert!(gain_l > -0.5);
        assert!(gain_l < 0.01);
    }

    #[test]
    //    #[ignore] // Currently failing.
    fn filter_iir_high_pass_performance_2pole() {
        let gain_l = calc_gain(FilterChebyshev::<_, 3>::high_pass(0.25f64, 0.5f64), 0.05f64);
        println!("filter_iir_high_pass: 2-pole gain_l: {:.2}dB", gain_l);
        assert!(gain_l < -16.0);

        let gain_h = calc_gain(FilterChebyshev::<_, 3>::high_pass(0.25f64, 0.5f64), 0.5f64);
        println!("filter_iir_high_pass: 2-pole gain_h: {:.2}dB", gain_h);
        assert!(gain_h > -0.5);
        assert!(gain_h < 0.01);
    }

    #[test]
    //    #[ignore] // Currently failing.
    fn filter_iir_high_pass_performance_4pole() {
        let gain_l = calc_gain(FilterChebyshev::<_, 5>::high_pass(0.25f64, 0.5f64), 0.05f64);
        println!("filter_iir_high_pass: 4-pole gain_l: {:.2}dB", gain_l);
        assert!(gain_l < -35.0);

        let gain_h = calc_gain(FilterChebyshev::<_, 5>::high_pass(0.25f64, 0.5f64), 0.5f64);
        println!("filter_iir_high_pass: 4-pole gain_h: {:.2}dB", gain_h);
        assert!(gain_h > -0.5);
        assert!(gain_h < 0.01);
    }

    #[test]
    //    #[ignore] // Currently failing.
    fn filter_iir_high_pass_performance_6pole() {
        let gain_l = calc_gain(FilterChebyshev::<_, 7>::high_pass(0.25f64, 0.5f64), 0.05f64);
        println!("filter_iir_high_pass: 6-pole gain_l: {:.2}dB", gain_l);
        assert!(gain_l < -52.0);

        let gain_h = calc_gain(FilterChebyshev::<_, 7>::high_pass(0.25f64, 0.5f64), 0.5f64);
        println!("filter_iir_high_pass: 6-pole gain_h: {:.2}dB", gain_h);
        assert!(gain_h > -0.5);
        assert!(gain_h < 0.01);
    }
}
