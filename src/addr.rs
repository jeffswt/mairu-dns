use std::error::Error as StdError;
use std::fmt;

#[derive(PartialEq, Eq)]
pub enum Error {
    IllegalChar,
    Overflow,
    NullComponent,
    MissingComponents,
    DoubleCompression,
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::IllegalChar => write!(f, "IllegalChar"),
            Self::Overflow => write!(f, "Overflow"),
            Self::NullComponent => write!(f, "NullComponent"),
            Self::MissingComponents => write!(f, "MissingComponents"),
            Self::DoubleCompression => write!(f, "DoubleCompression"),
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
            Self::DoubleCompression => write!(f, "Multiple '::' occurences"),
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
            Self::DoubleCompression => "Multiple '::' occurences",
        }
    }
    fn cause(&self) -> Option<&dyn StdError> {
        match self {
            Self::IllegalChar => None,
            Self::Overflow => None,
            Self::NullComponent => None,
            Self::MissingComponents => None,
            Self::DoubleCompression => None,
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
        Ok(Self { addr })
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
            Ok(Self { addr: irepr })
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
            Ok(Self { addr: res })
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

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct AddrV6 {
    addr: u128,
}

impl AddrV6 {
    pub fn hextet_to_u16(hextet: &str) -> Result<u16, Error> {
        let mut comp: u16 = 0;
        // length-related issues
        // it should be noted that Error::NullComponent would never be raised
        //     because `from_string` won't let me
        if hextet.len() > 4 {
            return Err(Error::Overflow);
        }
        // iterate chars and converto to int
        for ch in hextet.chars() {
            let mut _cur = 0;
            match ch {
                '0'..='9' => _cur = ch as u8 - '0' as u8,
                'a'..='f' => _cur = ch as u8 - 'a' as u8 + 10,
                'A'..='F' => _cur = ch as u8 - 'A' as u8 + 10,
                _ => return Err(Error::IllegalChar),
            }
            comp = (comp << 4) | _cur as u16;
        }
        Ok(comp)
    }
    pub fn from_u128(addr: u128) -> Result<Self, Error> {
        Ok(Self { addr })
    }
    pub fn from_string(addr: &str) -> Result<Self, Error> {
        // ensure that no two '::' appears and split into prefix and suffix
        let mut parts: Vec<Vec<String>> = String::from(addr)
            .split("::")
            .map(|part| {
                String::from(part)
                    .split(":")
                    .map(|s| String::from(s))
                    .filter(|s| s.len() > 0)
                    .collect()
            })
            .collect();
        // part length verdict
        let mut prefix = vec![];
        let mut suffix = vec![];
        match parts.len() {
            0 => return Err(Error::MissingComponents),
            1 => {
                suffix.append(&mut parts[0]);
                if suffix.len() < 8 {
                    return Err(Error::MissingComponents);
                } else if suffix.len() > 8 {
                    return Err(Error::Overflow);
                }
            }
            2 => {
                prefix.append(&mut parts[0]);
                suffix.append(&mut parts[1]);
                // rfc4291, section 2.2, 2.
                //     In order to make writing addresses containing zero bits
                //     easier, a special syntax is available to compress the
                //     zeros. The use of "::" indicates one or more groups of
                //     16 bits of zeros. The "::" can only appear once in an
                //     address.
                // compression of ':0:' into '::' is allowed as input
                // however compression of ': :' into '::' is never allowed
                // considering the semantics of '::', overflow is raised as it
                //     will imply at least a ':0:' component
                if prefix.len() + suffix.len() >= 8 {
                    return Err(Error::Overflow);
                }
            }
            _ => return Err(Error::DoubleCompression),
        }
        // convert prefix and suffix
        let mut prefix_i: u128 = 0;
        let mut suffix_i: u128 = 0;
        for s in &prefix {
            let cur = Self::hextet_to_u16(&s)?;
            prefix_i = (prefix_i << 16) | cur as u128;
        }
        for s in &suffix {
            let cur = Self::hextet_to_u16(&s)?;
            suffix_i = (suffix_i << 16) | cur as u128;
        }
        if prefix.len() > 0 {
            suffix_i |= prefix_i << 16 * (8 - prefix.len());
        }
        Ok(Self { addr: suffix_i })
    }
    pub fn to_string(&self) -> String {
        // rfc5952, section 4:
        //     The recommendation in this section SHOULD be followed by systems
        //     when generating an address to be represented as text, but all
        //     implementations MUST accept and be able to handle any legitimate
        //     [RFC4291] format.
        if self.addr == 0 {
            return String::from("::");
        }
        // split u128 into hextets
        let mut hextets = [0; 8];
        let mut dp = [0; 8];
        for i in 0..8 {
            hextets[i] = (self.addr >> 16 * (7 - i)) & 0xffff;
            dp[i] = if hextets[i] == 0 {
                (if i > 0 { dp[i - 1] } else { 0 }) + 1
            } else {
                0
            };
        }
        // rfc5952, section 4.2.1., Shorten as Much as Possible
        //     The use of the symbol "::" MUST be used to its maximum
        //     capability. For example, 2001:db8:0:0:0:0:2:1 must be shortened
        //     to 2001:db8::2:1. Likewise, 2001:db8::0:1 is not acceptable,
        //     because the symbol "::" could have been used to produce a
        //     shorter representation 2001:db8::1.
        // rfc5952, section 4.2.3., Choice in Placement of "::"
        //     When there is an alternative choice in the placement of a "::",
        //     the longest run of consecutive 16-bit 0 fields MUST be shortened
        //     (i.e., the sequence with three consecutive zero fields is
        //     shortened in 2001:0:0:1:0:0:0:1).  When the length of the
        //     consecutive 16-bit 0 fields are equal (i.e.,
        //     2001:db8:0:0:1:0:0:1), the first sequence of zero bits MUST be
        //     shortened.  For example, 2001:db8::1:0:0:1 is correct
        //     representation.
        // locate unique compress position
        let mut max_len = 0;
        let mut max_pos = 0;
        for i in 0..8 {
            if dp[i] > max_len {
                max_len = dp[i];
                max_pos = i as i32;
            }
        }
        // rfc5952, section 4.2.2., Handling One 16-Bit 0 Field
        //     The symbol "::" MUST NOT be used to shorten just one 16-bit 0
        //     field. For example, the representation 2001:db8:0:1:1:1:1:1 is
        //     correct, but 2001:db8::1:1:1:1:1 is not correct.
        // rfc5952, section 4.3., Lowercase
        //     The characters "a", "b", "c", "d", "e", and "f" in an IPv6
        //     address MUST be represented in lowercase.
        // perform compression, join and leave
        if max_len > 1 {
            let idx_to_str = |&i| format!("{:x}", hextets[i as usize]);
            let pre_i: Vec<i32> = (0..=max_pos - max_len).collect();
            let suf_i: Vec<i32> = (max_pos + 1..8).collect();
            println!("error: {:?}, {}, {}", pre_i, max_pos, max_len);
            let mut prefix: Vec<String> = pre_i.iter().map(idx_to_str).collect();
            prefix.push(String::default());
            prefix.append(&mut suf_i.iter().map(idx_to_str).collect());
            // fix prefix / suffix compression problems
            let len = prefix.len();
            if prefix[0] == "" {
                prefix[0].push(':');
            } else if prefix[len - 1] == "" {
                prefix[len - 1].push(':');
            }
            prefix
        } else {
            hextets.iter().map(|v| format!("{:x}", v)).collect()
        }
        .join(":")
    }
}

impl fmt::Debug for AddrV6 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("AddrV6({})", self.to_string()))
    }
}

