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

use std::fmt::{Debug, Formatter};
use super::*;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct NetworkId(pub u16);

impl NetworkId {
    pub fn from_iter<'a, T: Iterator<Item=&'a u8>>(iter: &mut T) -> NetworkId {
        let msb = *iter.next().unwrap();
        let lsb = *iter.next().unwrap();
        NetworkId(((msb as u16)<<8) | (lsb as u16))
    }
}

impl Debug for NetworkId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:04X}]", self.0)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FrameType {
    Beacon,
    Data,
    Ack,
    MacCommand,
}

impl FrameType {
    pub fn try_from_u8(x: u8) -> Option<FrameType> {
        match x {
            0 => Some(Self::Beacon),
            1 => Some(Self::Data),
            2 => Some(Self::Ack),
            3 => Some(Self::MacCommand),
            _ => None,
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            Self::Beacon => 0,
            Self::Data => 1,
            Self::Ack => 2,
            Self::MacCommand => 3,
        }
    }
}

impl TryFrom<u8> for FrameType {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        FrameType::try_from_u8(value).ok_or(format_err!("{} is not a valid frame type", value))
    }
}

impl From<FrameType> for u8 {
    fn from(value: FrameType) -> u8 {
        value.to_u8()
    }
}

/// Enum encoding the length of a Message Integrity Code.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MicLen {
    Mic32 = 0,
    Mic64 = 1,
    Mic96 = 2,
    Mic128 = 3,
}

impl MicLen {
    pub fn try_from_u8(x: u8) -> Option<MicLen> {
        match x {
            0 => Some(Self::Mic32),
            1 => Some(Self::Mic64),
            2 => Some(Self::Mic96),
            3 => Some(Self::Mic128),
            _ => None,
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            Self::Mic32 => 0,
            Self::Mic64 => 1,
            Self::Mic96 => 2,
            Self::Mic128 => 3,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Mic32 => 4,
            Self::Mic64 => 8,
            Self::Mic96 => 12,
            Self::Mic128 => 16,
        }
    }
}

impl TryFrom<u8> for MicLen {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        MicLen::try_from_u8(value).ok_or(format_err!("{} is not a valid MIC length", value))
    }
}

impl From<MicLen> for u8 {
    fn from(value: MicLen) -> u8 {
        value.to_u8()
    }
}

/// Message Integrity Code.
#[derive(Clone, Eq, PartialEq)]
pub struct Mic {
    pub len: MicLen,
    pub code: [u8; 16],
}

impl Debug for Mic {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", hex::encode(self.as_slice()))
    }
}

impl Mic {
    const EMPTY: Mic = Mic { len: MicLen::Mic32, code: [0u8; 16]};

    pub fn len(&self) -> usize {
        self.len.len()
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.code[..self.len()]
    }

    pub fn try_from_slice(slice: &[u8]) -> Result<Mic, anyhow::Error> {
        if slice.len() < 4 {
            bail!("Bad MIC size");
        }
        let mic_len = MicLen::try_from_u8(((slice.len()/4)-1).try_into()?)
            .ok_or(format_err!("Bad MIC size"))?;
        let mut code = [0u8; 16];
        (&mut code[..slice.len()]).copy_from_slice(slice);
        Ok(Mic{len:mic_len, code})
    }

    pub fn bytes(&self) -> impl Iterator<Item=u8> {
        self.code.clone().into_iter().take(self.len())
    }
}

