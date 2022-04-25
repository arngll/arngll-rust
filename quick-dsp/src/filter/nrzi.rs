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

impl Filter<bool> for NrziEncode {
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

impl Filter<bool> for NrziDecode {
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
        let mut chained = FilterExt::<bool>::chain(encode, decode);

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
