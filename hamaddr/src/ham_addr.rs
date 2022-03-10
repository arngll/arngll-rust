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
use anyhow::bail;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Debug, Display};
use std::iter::FusedIterator;
use std::num::NonZeroU16;
use std::str::FromStr;

/// An [ARNCE][]-encoded address.
///
/// [ARNCE]: https://github.com/arngll/arnce-spec/blob/main/n6drc-arnce.md#introduction
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct HamAddr([u8; 8]);

/// Describes `HamAddr` types.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum HamAddrType {
    /// Empty address.
    Empty,

    /// Callsign.
    Callsign,

    /// For IPv4 Multicast.
    Ipv4Multicast,

    /// For IPv6 Multicast.
    Ipv6Multicast,

    /// A short address.
    Short,

    /// The broadcast address.
    Broadcast,

    /// A reserved address value.
    Reserved,
}

impl HamAddr {
    /// Empty Address Constant.
    ///
    /// ```
    /// # use hamaddr::HamAddr;
    /// assert!(HamAddr::EMPTY.is_empty());
    /// ```
    pub const EMPTY: HamAddr = HamAddr([0, 0, 0, 0, 0, 0, 0, 0]);

    /// Broadcast Address Constant.
    ///
    /// ```
    /// # use hamaddr::HamAddr;
    /// assert!(HamAddr::BROADCAST.is_broadcast());
    /// ```
    pub const BROADCAST: HamAddr = HamAddr([0xFF, 0xFF, 0, 0, 0, 0, 0, 0]);

    /// Creates a new `HamAddr` from the given array of 8 bytes.
    ///
    /// ```
    /// # use hamaddr::HamAddr;
    /// let addr = HamAddr::new([0x46,0x71,0x6C,0xA0,0,0,0,0]);
    /// assert_eq!(addr.to_string(), "KJ6QOH");
    /// ```
    pub const fn new(octets: [u8; 8]) -> HamAddr {
        HamAddr(octets)
    }

    /// Creates a new `HamAddr` from the given short address.
    ///
    /// Returns `None` if `shortaddr` is larger than `0x063F`.
    ///
    /// ```
    /// # use std::num::NonZeroU16;
    /// # use hamaddr::HamAddr;
    /// let addr = HamAddr::try_from_shortaddr(NonZeroU16::new(48).unwrap()).unwrap();
    /// assert_eq!(addr.shortaddr(), NonZeroU16::new(48));
    /// ```
    pub const fn try_from_shortaddr(shortaddr: NonZeroU16) -> Option<HamAddr> {
        if shortaddr.get() > 0x063F {
            return None;
        }
        let bytes = shortaddr.get().to_be_bytes();
        let mut ret = Self::EMPTY;
        ret.0[0] = bytes[0];
        ret.0[1] = bytes[1];
        Some(ret)
    }

    /// Creates a `HamAddr` from a an array of four `u16` "chunks".
    ///
    /// ```
    /// # use hamaddr::HamAddr;
    /// let addr = HamAddr::from_chunks([0x4671,0x6CA0,0,0]);
    /// assert_eq!(addr.to_string(), "KJ6QOH");
    /// ```
    pub fn from_chunks(chunks: [u16; 4]) -> HamAddr {
        let mut ret = Self::EMPTY;
        let mut iter_mut = ret.0.iter_mut();
        let mut iter = chunks.into_iter().flat_map(u16::to_be_bytes);
        for _ in 0..8 {
            *iter_mut.next().unwrap() = iter.next().unwrap()
        }
        ret
    }

    /// Tries to create a HamAddr from a byte slice.
    ///
    /// The byte slice must be either 2, 4, 6, or 8 bytes long.
    ///
    /// ```
    /// # use hamaddr::HamAddr;
    /// let addr = HamAddr::try_from_slice(&[0x46,0x71,0x6C,0xA0]).unwrap();
    /// assert_eq!(addr.to_string(), "KJ6QOH");
    /// ```
    pub fn try_from_slice(bytes: &[u8]) -> Result<HamAddr> {
        if (bytes.len() & 1) == 1 || bytes.len() > 8 {
            bail!("Invalid slice length");
        }
        let mut ret = HamAddr::EMPTY;
        ret.0[..bytes.len()].copy_from_slice(bytes);
        Ok(ret)
    }

