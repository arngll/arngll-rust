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

mod error;
mod eui;
mod ham_addr;
mod ham_char;

pub use crate::error::*;
pub use crate::eui::*;
pub use crate::ham_addr::*;
pub use crate::ham_char::*;

#[cfg(test)]
mod ham_addr_tests {
    use super::*;

    #[test]
    fn test_ham_addr_parse_callsign() {
        let addr: HamAddr = "KZ2X-1".parse().unwrap();
        assert_eq!(addr.to_string(), "KZ2X-1");

        let addr: HamAddr = "".parse().unwrap();
        assert_eq!(addr.to_string(), "~");

        let addr: HamAddr = "~".parse().unwrap();
        assert_eq!(addr.to_string(), "~");

        let addr: HamAddr = "~ffff".parse().unwrap();
        assert_eq!(addr.to_string(), "~FFFF");

        let addr: HamAddr = "~FFFF".parse().unwrap();
        assert_eq!(addr.to_string(), "~FFFF");
    }

    #[test]
    fn test_ham_addr_to_hex_string() {
        let addr: HamAddr = "KZ2X-1".parse().unwrap();
        assert_eq!(format!("{:?}", addr), "48ED-9C0C");

        let addr: HamAddr = "N6DRC".parse().unwrap();
        assert_eq!(format!("{:?}", addr), "5CAC-70F8");
        assert_eq!(format!("{:#?}", addr), "5CAC-70F8-0000-0000");

        let addr: HamAddr = "VI2BMARC50".parse().unwrap();
        assert_eq!(format!("{:?}", addr), "8B05-0E89-7118-A8C0");

        let addr: HamAddr = "KJ6QOH/P".parse().unwrap();
        assert_eq!(format!("{:#?}", addr), "4671-6CA0-E9C0-0000");
    }

    #[test]
    fn test_ham_addr_to_eui64_12_char_hack() {
        let addr = "KJ6QOH-23".parse::<HamAddr>().unwrap();
        let eui64: Eui64 = addr.try_into().unwrap();
        assert_eq!(eui64.to_string(), "22:46:71:ff:fe:6c:a0:f2");
        assert_eq!(HamAddr::try_from(eui64).unwrap(), addr);

        let eui64 = Eui64([0x02, 0x46, 0x71, 0x6C, 0xA0, 0xF2, 0x20, 0x00]);
        assert_eq!(HamAddr::try_from(eui64).unwrap().to_string(), "KJ6QOH-2X");

        let addr = "KJ6QOH-99".parse::<HamAddr>().unwrap();
        let eui64: Eui64 = addr.try_into().unwrap();
        assert_eq!(eui64.to_string(), "02:46:71:6c:a0:f3:44:00");
        assert_eq!(HamAddr::try_from(eui64).unwrap(), addr);

        let addr = "VI2BMARC50-X".parse::<HamAddr>().unwrap();
        assert!(Eui64::try_from(addr).is_err());
        assert!(Eui48::try_from(addr).is_err());

        let addr = "VI2BMARC50-1".parse::<HamAddr>().unwrap();
        assert!(Eui48::try_from(addr).is_err());
        let eui64: Eui64 = addr.try_into().unwrap();
        assert_eq!(eui64.to_string(), "ba:8b:05:0e:89:71:18:ae");
        assert_eq!(HamAddr::try_from(eui64).unwrap(), addr);
    }

    #[test]
    fn test_ham_addr_to_eui48_9_char_hack() {
        let addr = "KJ6QOH-23".parse::<HamAddr>().unwrap();
        let eui48: Eui48 = addr.try_into().unwrap();
        assert_eq!(eui48.to_string(), "22:46:71:6c:a0:f2");
        assert_eq!(HamAddr::try_from(eui48).unwrap(), addr);

        let addr = "KJ6QOH-2X".parse::<HamAddr>().unwrap();
        let eui48_result = Eui48::try_from(addr);
        assert!(
            eui48_result.is_err(),
            "KJ6QOH-2X parsed to EUI48: {}",
            eui48_result.unwrap()
        );

        let addr = "KJ6QOH-99".parse::<HamAddr>().unwrap();
        let eui48_result = Eui48::try_from(addr);
        assert!(
            eui48_result.is_err(),
            "KJ6QOH-99 parsed to EUI48: {}",
            eui48_result.unwrap()
        );
    }