impl fmt::Display for AddrV6 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.to_string())
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

#[cfg(test)]
mod tests_v6_ok {
    use crate::addr::AddrV6;

    fn expect(origin: &str, target: u128) {
        assert_eq!(
            AddrV6::from_string(origin).unwrap(),
            AddrV6 { addr: target }
        );
    }

    #[test]
    fn rfc5952_intro_full() {
        expect(
            "2001:0db8:0000:0000:0001:0000:0000:0001",
            0x2001_0db8_0000_0000_0001_0000_0000_0001,
        );
    }

    #[test]
    fn rfc5952_intro_no_prefix_zero() {
        expect(
            "2001:db8:0:0:1:0:0:1",
            0x2001_0db8_0000_0000_0001_0000_0000_0001,
        );
    }

    #[test]
    fn rfc5952_intro_some_prefix_zero() {
        expect(
            "2001:0db8:0:0:1:0:0:1",
            0x2001_0db8_0000_0000_0001_0000_0000_0001,
        );
    }

    #[test]
    fn rfc5952_intro_simplified() {
        expect(
            "2001:db8::1:0:0:1",
            0x2001_0db8_0000_0000_0001_0000_0000_0001,
        );
    }

    #[test]
    fn rfc5952_intro_partial_simplified() {
        expect(
            "2001:db8::0:1:0:0:1",
            0x2001_0db8_0000_0000_0001_0000_0000_0001,
        );
    }