    /// Tries to create a HamAddr from the given callsign string.
    ///
    /// ```
    /// # use hamaddr::HamAddr;
    /// let addr = HamAddr::try_from_callsign("kj6QOH").unwrap();
    /// assert_eq!(addr.to_string(), "KJ6QOH");
    /// ```
    pub fn try_from_callsign(callsign: &str) -> Result<HamAddr> {
        // Iterator type for converting a string into chunks.
        struct StrChunkIterator<T: Iterator<Item = char> + FusedIterator>(T);
        impl<T: Iterator<Item = char> + FusedIterator> Iterator for StrChunkIterator<T> {
            type Item = Result<u16, anyhow::Error>;
            fn next(&mut self) -> Option<Self::Item> {
                let c0 = self.0.next()?;
                let c1 = self.0.next().unwrap_or('\x00');
                let c2 = self.0.next().unwrap_or('\x00');
                Some(
                    HamCharChunk::try_from([c0, c1, c2])
                        .map(u16::from)
                        .map_err(anyhow::Error::from),
                )
            }
        }

        // Handle special non-callsign cases.
        if let Some('~') | None = callsign.chars().next() {
            if callsign.len() <= 1 {
                return Ok(HamAddr::EMPTY);
            }
            if callsign == "~FFFF" || callsign == "~ffff" {
                return Ok(HamAddr::BROADCAST);
            }
            bail!("Unsupported raw notation: {:?}", callsign);
        }

        let mut iter = StrChunkIterator(callsign.chars());
        let mut chunks = [0u16; 4];

        for chunk in chunks.iter_mut() {
            *chunk = iter.next().transpose()?.unwrap_or(0);
        }

        if iter.next().is_some() {
            bail!("Callsign too long");
        }

        Ok(HamAddr::from_chunks(chunks))
    }

    /// Tries to return the value of this `HamAddr` as a temporary short addreses.
    ///
    /// ```
    /// # use std::num::NonZeroU16;
    /// # use hamaddr::HamAddr;
    /// let addr = HamAddr::try_from_shortaddr(NonZeroU16::new(48).unwrap()).unwrap();
    /// assert_eq!(addr.shortaddr(), NonZeroU16::new(48));
    /// ```
    pub const fn shortaddr(&self) -> Option<NonZeroU16> {
        // We use match instead of == so we can stay const.
        match self.get_type() {
            HamAddrType::Short => NonZeroU16::new(self.chunk(0)),
            _ => None,
        }
    }

    /// Returns the value of this HamAddr as an array of 8 bytes.
    pub fn octets(&self) -> [u8; 8] {
        self.0.clone()
    }

    /// Returns the value of this HamAddr as a byte slice of 8 bytes.
    pub const fn as_slice(&self) -> &[u8] {
        &self.0
    }

    /// Returns the value of this HamAddr as a trimmed byte slice of
    /// either 2, 4, 6, or 8 bytes.
    pub fn as_trimmed_slice(&self) -> &[u8] {
        &self.0[..self.len()]
    }

    /// Returns the minimum required length to
    /// encode this address in bytes.
    pub const fn len(&self) -> usize {
        if self.chunk(3) != 0 {
            8
        } else if self.chunk(2) != 0 {
            6
        } else if self.chunk(1) != 0 {
            4
        } else {
            2
        }
    }

    /// Returns a single 16-bit chunk, by index.
    ///
    /// Will panic if `i` is greater than 3.
    pub const fn chunk(&self, i: usize) -> u16 {
        u16::from_be_bytes([self.0[i * 2], self.0[i * 2 + 1]])
    }

    /// Returns the underlying value as an array of four u16 "chunks".
    pub const fn chunks(&self) -> [u16; 4] {
        [self.chunk(0), self.chunk(1), self.chunk(2), self.chunk(3)]
    }

    /// Returns `true` if this `HamAddr` is equal to `HamAddr::EMPTY`.
    pub const fn is_empty(&self) -> bool {
        // We do individual chunk comparisons so that we can remain const.
        self.chunk(0) == 0 && self.chunk(1) == 0 && self.chunk(2) == 0 && self.chunk(3) == 0
    }

    /// Returns `true` if this `HamAddr` is a callsign.
    pub const fn is_callsign(&self) -> bool {
        // We use match instead of == so we can stay const.
        match self.get_type() {
            HamAddrType::Callsign => true,
            _ => false,
        }
    }

    /// Returns `true` if this `HamAddr` is a unicast address(either
    /// a callsign or a short address).
    pub const fn is_unicast(&self) -> bool {
        match self.get_type() {
            HamAddrType::Callsign | HamAddrType::Short => true,
            _ => false,
        }
    }

    /// Returns `true` if this `HamAddr` is equal to `HamAddr::BROADCAST`.
    pub const fn is_broadcast(&self) -> bool {
        // We do individual chunk comparisons so that we can remain const.
        self.chunk(0) == 0xFFFF && self.chunk(1) == 0 && self.chunk(2) == 0 && self.chunk(3) == 0
    }

    /// Returns `true` if this `HamAddr` is a reserved value.
    pub const fn is_reserved(&self) -> bool {
        // We use match instead of == so we can stay const.
        match self.get_type() {
            HamAddrType::Reserved => true,
            _ => false,
        }
    }

