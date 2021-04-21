use std::error::Error as StdError;
use std::fmt;

#[derive(PartialEq, Eq)]
pub enum Error {
    IllegalChar,
    Overflow,
    NullComponent,
    MissingComponents,
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::IllegalChar => write!(f, "IllegalChar"),
            Self::Overflow => write!(f, "Overflow"),
            Self::NullComponent => write!(f, "NullComponent"),
            Self::MissingComponents => write!(f, "MissingComponents"),
        }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::IllegalChar => write!(f, "Illegal character (expected numerics)"),
            Self::Overflow => write!(f, "Too large an input for IP address"),
            Self::NullComponent => write!(f, "Empty component"),
            Self::MissingComponents => write!(f, "Missing components for IP address"),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match self {
            Self::IllegalChar => "Illegal character (expected numerics)",
            Self::Overflow => "Too large an input for IP address",
            Self::NullComponent => "Empty component",
            Self::MissingComponents => "Missing components for IP address",
        }
    }
    fn cause(&self) -> Option<&dyn StdError> {
        match self {
            Self::IllegalChar => None,
            Self::Overflow => None,
            Self::NullComponent => None,
            Self::MissingComponents => None,
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct AddrV4 {
    addr: u32,
}

impl AddrV4 {
    fn component_to_u8(s: &str) -> Result<u32, Error> {
        let mut res = 0;
        // empty component should be garbaged
        if s.len() == 0 {
            return Err(Error::NullComponent);
        }
        // strip preleading zeros
        let s = s.trim_start_matches('0');
        // overflow pre-verifications
        if s.len() > 3 {
            return Err(Error::Overflow);
        }
        // transform base10 string to int
        for ch in s.chars() {
            if ch < '0' || ch > '9' {
                return Err(Error::IllegalChar);
            }
            res = res * 10 + ch as u32 - '0' as u32;
        }
        // verify final result
        if res < 256 {
            Ok(res)
        } else {
            Err(Error::Overflow)
        }
    }

    pub fn from_u32(addr: u32) -> Result<Self, Error> {
        Ok(AddrV4 { addr })
    }

    pub fn from_hex(addr: &str) -> Result<Self, Error> {
        let mut cnt: i32 = 0;
        let mut irepr: u32 = 0;
        for ch in addr.chars() {
            let mut _cur = 0;
            if ch >= '0' && ch <= '9' {
                _cur = ch as u8 - '0' as u8;
            } else if ch >= 'a' && ch <= 'f' {
                _cur = ch as u8 - 'a' as u8 + 10;
            } else if ch >= 'A' && ch <= 'F' {
                _cur = ch as u8 - 'A' as u8 + 10;
            } else {
                continue;
            }
            if cnt >= 8 {
                return Err(Error::Overflow);
            }
            irepr = (irepr << 4) | _cur as u32;
            cnt += 1;
        }
        if cnt == 8 {
            Ok(AddrV4 { addr: irepr })
        } else {
            Err(Error::MissingComponents)
        }
    }

    pub fn from_str(addr: &str) -> Result<Self, Error> {
        let mut buffer = String::from("");
        let mut res = 0;
        let mut cnt = 0;
        // parse first 3 components
        for ch in addr.chars() {
            if ch == '.' {
                if cnt >= 4 {
                    return Err(Error::Overflow);
                }
                res = (res << 8) | Self::component_to_u8(&buffer)?;
                cnt += 1;
                buffer.clear();
            } else {
                buffer.push(ch);
            }
        }
        // append last component, if any
        if cnt >= 4 {
            return Err(Error::Overflow);
        }
        res = (res << 8) | Self::component_to_u8(&buffer)?;
        cnt += 1;
        // check for count mismatch
        if cnt == 4 {
            Ok(AddrV4 { addr: res })
        } else {
            Err(Error::MissingComponents)
        }
    }
}

impl fmt::Debug for AddrV4 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "AddrV4({}.{}.{}.{})",
            (self.addr >> 24) & 0xff,
            (self.addr >> 16) & 0xff,
            (self.addr >> 8) & 0xff,
            self.addr & 0xff
        ))?;
        Ok(())
    }
}

