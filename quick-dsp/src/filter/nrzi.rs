use super::*;

// NRZI Encoding...
//
// 0: Transition
// 1: No transition

#[derive(Clone, Debug)]
pub struct NrziEncode {
    pub last: bool,
}

impl NrziEncode {
    pub fn new() -> Self {
        NrziEncode { last: false }
    }
}

impl OneToOne<bool> for NrziEncode {
    type Output = bool;

    fn filter(&mut self, sample: bool) -> Self::Output {
        if sample == false {
            self.last = !self.last;
        }
        self.last
    }
}

impl Delay for NrziEncode {
    fn delay(&self) -> usize {
        0
    }
}

#[derive(Clone, Debug)]
pub struct NrziDecode {
    pub last: bool,
}

impl NrziDecode {
    pub fn new() -> Self {
        NrziDecode { last: false }
    }
}

impl OneToOne<bool> for NrziDecode {
    type Output = bool;

    fn filter(&mut self, sample: bool) -> Self::Output {
        if sample != self.last {
            self.last = sample;
            false
        } else {
            true
        }
    }
}

impl Delay for NrziDecode {
    fn delay(&self) -> usize {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nrzi_pipe() {
        let encode = NrziEncode::new();
        let decode = NrziDecode::new();
        let mut chained = OneToOneExt::<bool>::chain(encode, decode);

        assert_eq!(chained.filter(true), true);
        assert_eq!(chained.filter(false), false);
        assert_eq!(chained.filter(true), true);
        assert_eq!(chained.filter(false), false);
        assert_eq!(chained.filter(true), true);
        assert_eq!(chained.filter(false), false);
        assert_eq!(chained.filter(false), false);
        assert_eq!(chained.filter(false), false);
        assert_eq!(chained.filter(true), true);
        assert_eq!(chained.filter(true), true);
        assert_eq!(chained.filter(true), true);
        assert_eq!(chained.filter(false), false);
        assert_eq!(chained.filter(true), true);
    }
}