    /// Returns `true` if this `HamAddr` is any type of multicast address.
    pub const fn is_multicast(&self) -> bool {
        match self.get_type() {
            HamAddrType::Ipv4Multicast | HamAddrType::Ipv6Multicast => true,
            _ => false,
        }
    }

    /// Returns `true` if this `HamAddr` is any type of multicast or broadcast address.
    pub const fn is_multicast_or_broadcast(&self) -> bool {
        match self.get_type() {
            HamAddrType::Ipv4Multicast | HamAddrType::Ipv6Multicast | HamAddrType::Broadcast => {
                true
            }
            _ => false,
        }
    }

    /// Returns the type of this `HamAddr`.
    pub const fn get_type(&self) -> HamAddrType {
        if self.is_empty() {
            HamAddrType::Empty
        } else if self.chunk(0) < 0x0640 {
            if self.len() == 2 {
                HamAddrType::Short
            } else {
                HamAddrType::Reserved
            }
        } else if self.chunk(0) < 0xFA00 {
            // We unroll our loop over the remaining chunks
            // so we can keep this function const.
            let chunk = self.chunk(1);
            if chunk != 0 && (chunk < 0x0640 || chunk >= 0xFA00) {
                return HamAddrType::Reserved;
            }
            let chunk = self.chunk(2);
            if chunk != 0 && (chunk < 0x0640 || chunk >= 0xFA00) {
                return HamAddrType::Reserved;
            }
            let chunk = self.chunk(3);
            if chunk != 0 && (chunk < 0x0640 || chunk >= 0xFA00) {
                return HamAddrType::Reserved;
            }

            HamAddrType::Callsign
        } else if self.is_broadcast() {
            HamAddrType::Broadcast
        } else if self.0[0] == 0xFA {
            HamAddrType::Ipv6Multicast
        } else if self.0[0] == 0xFB {
            HamAddrType::Ipv4Multicast
        } else {
            HamAddrType::Reserved
        }
    }

    /// Renders this address to a string in trimmed
    /// hexadecimal notation.
    pub fn to_addr_string(&self) -> String {
        format!("{:?}", self)
    }
}

/// The Display formatter for `HamAddr` prints out the address
/// in a format appropriate for human reading.
///
/// Callsigns are rendered directly to text, with all other
/// representations rendered as the hex notation preceded with a `~`.
impl Display for HamAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.get_type() {
            HamAddrType::Empty => {
                write!(f, "~")
            }
            HamAddrType::Callsign => {
                for chunk in self
                    .chunks()
                    .into_iter()
                    .map(|x| HamCharChunk::try_from(x).unwrap())
                {
                    write!(f, "{}", chunk)?;
                }
                Ok(())
            }
            _ => {
                write!(f, "~{:?}", self)
            }
        }
    }
}

/// The Debug formatter for `HamAddr` prints out the address
/// in its abbreviated raw hex form, like `5CAC-70F8` or `FAFB`.
/// In the alternate rendering (`{:#?}`), the non-abbreviated
/// form is used (e.g. `5CAC-70F8-0000-0000` or `FAFB-0000-0000-0000`).
impl Debug for HamAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() || self.len() >= 8 {
            write!(
                f,
                "{:04X?}-{:04X?}-{:04X?}-{:04X?}",
                self.chunk(0),
                self.chunk(1),
                self.chunk(2),
                self.chunk(3)
            )
        } else if self.len() >= 6 {
            write!(
                f,
                "{:04X?}-{:04X?}-{:04X?}",
                self.chunk(0),
                self.chunk(1),
                self.chunk(2)
            )
        } else if self.len() >= 4 {
            write!(f, "{:04X?}-{:04X?}", self.chunk(0), self.chunk(1))
        } else {
            write!(f, "{:04X?}", self.chunk(0))
        }
    }
}

impl FromStr for HamAddr {
    type Err = anyhow::Error;

    fn from_str(callsign: &str) -> std::result::Result<Self, Self::Err> {
        HamAddr::try_from_callsign(callsign)
    }
}