    #[test]
    fn test_ham_addr_to_eui64() {
        let addr = "KZ2X-1".parse::<HamAddr>().unwrap();
        let eui64: Eui64 = addr.try_into().unwrap();
        assert_eq!(eui64.to_string(), "02:48:ed:ff:fe:9c:0c:00");
        let addr: HamAddr = eui64.try_into().unwrap();
        assert_eq!(addr.to_string(), "KZ2X-1");
        let eui64: Eui64 = addr.try_into().unwrap();
        assert_eq!(eui64.to_string(), "02:48:ed:ff:fe:9c:0c:00");

        let addr = "AC2OI".parse::<HamAddr>().unwrap();
        let eui64: Eui64 = addr.try_into().unwrap();
        assert_eq!(eui64.to_string(), "02:06:d5:ff:fe:5f:28:00");

        let addr = "WB3KUZ-111".parse::<HamAddr>().unwrap();
        let eui64: Eui64 = addr.try_into().unwrap();
        assert_eq!(eui64.to_string(), "02:90:2e:48:22:f1:fc:af");

        let addr = "VI2BMARC50".parse::<HamAddr>().unwrap();
        let eui64: Eui64 = addr.try_into().unwrap();
        assert_eq!(eui64.to_string(), "c2:8b:05:0e:89:71:18:a8");
        let addr: HamAddr = eui64.try_into().unwrap();
        assert_eq!(addr.to_string(), "VI2BMARC50");
        let eui64: Eui64 = addr.try_into().unwrap();
        assert_eq!(eui64.to_string(), "c2:8b:05:0e:89:71:18:a8");

        let addr = HamAddr::BROADCAST;
        let eui64: Eui64 = addr.try_into().unwrap();
        assert_eq!(eui64, Eui64::BROADCAST);

        let addr = HamAddr::EMPTY;
        let eui64: Eui64 = addr.try_into().unwrap();
        assert_eq!(eui64, Eui64::EMPTY);
    }

    #[test]
    fn test_ham_addr_to_eui48() {
        let addr = "KZ2X-1".parse::<HamAddr>().unwrap();
        let eui48: Eui48 = addr.try_into().unwrap();
        assert_eq!(eui48.to_string(), "02:48:ed:9c:0c:00");

        let addr: HamAddr = eui48.try_into().unwrap();
        assert_eq!(addr.to_string(), "KZ2X-1");

        let addr = "AC2OI".parse::<HamAddr>().unwrap();
        let eui48: Eui48 = addr.try_into().unwrap();
        assert_eq!(eui48.to_string(), "02:06:d5:5f:28:00");

        let addr = "WB3KUZ-1".parse::<HamAddr>().unwrap();
        let eui48: Eui48 = addr.try_into().unwrap();
        assert_eq!(eui48.to_string(), "e2:90:2e:48:22:f1");
        let addr: HamAddr = eui48.try_into().unwrap();
        assert_eq!(addr.to_string(), "WB3KUZ-1");

        let addr = "NA1SS".parse::<HamAddr>().unwrap();
        let eui48: Eui48 = addr.try_into().unwrap();
        assert_eq!(eui48.to_string(), "02:57:c4:79:b8:00");
        let addr: HamAddr = eui48.try_into().unwrap();
        assert_eq!(addr.to_string(), "NA1SS");

        let addr = "VI2BMARC50".parse::<HamAddr>().unwrap();
        let not_eui48: Result<Eui48> = addr.try_into();
        assert!(matches!(not_eui48, Err(_)));

        let addr = HamAddr::BROADCAST;
        let eui48: Eui48 = addr.try_into().unwrap();
        assert_eq!(eui48, Eui48::BROADCAST);

        let addr = HamAddr::EMPTY;
        let eui48: Eui48 = addr.try_into().unwrap();
        assert_eq!(eui48, Eui48::EMPTY);

        let addr = HamAddr::from_chunks([0xFAFB, 0, 0, 0]);
        let eui48: Eui48 = addr.try_into().unwrap();
        assert_eq!(eui48.to_string(), "cc:cc:00:00:00:fb");

        let addr = HamAddr::from_chunks([0xFBFB, 0, 0, 0]);
        let eui48: Eui48 = addr.try_into().unwrap();
        assert_eq!(eui48.to_string(), "01:00:5e:00:00:fb");
    }
}