    #[test]
    fn rfc5952_intro_simplified_with_prefix_zero() {
        expect(
            "2001:0db8::1:0:0:1",
            0x2001_0db8_0000_0000_0001_0000_0000_0001,
        );
    }

    #[test]
    fn rfc5952_intro_alt_simplified() {
        expect(
            "2001:db8:0:0:1::1",
            0x2001_0db8_0000_0000_0001_0000_0000_0001,
        );
    }

    #[test]
    fn rfc5952_intro_alt_simplified_with_prefix_zero() {
        expect(
            "2001:db8:0000:0:1::1",
            0x2001_0db8_0000_0000_0001_0000_0000_0001,
        );
    }

    #[test]
    fn rfc5952_intro_alt_simplified_caps() {
        expect(
            "2001:DB8:0:0:1::1",
            0x2001_0db8_0000_0000_0001_0000_0000_0001,
        );
    }

    #[test]
    fn rfc_4291_2_2_uncomp_unicast() {
        expect(
            "2001:DB8:0:0:8:800:200C:417A",
            0x2001_0db8_0000_0000_0008_0800_200c_417a,
        );
    }

    #[test]
    fn rfc_4291_2_2_uncomp_multicast() {
        expect(
            "FF01:0:0:0:0:0:0:101",
            0xff01_0000_0000_0000_0000_0000_0000_0101,
        );
    }

    #[test]
    fn rfc_4291_2_2_uncomp_loopback() {
        expect("0:0:0:0:0:0:0:1", 0x0000_0000_0000_0000_0000_0000_0000_0001);
    }

    #[test]
    fn rfc_4291_2_2_uncomp_unspecified() {
        expect("0:0:0:0:0:0:0:0", 0x0000_0000_0000_0000_0000_0000_0000_0000);
    }

    #[test]
    fn rfc_4291_2_2_comp_unicast() {
        expect(
            "2001:db8::8:800:200c:417a",
            0x2001_0db8_0000_0000_0008_0800_200c_417a,
        );
    }

    #[test]
    fn rfc_4291_2_2_comp_multicast() {
        expect("ff01::101", 0xff01_0000_0000_0000_0000_0000_0000_0101);
    }

    #[test]
    fn rfc_4291_2_2_comp_loopback() {
        expect("::1", 0x0000_0000_0000_0000_0000_0000_0000_0001);
    }

    #[test]
    fn rfc_4291_2_2_comp_unspecified() {
        expect("::", 0x0000_0000_0000_0000_0000_0000_0000_0000);
    }

    #[test]
    fn allow_lower() {
        expect(
            "0001:0002:000a:000b:000c:000d:000e:000f",
            0x0001_0002_000a_000b_000c_000d_000e_000f,
        );
    }

    #[test]
    fn allow_caps() {
        expect(
            "0001:0002:000A:000B:000C:000D:000E:000F",
            0x0001_0002_000a_000b_000c_000d_000e_000f,
        );
    }

    #[test]
    fn allow_useless_comp_front() {
        expect(
            "::0002:0003:0004:0005:0006:0007:0008",
            0x0000_0002_0003_0004_0005_0006_0007_0008,
        );
    }

    #[test]
    fn allow_useless_comp_middle() {
        expect(
            "0001:0002:0003:0004::0006:0007:0008",
            0x0001_0002_0003_0004_0000_0006_0007_0008,
        );
    }
    #[test]
    fn allow_useless_comp_end() {
        expect(
            "0001:0002:0003:0004:0005:0006:0007::",
            0x0001_0002_0003_0004_0005_0006_0007_0000,
        );
    }

    #[test]
    fn compress_end() {
        expect("1::", 0x0001_0000_0000_0000_0000_0000_0000_0000);
    }

    #[test]
    fn compress_one_front() {
        expect("::2:3:4:5:6:7:8", 0x0000_0002_0003_0004_0005_0006_0007_0008);
    }