impl Default for Mic {
    fn default() -> Self {
        Mic::EMPTY
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum KeyIdentMode {
    Addresses = 0,
    KeyIndex = 1,
    Reserved2 = 2,
    Reserved3 = 3,
}

impl KeyIdentMode {
    pub fn try_from_u8(x: u8) -> Option<KeyIdentMode> {
        match x {
            0 => Some(Self::Addresses),
            1 => Some(Self::KeyIndex),
            2 => Some(Self::Reserved2),
            3 => Some(Self::Reserved3),
            _ => None,
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            Self::Addresses => 0,
            Self::KeyIndex => 1,
            Self::Reserved2 => 2,
            Self::Reserved3 => 3,
        }
    }
}

impl TryFrom<u8> for KeyIdentMode {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        KeyIdentMode::try_from_u8(value).ok_or(format_err!("{} is not a valid key ident mode", value))
    }
}

impl From<KeyIdentMode> for u8 {
    fn from(value: KeyIdentMode) -> u8 {
        value.to_u8()
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct SecInfo {
    pub enc: bool,
    pub kim: KeyIdentMode,
    pub fcntr: u32,
    pub kid: Option<u8>,
    pub mic: Mic,
}

impl SecInfo {
    pub fn from_iter<'a, T: Iterator<Item=&'a u8>>(iter: &mut T) -> SecInfo {
        let scf = iter.next().copied().unwrap();
        let enc = (scf & 0b10000000) != 0;
        let miclen = MicLen::try_from_u8((scf & 0b01100000) >> 5).unwrap();
        let kim = KeyIdentMode::try_from_u8((scf & 0b00011000) >> 3).unwrap();
        let fcntr = u32::from_be_bytes([
            iter.next().copied().unwrap(),
            iter.next().copied().unwrap(),
            iter.next().copied().unwrap(),
            iter.next().copied().unwrap()
        ]);
        let kid = if kim == KeyIdentMode::KeyIndex {
            Some(iter.next().copied().unwrap())
        } else {
            None
        };

        SecInfo {
            enc,
            kim,
            fcntr,
            kid,
            mic: Mic { len: miclen, .. Mic::EMPTY }
        }
    }

    pub fn scf(&self) -> u8 {
        (u8::from(self.enc) << 7)
            + (self.mic.len.to_u8() << 5)
            + (self.kim.to_u8() << 3)
    }

    pub fn bytes(&self) -> impl Iterator<Item=u8> {
        once(self.scf())
            .chain(self.fcntr.to_be_bytes())
            .chain(self.kid)
    }
}

impl Debug for SecInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        if self.enc {
            write!(f, "ENC ")?;
        }

        write!(f, "FCNTR={} KIM={:?}",self.fcntr, self.kim)?;

        if let Some(kid) = self.kid {
            write!(f, " KID=0x{:02X}",kid)?;
        }

        write!(f, " MIC={:?}",self.mic)?;
        write!(f, "}}")
    }
}


#[derive(Clone, Eq, PartialEq)]
pub struct FrameInfo {
    pub frame_type: FrameType,
    pub ack_requested: bool,
    pub is_from_relay: bool,
    pub network_id: Option<NetworkId>,
    pub dst_addr: HamAddr,
    pub src_addr: HamAddr,
    pub rly_addr: Option<HamAddr>,
    pub sec_info: Option<SecInfo>,
    pub ack_crc: u16,
}

impl Debug for FrameInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{{{:?}", self.frame_type)?;

        if self.ack_requested {
            write!(f," AckReq")?;
        }

        if self.is_from_relay {
            write!(f," FromRly")?;
        }

        if let Some(netid) = self.network_id {
            write!(f," NetId={:?}", netid)?;
        }

        if !self.dst_addr.is_empty() {
            write!(f," Dst={}", self.dst_addr)?;
        }

        write!(f," Src={}", self.src_addr)?;

        if self.frame_type == FrameType::Ack {
            write!(f," AckCrc=0x{:04X}", self.ack_crc)?;
        }

        if let Some(rly_addr) = self.rly_addr {
            write!(f," Rly={}", rly_addr)?;
        }

        if let Some(secinfo) = &self.sec_info {
            write!(f," SecInfo={:?}", secinfo)?;
        }

        write!(f,"}}")
    }
}

impl FrameInfo {
    pub const EMPTY: FrameInfo = FrameInfo {
        frame_type: FrameType::Data,
        ack_requested: false,
        is_from_relay: false,
        network_id: None,
        dst_addr: HamAddr::EMPTY,
        src_addr: HamAddr::EMPTY,
        rly_addr: None,
        sec_info: None,
        ack_crc: 0,
    };

