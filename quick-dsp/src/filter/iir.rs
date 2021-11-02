/// IIR filter. Not yet complete.
use super::*;

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

impl<T> FilterIirKernel<T> {
    pub fn new(a: Vec<T>, b: Vec<T>, delay: usize) -> Self {
        FilterIirKernel { a, b, delay }
    }

    pub fn len(&self) -> usize {
        self.a.len()
    }

    pub fn poles(&self) -> usize {
        self.a.len() - 1
    }
}

impl<T: Real> FilterIirKernel<T> {
    pub fn low_pass(_poles: usize, _cutoff: f64, _window: Window) -> Self {
        todo!()
    }

    pub fn high_pass(_poles: usize, _cutoff: f64, _window: Window) -> Self {
        todo!()
    }

    pub fn band_pass(_poles: usize, _lcutoff: f64, _hcutoff: f64, _window: Window) -> Self {
        todo!()
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

    pub fn low_pass(poles: usize, cutoff: f64, window: Window) -> Self {
        FilterIirKernel::low_pass(poles, cutoff, window).into()
    }

    pub fn high_pass(poles: usize, cutoff: f64, window: Window) -> Self {
        FilterIirKernel::high_pass(poles, cutoff, window).into()
    }

    pub fn band_pass(poles: usize, lcutoff: f64, hcutoff: f64, window: Window) -> Self {
        FilterIirKernel::band_pass(poles, lcutoff, hcutoff, window).into()
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
        self.y.push(
            self.x
                .iter()
                .zip(self.kernel.a.iter())
                .map(|(x, a)| x.mul(*a))
                .sum(),
        );

        let output: T = self
            .y
            .iter()
            .zip(self.kernel.b.iter())
            .map(|(x, b)| x.mul(*b))
            .sum();

        if output.is_finite() {
            *self.y.iter_mut().next().unwrap() = output;
        }

        output
    }
}
