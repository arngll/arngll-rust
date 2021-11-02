/// FIR Filter.
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

    #[test]
    fn filter_fir() {
        let mut filter = FilterFir::low_pass(12, 0.25, Window::Hamming);

        println!("Filter = {:?}", filter);
        assert_eq!(filter.filter(0.0), 0.0);
        assert_eq!(filter.filter(0.0), 0.0);
        assert_eq!(filter.filter(0.0), 0.0);
        assert_eq!(filter.filter(0.0), 0.0);
        assert_eq!(filter.filter(0.0), 0.0);
        assert_eq!(filter.filter(0.0), 0.0);
        assert_eq!(filter.filter(0.0), 0.0);
        assert_eq!(filter.filter(0.0), 0.0);
        assert_eq!(filter.filter(0.0), 0.0);
    }
}