    pub fn ack_calc(&self, payload: &[u8]) -> Option<(u16,HamAddr)> {
        if self.ack_requested {
            let ack_sender = if let (Some(rly_addr),true) = (self.rly_addr, self.is_from_relay) {
                rly_addr
            } else {
                self.dst_addr
            };
            Some((self
                    .bytes_with_payload(payload)
                    .fold(X25.digest(), |mut digest, x| {
                        digest.update(&[x]);
                        digest
                    })
                    .finalize(),
                  ack_sender,
            ))
        } else {
            None
        }
    }

    pub fn generate_ack_frame(&self, payload: &[u8]) -> Option<FrameInfo> {
        if let Some((ack_crc, src_addr)) = self.ack_calc(payload) {
            Some(FrameInfo {
                frame_type: FrameType::Ack,
                src_addr,
                ack_crc,
                .. FrameInfo::EMPTY
            })
        } else {
            None
        }
    }
    
    pub fn try_from_bytes(frame: &[u8]) -> Result<(FrameInfo, &[u8]), Error> {
        if frame.len() < 5 {
            bail!("Frame too small");
        }
        let mut iter = frame.into_iter();

        let fcb_msb = iter.next().copied().ok_or_else(||format_err!("Frame too small"))?;
        let ver = fcb_msb >> 6;

        if ver != VERSION_EXPERIMENTAL && ver != VERSION_1 {
            bail!("Unexpected version {}", ver);
        }

        let dst_len = ((((fcb_msb & 0b1100) >> 2) + 1) * 2) as usize;
        let frame_type = FrameType::try_from_u8((fcb_msb & 0b00110000) >> 4).unwrap();

        let (
            has_security_header,
            has_netid,
            ack_requested,
            has_rly_addr,
            is_from_relay,
            rly_len,
            has_dst_addr,
        ) = if frame_type != FrameType::Ack {
            let lsb = iter.next().copied().ok_or_else(||format_err!("Frame too small"))?;

            (
                (lsb & 0b10000000) != 0,
                (lsb & 0b01000000) != 0,
                (lsb & 0b00100000) != 0,
                (lsb & 0b00010000) != 0,
                (lsb & 0b00001000) != 0,
                (((lsb & 0b11) + 1) * 2) as usize,
                true,
            )
        } else {
            (
                false,
                false,
                false,
                false,
                false,
                0,
                false,
            )
        };

        let network_id = if has_netid {
            Some(NetworkId::from_iter(&mut iter))
        } else {
            None
        };

        let dst_addr = if has_dst_addr {
            let dst_addr = HamAddr::try_from_slice(&iter.as_slice()[..dst_len])?;
            for _ in 0..dst_len {
                iter.next().unwrap();
            }
            dst_addr
        } else {
            HamAddr::EMPTY
        };

        let src_len = (((fcb_msb & 0b0011) + 1) * 2) as usize;
        let src_addr = HamAddr::try_from_slice(&iter.as_slice()[..src_len])?;
        for _ in 0..src_len {
            iter.next().unwrap();
        }

        let rly_addr = if has_rly_addr {
            let rly_addr = HamAddr::try_from_slice(&iter.as_slice()[..rly_len])?;
            for _ in 0..rly_len {
                iter.next().unwrap();
            }
            Some(rly_addr)
        } else {
            None
        };

        let ack_crc = if frame_type == FrameType::Ack {
            let msb = *iter.next().unwrap();
            let lsb = *iter.next().unwrap();
            ((msb as u16)<<8) + (lsb as u16)
        } else {
            0
        };

        let (sec_info, payload) = if has_security_header {
            let mut sec_info = SecInfo::from_iter(&mut iter);
            let payload_and_mic = iter.as_slice();
            let mic_len = sec_info.mic.len();
            let (payload, mic_slice) = payload_and_mic.split_at(payload_and_mic.len()-mic_len);

            sec_info.mic = Mic::try_from_slice(mic_slice).unwrap();

            (Some(sec_info),payload)
        } else {
            (None, iter.as_slice())
        };

        Ok((FrameInfo {
            frame_type,
            ack_requested,
            is_from_relay,
            network_id,
            dst_addr,
            src_addr,
            rly_addr,
            sec_info,
            ack_crc,
        }, payload))
    }