    #[test]
    fn compress_one_middle() {
        expect("1:2:3::5:6:7:8", 0x0001_0002_0003_0000_0005_0006_0007_0008);
    }

    #[test]
    fn compress_one_end() {
        expect("1:2:3:4:5:6:7::", 0x0001_0002_0003_0004_0005_0006_0007_0000);
    }
}

#[cfg(test)]
mod tests_v6_fail {
    use crate::addr::AddrV6;
    use crate::addr::Error;

    fn expect(origin: &str, target: Error) {
        assert_eq!(AddrV6::from_string(origin).unwrap_err(), target);
    }

    #[test]
    fn overflow() {
        expect(
            "0001:0002:0003:0004:0005:0006:0007:0008:0009",
            Error::Overflow,
        );
    }

    #[test]
    fn compressing_nothing_front() {
        expect("::0001:0002:0003:0004:0005:0006:0007:0008", Error::Overflow);
    }

    #[test]
    fn compressing_nothing_middle() {
        expect("0001:0002:0003:0004::0005:0006:0007:0008", Error::Overflow);
    }
    #[test]
    fn compressing_nothing_end() {
        expect("0001:0002:0003:0004:0005:0006:0007:0008::", Error::Overflow);
    }

    #[test]
    fn component_too_large_just_a_zero() {
        expect("01234::", Error::Overflow);
    }

    #[test]
    fn component_too_large_really() {
        expect("1234f::", Error::Overflow);
    }

    #[test]
    fn illegal_char() {
        expect("123g::", Error::IllegalChar);
    }

    #[test]
    fn illegal_char_space() {
        expect("123: 456::", Error::IllegalChar);
    }

    #[test]
    fn double_compression() {
        expect("1::2::3", Error::DoubleCompression);
    }

    #[test]
    fn double_compression_but_inferrable() {
        expect("1::2::3:4:5:6:7:8", Error::DoubleCompression);
    }

    #[test]
    fn missing_components() {
        expect("1:2:3:4:5:6:7", Error::MissingComponents);
    }
}

#[cfg(test)]
mod tests_v6_out {
    use crate::addr::AddrV6;

    fn expect(origin: u128, target: &str) {
        assert_eq!(AddrV6 { addr: origin }.to_string(), String::from(target));
    }

    #[test]
    fn empty() {
        expect(0x0000_0000_0000_0000_0000_0000_0000_0000, "::");
    }

    #[test]
    fn full() {
        expect(
            0x111a_22ab_33bc_44cd_55de_66ef_777f_8888,
            "111a:22ab:33bc:44cd:55de:66ef:777f:8888",
        );
    }

    #[test]
    fn no_prefix_zero() {
        expect(
            0x1111_022a_003b_0004_0000_5d00_0e60_07f7,
            "1111:22a:3b:4:0:5d00:e60:7f7",
        );
    }

    #[test]
    fn leading_zero() {
        expect(0x0000_0000_0000_0000_0000_0000_0000_1234, "::1234");
    }

    #[test]
    fn trailing_zero() {
        expect(0x4321_0000_0000_0000_0000_0000_0000_0000, "4321::");
    }

    #[test]
    fn middle_zero() {
        expect(0x001a_0000_0000_0000_0000_0000_0000_b400, "1a::b400");
    }

    #[test]
    fn typical_addr() {
        expect(0x9231_0db8_0000_0000_0000_0000_0000_0001, "9231:db8::1");
    }

    #[test]
    fn compress_longest_former() {
        expect(0x0000_0000_0000_0000_1234_0000_0000_abcd, "::1234:0:0:abcd");
    }

    #[test]
    fn compress_longest_latter() {
        expect(0x0000_0000_1234_0000_0000_0000_0000_0abc, "0:0:1234::abc");
    }

    #[test]
    fn compress_equal_first() {
        expect(
            0x9231_0000_0000_0db8_0acd_0000_0000_0192,
            "9231::db8:acd:0:0:192",
        );
    }

    #[test]
    fn all_zero_but_last() {
        expect(0x0000_0000_0000_0000_0000_0000_0000_0001, "::1");
    }

    #[test]
    fn all_zero_but_first() {
        expect(0x00a_0000_0000_0000_0000_0000_0000_0000, "a::");
    }

    #[test]
    fn no_shorten_single() {
        expect(0x0001_0000_0001_0000_0001_0000_0001_0000, "1:0:1:0:1:0:1:0");
    }
}