impl fmt::Display for AddrV4 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "{}.{}.{}.{}",
            (self.addr >> 24) & 0xff,
            (self.addr >> 16) & 0xff,
            (self.addr >> 8) & 0xff,
            self.addr & 0xff
        ))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::addr::AddrV4;
    use crate::addr::Error;

    #[test]
    fn v4_from_u32() {
        assert_eq!(
            AddrV4::from_u32(0x0000000).unwrap(),
            AddrV4 { addr: 0x00000000 }
        );
    }

    #[test]
    fn v4_str_null() {
        assert_eq!(
            AddrV4::from_str("0.0.0.0").unwrap(),
            AddrV4 { addr: 0x00000000 }
        );
    }

    #[test]
    fn v4_str_loopback() {
        assert_eq!(
            AddrV4::from_str("127.0.0.1").unwrap(),
            AddrV4 { addr: 0x7f000001 }
        );
    }

    #[test]
    fn v4_str_broadcast() {
        assert_eq!(
            AddrV4::from_str("255.255.255.255").unwrap(),
            AddrV4 { addr: 0xffffffff }
        );
    }

    #[test]
    fn v4_str_typec() {
        assert_eq!(
            AddrV4::from_str("192.168.1.2").unwrap(),
            AddrV4 { addr: 0xc0a80102 }
        );
    }

    #[test]
    fn v4_str_subnet_mask() {
        assert_eq!(
            AddrV4::from_str("255.255.255.0").unwrap(),
            AddrV4 { addr: 0xffffff00 }
        );
    }

    #[test]
    fn v4_str_allow_leading_zero() {
        assert_eq!(
            AddrV4::from_str("000000.000001.000010.000100").unwrap(),
            AddrV4 { addr: 0x00010a64 }
        );
    }

    #[test]
    fn v4_str_fail_illegal_char_too_long() {
        assert_eq!(
            AddrV4::from_str("-192.168.0.1").unwrap_err(),
            Error::Overflow
        );
    }

    #[test]
    fn v4_str_fail_illegal_char() {
        assert_eq!(
            AddrV4::from_str("-92.168.0.1").unwrap_err(),
            Error::IllegalChar
        );
    }

    #[test]
    fn v4_str_fail_overflow_comp_1() {
        assert_eq!(AddrV4::from_str("256.0.0.0").unwrap_err(), Error::Overflow);
    }

    #[test]
    fn v4_str_fail_overflow_comp_2() {
        assert_eq!(AddrV4::from_str("0.0.0.256").unwrap_err(), Error::Overflow);
    }

    #[test]
    fn v4_str_fail_missing_comp() {
        assert_eq!(
            AddrV4::from_str("127.0.0").unwrap_err(),
            Error::MissingComponents
        );
    }

    #[test]
    fn v4_str_fail_too_many_comp() {
        assert_eq!(
            AddrV4::from_str("127.0.0.1.2").unwrap_err(),
            Error::Overflow
        );
    }

    #[test]
    fn v4_str_fail_excessive_dot() {
        assert_eq!(AddrV4::from_str("127.0.0.1.").unwrap_err(), Error::Overflow);
    }

    #[test]
    fn v4_str_fail_null_comp() {
        assert_eq!(
            AddrV4::from_str("127..0.0.1").unwrap_err(),
            Error::NullComponent
        );
    }

    #[test]
    fn v4_hex_null() {
        assert_eq!(
            AddrV4::from_hex("00000000").unwrap(),
            AddrV4 { addr: 0x00000000 }
        );
    }

    #[test]
    fn v4_hex_loopback() {
        assert_eq!(
            AddrV4::from_hex("7f000001").unwrap(),
            AddrV4 { addr: 0x7f000001 }
        );
    }

    #[test]
    fn v4_hex_intranet() {
        assert_eq!(
            AddrV4::from_hex("c0a80102").unwrap(),
            AddrV4 { addr: 0xc0a80102 }
        );
    }

    #[test]
    fn v4_hex_intranet_colon() {
        assert_eq!(
            AddrV4::from_hex("c0:a8:01:02").unwrap(),
            AddrV4 { addr: 0xc0a80102 }
        );
    }

    #[test]
    fn v4_hex_intranet_dash() {
        assert_eq!(
            AddrV4::from_hex("c0-a8-01-02").unwrap(),
            AddrV4 { addr: 0xc0a80102 }
        );
    }

    #[test]
    fn v4_hex_ignore_other() {
        assert_eq!(
            AddrV4::from_hex("cG0HaI8J0K1L0M2").unwrap(),
            AddrV4 { addr: 0xc0a80102 }
        );
    }

    #[test]
    fn v4_hex_broadcast() {
        assert_eq!(
            AddrV4::from_hex("ffffffff").unwrap(),
            AddrV4 { addr: 0xffffffff }
        );
    }

    #[test]
    fn v4_hex_fail_overflow() {
        assert_eq!(AddrV4::from_hex("012345678").unwrap_err(), Error::Overflow);
    }

    #[test]
    fn v4_hex_fail_missing() {
        assert_eq!(
            AddrV4::from_hex("0123456").unwrap_err(),
            Error::MissingComponents
        );
    }
}