    pub fn fcf_msb(&self) -> u8 {
        let ver = VERSION_EXPERIMENTAL;
        (ver << 6)
            + (self.frame_type.to_u8() << 4)
            + ((u8::try_from(self.dst_addr.len()).unwrap() / 2u8 - 1) << 2)
            + (u8::try_from(self.src_addr.len()).unwrap() / 2u8 - 1)
    }

    pub fn fcf_lsb(&self) -> Option<u8> {
        if self.frame_type != FrameType::Ack {
            Some((u8::from(self.sec_info.is_some()) << 7)
                + (u8::from(self.network_id.is_some()) << 6)
                + (u8::from(self.ack_requested) << 5)
                + (u8::from(self.rly_addr.is_some()) << 4)
                + (u8::from(self.is_from_relay) << 3)
                + (u8::try_from(self.rly_addr.map(|x|x.len()).unwrap_or(2)).unwrap() / 2 - 1)
            )
        } else {
            None
        }
    }

    pub fn bytes_with_payload<'a>(&self, payload: &'a[u8]) -> impl Iterator<Item=u8> + 'a {
        let (dst_addr, network_id, rly_addr, sec_info, ack_crc) = if self.frame_type != FrameType::Ack {
            (Some(self.dst_addr),self.network_id,self.rly_addr, self.sec_info.clone(),None)
        } else {
            assert!(payload.is_empty());
            (None, None, None, None, Some(self.ack_crc))
        };

