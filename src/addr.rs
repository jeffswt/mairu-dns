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

    pub fn from_string(addr: &str) -> Result<Self, Error> {
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

    pub fn to_string(&self) -> String {
        format!(
            "{}.{}.{}.{}",
            (self.addr >> 24) & 0xff,
            (self.addr >> 16) & 0xff,
            (self.addr >> 8) & 0xff,
            self.addr & 0xff
        )
    }
}

impl fmt::Debug for AddrV4 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("AddrV4({})", self.to_string()))
    }
}

#[cfg(test)]
mod tests_v4_u32 {
    use crate::addr::AddrV4;

    #[test]
    fn null() {
        assert_eq!(
            AddrV4::from_u32(0x0000000).unwrap(),
            AddrV4 { addr: 0x00000000 }
        );
    }
}

#[cfg(test)]
mod tests_v4_str_ok {
    use crate::addr::AddrV4;

    fn expect(origin: &str, target: u32) {
        assert_eq!(
            AddrV4::from_string(origin).unwrap(),
            AddrV4 { addr: target }
        );
    }

    #[test]
    fn null() {
        expect("0.0.0.0", 0x00000000);
    }

    #[test]
    fn loopback() {
        expect("127.0.0.1", 0x7f000001);
    }

    #[test]
    fn broadcast() {
        expect("255.255.255.255", 0xffffffff);
    }

    #[test]
    fn typec() {
        expect("192.168.1.2", 0xc0a80102);
    }

    #[test]
    fn subnet_mask() {
        expect("255.255.255.0", 0xffffff00);
    }

    #[test]
    fn allow_leading_zero() {
        expect("000000.000001.000010.000100", 0x00010a64);
    }
}

#[cfg(test)]
mod tests_v4_str_fail {
    use crate::addr::AddrV4;
    use crate::addr::Error;

    fn expect(origin: &str, target: Error) {
        assert_eq!(AddrV4::from_string(origin).unwrap_err(), target);
    }

    #[test]
    fn illegal_char_too_long() {
        expect("-192.168.0.1", Error::Overflow);
    }

    #[test]
    fn illegal_char() {
        expect("-92.168.0.1", Error::IllegalChar);
    }

    #[test]
    fn overflow_comp_1() {
        expect("256.0.0.0", Error::Overflow);
    }

    #[test]
    fn overflow_comp_2() {
        expect("0.0.0.256", Error::Overflow);
    }

    #[test]
    fn missing_comp() {
        expect("127.0.0", Error::MissingComponents);
    }

    #[test]
    fn too_many_comp() {
        expect("127.0.0.1.2", Error::Overflow);
    }

    #[test]
    fn excessive_dot() {
        expect("127.0.0.1.", Error::Overflow);
    }

    #[test]
    fn null_comp() {
        expect("127..0.0.1", Error::NullComponent);
    }
}

#[cfg(test)]
mod tests_v4_hex_ok {
    use crate::addr::AddrV4;

    fn expect(origin: &str, target: u32) {
        assert_eq!(AddrV4::from_hex(origin).unwrap(), AddrV4 { addr: target });
    }

    #[test]
    fn null() {
        expect("00000000", 0x00000000);
    }

    #[test]
    fn loopback() {
        expect("7f000001", 0x7f000001);
    }

    #[test]
    fn intranet() {
        expect("c0a80102", 0xc0a80102);
    }

    #[test]
    fn intranet_colon() {
        expect("c0:a8:01:02", 0xc0a80102);
    }

    #[test]
    fn intranet_dash() {
        expect("c0-a8-01-02", 0xc0a80102);
    }

    #[test]
    fn ignore_other() {
        expect("cG0HaI8J0K1L0M2", 0xc0a80102);
    }

    #[test]
    fn broadcast() {
        expect("ffffffff", 0xffffffff);
    }
}

#[cfg(test)]
mod tests_v4_hex_fail {
    use crate::addr::AddrV4;
    use crate::addr::Error;

    fn expect(origin: &str, target: Error) {
        assert_eq!(AddrV4::from_hex(origin).unwrap_err(), target);
    }

    #[test]
    fn overflow() {
        expect("012345678", Error::Overflow);
    }

    #[test]
    fn missing() {
        expect("0123456", Error::MissingComponents);
    }
}

#[cfg(test)]
mod tests_v4_out {
    use crate::addr::AddrV4;

    fn expect_str(origin: &str, target: &str) {
        assert_eq!(
            AddrV4::from_string(origin).unwrap().to_string(),
            String::from(target)
        );
    }

    fn expect_hex(origin: &str, target: &str) {
        assert_eq!(
            AddrV4::from_hex(origin).unwrap().to_string(),
            String::from(target)
        );
    }

    #[test]
    fn str_null() {
        expect_str("0.0.0.0", "0.0.0.0");
    }

    #[test]
    fn str_loopback() {
        expect_str("00127.0.000.000000001", "127.0.0.1");
    }

    #[test]
    fn hex_broadcast() {
        expect_str("255.255.255.255", "255.255.255.255");
    }

    #[test]
    fn hex_subnet_mask_1() {
        expect_hex("ff:ff:ff:00", "255.255.255.0");
    }

    #[test]
    fn hex_subnet_mask_2() {
        expect_hex("ff-ff-00-00", "255.255.0.0");
    }
}
