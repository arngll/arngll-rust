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

use crate::*;
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::fmt::{Debug, Display, Formatter};

/// Represents a single callsign character from the ARNCE character set.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
#[repr(transparent)]
pub struct HamChar(u8);

impl HamChar {
    /// Constant for null (`NUL`) HamChar character.
    pub const NUL: HamChar = HamChar(0);

    /// Constant for escape (`ESC`) HamChar character.
    pub const ESC: HamChar = HamChar(39);

    /// Tries to create a HamChar from the given ASCII byte.
    ///
    /// ```
    /// # use hamaddr::HamChar;
    /// let c = HamChar::from_ascii_byte(b'3');
    /// assert!(c.is_some());
    ///
    /// let c = HamChar::from_ascii_byte(b'?');
    /// assert!(c.is_none());
    /// ```
    pub const fn from_ascii_byte(c: u8) -> Option<HamChar> {
        match c {
            b'\x00' => Some(HamChar::NUL),
            b'A'..=b'Z' => Some(HamChar(c - b'A' + 1)),
            b'a'..=b'z' => Some(HamChar(c - b'a' + 1)),
            b'0'..=b'9' => Some(HamChar(c - b'0' + 27)),
            b'/' => Some(HamChar(37)),
            b'-' => Some(HamChar(38)),
            b'^' => Some(HamChar::ESC),
            _ => None,
        }
    }

    /// Tries to create a HamChar from the given unicode character.
    ///
    /// ```
    /// # use hamaddr::HamChar;
    /// let c = HamChar::from_char('3');
    /// assert!(c.is_some());
    ///
    /// let c = HamChar::from_char('?');
    /// assert!(c.is_none());
    /// ```
    pub const fn from_char(c: char) -> Option<HamChar> {
        let c = c as u32;
        if c < 128 {
            Self::from_ascii_byte(c.to_le_bytes()[0])
        } else {
            None
        }
    }

    /// Converts this HamChar into an ASCII-encoded byte.
    ///
    /// ```
    /// # use hamaddr::HamChar;
    /// let c = HamChar::from_char('3').unwrap();
    /// assert_eq!(c.to_ascii_byte(), b'3');
    ///
    /// let c = HamChar::from_char('\x00').unwrap();
    /// assert_eq!(c.to_ascii_byte(), 0);
    /// ```
    pub const fn to_ascii_byte(&self) -> u8 {
        const CHARS: &'static [u8] = b"\x00ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789/-^";
        CHARS[self.0 as usize]
    }

    /// Converts this HamChar into a unicode character, rendering `NUL` as `␀`.
    ///
    /// ```
    /// # use hamaddr::HamChar;
    /// let c = HamChar::from_char('3').unwrap();
    /// assert_eq!(c.to_char(), '3');
    ///
    /// let c = HamChar::from_char('\x00').unwrap();
    /// assert_eq!(c.to_char(), '␀');
    /// ```
    pub const fn to_char(&self) -> char {
        const ALT_NUL: char = '␀';
        if self.is_nul() {
            ALT_NUL
        } else {
            self.to_ascii_byte() as char
        }
    }

    /// Returns the index of this character in the ARNCE character set.
    ///
    /// ```
    /// # use hamaddr::HamChar;
    /// let c = HamChar::from_char('0').unwrap();
    /// assert_eq!(c.index(), 27);
    /// ```
    pub const fn index(&self) -> u8 {
        self.0
    }

    /// Returns true if this HamChar is the NULL (`NUL`) character.
    ///
    /// ```
    /// # use hamaddr::HamChar;
    /// let c = HamChar::from_ascii_byte(0).unwrap();
    /// assert!(c.is_nul());
    /// ```
    pub const fn is_nul(&self) -> bool {
        self.0 == Self::NUL.0
    }

    /// Returns true if this HamChar is the escape (`ESC`) character.
    ///
    /// ```
    /// # use hamaddr::HamChar;
    /// let c = HamChar::from_char('^').unwrap();
    /// assert!(c.is_esc());
    /// ```
    pub const fn is_esc(&self) -> bool {
        self.0 == Self::ESC.0
    }
}

impl TryFrom<char> for HamChar {
    type Error = InvalidChar;

    fn try_from(value: char) -> std::result::Result<Self, Self::Error> {
        HamChar::from_char(value).ok_or(InvalidChar)
    }
}

impl TryFrom<u8> for HamChar {
    type Error = InvalidChar;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        HamChar::from_ascii_byte(value).ok_or(InvalidChar)
    }
}

impl From<HamChar> for char {
    fn from(value: HamChar) -> Self {
        value.to_char()
    }
}

impl Display for HamChar {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.to_char(), f)
    }
}

/// A "Chunk" of three HamChars that can be easily converted from/into a `u16`.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub(crate) struct HamCharChunk(pub [HamChar; 3]);

impl From<HamCharChunk> for u16 {
    fn from(chunk: HamCharChunk) -> Self {
        (chunk.0[0].0 as u16) * 1600 + (chunk.0[1].0 as u16) * 40 + (chunk.0[2].0 as u16)
    }
}

impl TryFrom<u16> for HamCharChunk {
    type Error = InvalidChunk;

    fn try_from(chunk: u16) -> Result<Self, Self::Error> {
        if chunk == 0 || (chunk >= 0x0640 && chunk <= 0xF9FF) {
            Ok(HamCharChunk([
                HamChar((chunk / 1600u16 % 40u16).try_into().unwrap()),
                HamChar((chunk / 40u16 % 40u16).try_into().unwrap()),
                HamChar((chunk % 40u16).try_into().unwrap()),
            ]))
        } else {
            Err(InvalidChunk)
        }
    }
}

impl TryFrom<[char; 3]> for HamCharChunk {
    type Error = InvalidCharAt;

    fn try_from(chunk: [char; 3]) -> Result<Self, Self::Error> {
        Ok(HamCharChunk([
            chunk[0].try_into().map_err(|_| InvalidCharAt(0))?,
            chunk[1].try_into().map_err(|_| InvalidCharAt(1))?,
            chunk[2].try_into().map_err(|_| InvalidCharAt(2))?,
        ]))
    }
}

impl Display for HamCharChunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if f.alternate() || !self.0[2].is_nul() {
            write!(f, "{}{}{}", self.0[0], self.0[1], self.0[2])
        } else if !self.0[1].is_nul() {
            write!(f, "{}{}", self.0[0], self.0[1])
        } else if !self.0[0].is_nul() {
            write!(f, "{}", self.0[0])
        } else {
            Ok(())
        }
    }
}