        once(self.fcf_msb())
            .chain(self.fcf_lsb())
            .chain(network_id.into_iter().flat_map(|x| x.0.to_be_bytes()))
            .chain(dst_addr.into_iter().flat_map(HamAddr::trimmed_bytes))
            .chain(self.src_addr.trimmed_bytes())
            .chain(rly_addr.into_iter().flat_map(HamAddr::trimmed_bytes))
            .chain(sec_info.clone().into_iter().flat_map(|x|x.bytes()))
            .chain(ack_crc.into_iter().flat_map(|x| x.to_be_bytes()))
            .chain(payload.iter().copied())
            .chain(sec_info.into_iter().flat_map(|x|x.mic.bytes()))
    }

    pub fn to_vec(&self, payload: &[u8]) -> Vec<u8> {
        let mut ret = Vec::new();
        ret.extend(self.bytes_with_payload(payload));
        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_info_1() {
        let frame = FrameInfo {
            frame_type: FrameType::Data,
            dst_addr: "X1X".parse().unwrap(),
            src_addr: "HUXLEY".parse().unwrap(),
            .. FrameInfo::EMPTY
        };
        let payload = b"Payload";

        println!("frame: {:?}", frame);
        let bytes = frame.bytes_with_payload(payload).collect::<Vec<_>>();

        let (decoded_frame, decoded_payload) = FrameInfo::try_from_bytes(&bytes).unwrap();

        assert_eq!(frame, decoded_frame);
        assert_eq!(payload, decoded_payload);
    }

    #[test]
    fn frame_info_2() {
        let frame = FrameInfo {
            frame_type: FrameType::Data,
            dst_addr: "X1X".parse().unwrap(),
            src_addr: "HUXLEY".parse().unwrap(),
            rly_addr: Some("RAD-RELAY".parse().unwrap()),
            sec_info: Some(SecInfo{
                enc: false,
                kim: KeyIdentMode::Addresses,
                fcntr: 0x31337,
                kid: None,
                mic: Default::default()
            }),
            .. FrameInfo::EMPTY
        };
        let payload = &[0xffu8,0x00,0x00,0x00,0xff];

        let bytes = frame.bytes_with_payload(payload).collect::<Vec<_>>();

        println!("frame: {:?}", frame);
        println!("frame.fcf: {:02x}{:02x}", frame.fcf_msb(),frame.fcf_lsb().unwrap());

        println!("bytes: {}", hex::encode(&bytes));
        let (decoded_frame, decoded_payload) = FrameInfo::try_from_bytes(&bytes).unwrap();

        println!("decoded_frame.fcf: {:02x}{:02x}", decoded_frame.fcf_msb(),decoded_frame.fcf_lsb().unwrap());

        assert_eq!(frame, decoded_frame);
        assert_eq!(payload, decoded_payload);
    }

    #[test]
    fn frame_info_3() {
        let frame = FrameInfo {
            frame_type: FrameType::Data,
            ack_requested: true,
            is_from_relay: true,
            network_id: Some(NetworkId(0x1234)),
            dst_addr: "X1X".parse().unwrap(),
            src_addr: "HUXLEY".parse().unwrap(),
            rly_addr: Some("RAD-RELAY".parse().unwrap()),
            sec_info: Some(SecInfo{
                enc: true,
                kim: KeyIdentMode::KeyIndex,
                fcntr: 0x31337,
                kid: Some(6),
                mic: Default::default()
            }),
            .. FrameInfo::EMPTY
        };
        let payload = &[0xffu8,0x00,0x00,0x00,0xff];

        let bytes = frame.bytes_with_payload(payload).collect::<Vec<_>>();

        println!("frame: {:?}", frame);
        println!("frame.fcf: {:02x}{:02x}", frame.fcf_msb(),frame.fcf_lsb().unwrap());

        println!("bytes: {}", hex::encode(&bytes));
        let (decoded_frame, decoded_payload) = FrameInfo::try_from_bytes(&bytes).unwrap();

        println!("decoded_frame.fcf: {:02x}{:02x}", decoded_frame.fcf_msb(),decoded_frame.fcf_lsb().unwrap());

        assert_eq!(frame, decoded_frame);
        assert_eq!(payload, decoded_payload);
    }

    #[test]
    fn frame_info_ack() {
        let frame = FrameInfo {
            frame_type: FrameType::Ack,
            src_addr: "HUXLEY".parse().unwrap(),
            ack_crc: 0xbeef,
            .. FrameInfo::EMPTY
        };
        let payload = &[];

        let bytes = frame.bytes_with_payload(payload).collect::<Vec<_>>();
        println!("frame: {:?}", frame);
        println!("frame.fcf: {:02x}XX", frame.fcf_msb());
        assert!(frame.fcf_lsb().is_none());

        println!("bytes: {}", hex::encode(&bytes));
        let (decoded_frame, decoded_payload) = FrameInfo::try_from_bytes(&bytes).unwrap();

        println!("decoded_frame.fcf: {:02x}XX", decoded_frame.fcf_msb());
        assert!(decoded_frame.fcf_lsb().is_none());

        assert_eq!(frame, decoded_frame);
        assert_eq!(payload, decoded_payload);
    }


    #[test]
    fn frame_test_vec_1() {
        let bytes = hex::decode("054013375CAC70F85CB626E8062839414D2D54414B002918FA9C").unwrap();

        let (decoded_frame, decoded_payload) = FrameInfo::try_from_bytes(&bytes).unwrap();

        let frame = FrameInfo {
            frame_type: FrameType::Beacon,
            network_id: Some(NetworkId(0x1337)),
            dst_addr: "N6DRC".parse().unwrap(),
            src_addr: "N6NFI".parse().unwrap(),
            .. FrameInfo::EMPTY
        };
        let payload = &[0x06u8,0x28,0x39,0x41,0x4D,0x2D,0x54,0x41,0x4B,0x00,0x29,0x18,0xFA,0x9C];

        assert_eq!(frame, decoded_frame);
        assert_eq!(payload, decoded_payload);
    }
}