impl TryFrom<HamAddr> for Eui64 {
    type Error = anyhow::Error;
    fn try_from(value: HamAddr) -> std::result::Result<Self, Self::Error> {
        match value.get_type() {
            HamAddrType::Empty => Ok(Eui64::EMPTY),
            HamAddrType::Broadcast => Ok(Eui64::BROADCAST),
            HamAddrType::Callsign => {
                if value.0[7] & 0b0111 != 0 {
                    bail!("HamAddr too big");
                }

                // If the last chunk is empty and the last three
                // bits on the second-to-last chunk are zero,
                // then the address can be rendered as an EUI-48.
                let is_small = value.chunk(3) == 0 && (value.chunk(2) & 0b0111) == 0;
                let mut bytes = value.octets();
                let first_byte = std::mem::take(&mut bytes[if is_small { 5 } else { 7 }]);
                bytes.rotate_right(1);
                bytes[0] = (first_byte & 0b1111_1000) | 0b0010;
                if is_small {
                    bytes[3..].rotate_right(2);
                    bytes[3] = 0xFF;
                    bytes[4] = 0xFE;
                }
                Ok(Eui64::new(bytes))
            }

            HamAddrType::Ipv4Multicast | HamAddrType::Ipv6Multicast => {
                bail!("Multicast EUI64 conversion not supported")
            }
            x => bail!("Cannot convert {:?} to EUI64", x),
        }
    }
}

impl TryFrom<HamAddr> for Eui48 {
    type Error = anyhow::Error;
    fn try_from(value: HamAddr) -> std::result::Result<Self, Self::Error> {
        match value.get_type() {
            HamAddrType::Empty => Ok(Eui48::EMPTY),
            HamAddrType::Broadcast => Ok(Eui48::BROADCAST),
            HamAddrType::Callsign => {
                let is_small = value.chunk(3) == 0 && (value.chunk(2) & 0b0111) == 0;
                if !is_small {
                    bail!("HamAddr too big");
                }
                let mut bytes = [0u8; 6];
                bytes.copy_from_slice(&value.octets()[..6]);
                bytes.rotate_right(1);
                bytes[0] = (bytes[0] & 0b1111_1000) | 0b0010;
                Ok(Eui48::new(bytes))
            }

            HamAddrType::Ipv4Multicast => {
                let bytes = value.as_slice();
                Ok(Eui48::new([0x01, 0x00, 0x5e, bytes[3], bytes[2], bytes[1]]))
            }

            HamAddrType::Ipv6Multicast => {
                let bytes = value.as_slice();
                Ok(Eui48::new([
                    0xcc, 0xcc, bytes[4], bytes[3], bytes[2], bytes[1],
                ]))
            }
            x => bail!("Cannot convert {:?} to EUI48", x),
        }
    }
}

/// Converts an Eui48 into a HamAddr
impl TryFrom<Eui48> for HamAddr {
    type Error = anyhow::Error;
    fn try_from(value: Eui48) -> std::result::Result<Self, Self::Error> {
        if value == Eui48::EMPTY {
            return Ok(HamAddr::EMPTY);
        }
        if value == Eui48::BROADCAST {
            return Ok(HamAddr::BROADCAST);
        }
        if value.0[..3] == [0x01, 0x00, 0x5e] {
            // IPv4 multicast
            return Ok(
                HamAddr::try_from_slice(&[0xFB, value.0[5], value.0[4], value.0[3]]).unwrap(),
            );
        }
        if value.0[..2] == [0xCC, 0xCC] {
            // IPv6 multicast
            return Ok(HamAddr::try_from_slice(&[
                0xFA, value.0[5], value.0[4], value.0[3], value.0[2], 0x00,
            ])
            .unwrap());
        }
        let mut octets = value.0;
        if octets[0] & 0b111 == 0b010 {
            octets[0] &= 0b1111_1101;
            octets.rotate_left(1);
            let mut bytes = [0; 8];
            bytes[..6].copy_from_slice(&octets);
            let ret = HamAddr(bytes);
            match ret.get_type() {
                HamAddrType::Callsign => Ok(ret),
                _ => bail!("Cannot convert from EUI48 to ham addr"),
            }
        } else {
            bail!("Cannot convert from EUI64 to ham addr")
        }
    }
}

/// Converts an Eui64 into a HamAddr.
impl TryFrom<Eui64> for HamAddr {
    type Error = anyhow::Error;
    fn try_from(value: Eui64) -> std::result::Result<Self, Self::Error> {
        if value == Eui64::EMPTY {
            return Ok(HamAddr::EMPTY);
        }
        if value == Eui64::BROADCAST {
            return Ok(HamAddr::BROADCAST);
        }
        let mut bytes = value.0;
        if bytes[0] & 0b111 == 0b010 {
            bytes[0] &= 0b1111_1101;
            if bytes[3] == 0xFF && bytes[4] == 0xFE {
                bytes[3..].rotate_left(2);
                bytes[6] = 0;
                bytes[7] = 0;
                bytes[..6].rotate_left(1);
            } else {
                bytes.rotate_left(1);
            }
            let ret = HamAddr(bytes);
            match ret.get_type() {
                HamAddrType::Callsign => Ok(ret),
                _ => bail!("Cannot convert from EUI64 to ham addr"),
            }
        } else {
            bail!("Cannot convert from EUI64 to ham addr")
        }
    }
}
