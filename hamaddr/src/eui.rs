use std::fmt;

/// Eui48 represents an EUI48 MAC address.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub struct Eui48(pub [u8; 6]);

impl Eui48 {
    pub const EMPTY: Eui48 = Eui48([0, 0, 0, 0, 0, 0]);
    pub const BROADCAST: Eui48 = Eui48([0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
    /// Creates a new Eui48 from the given six octets.
    pub const fn new(addr: [u8; 6]) -> Eui48 {
        Eui48(addr)
    }
}

/// Formats an Eui48 for display.
impl fmt::Display for Eui48 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bs = &self.0;
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            bs[0], bs[1], bs[2], bs[3], bs[4], bs[5]
        )
    }
}

/// Eui64 represents an EUI64 MAC address.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Eui64(pub [u8; 8]);

impl Eui64 {
    pub const EMPTY: Eui64 = Eui64([0, 0, 0, 0, 0, 0, 0, 0]);
    pub const BROADCAST: Eui64 = Eui64([0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
    /// Creates a new Eui64 from the given 8 octets.
    pub const fn new(addr: [u8; 8]) -> Eui64 {
        Eui64(addr)
    }
}

/// Formats an Eui64 for display.
impl fmt::Display for Eui64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bs = &self.0;
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            bs[0], bs[1], bs[2], bs[3], bs[4], bs[5], bs[6], bs[7]
        )
    }
}

/// Conversion from an Eui48 to an Eui64 always succeeds.
impl From<&Eui48> for Eui64 {
    fn from(eui48: &Eui48) -> Self {
        let mut bytes = [0; 8];
        bytes[..3].copy_from_slice(&eui48.0[..3]);
        bytes[3] = 0xFF;
        bytes[4] = 0xFE; // See RFC 4291, App A.
        bytes[5..].copy_from_slice(&eui48.0[3..]);
        Eui64(bytes)
    }
}

#[cfg(test)]
mod eui_tests {
    use super::*;

    #[test]
    fn test_from_eiu48_for_eui64() {
        let eui48 = Eui48::new([1, 2, 3, 4, 5, 6]);
        let eui64 = Eui64::from(&eui48);
        assert_eq!(eui64.0, [1, 2, 3, 0xFF, 0xFE, 4, 5, 6]);
    }

    #[test]
    fn test_to_str_eui48() {
        let eui = Eui48::new([1, 2, 3, 4, 5, 6]);
        let s = eui.to_string();
        assert_eq!(s, "01:02:03:04:05:06");
    }

    #[test]
    fn test_to_str_eui64() {
        let eui = Eui64::new([1, 2, 3, 4, 5, 6, 0x77, 0x88]);
        let s = eui.to_string();
        assert_eq!(s, "01:02:03:04:05:06:77:88");
    }
}
