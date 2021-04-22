use std::error::Error as StdError;
use std::fmt;

#[derive(PartialEq, Eq)]
pub enum Error {
    EmptySubdomain,
    IllegalChar,
    UnexpectedHyphen,
    EmptyDomain,
    NotFullyQualified,
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::EmptySubdomain => write!(f, "EmptySubdomain"),
            Self::IllegalChar => write!(f, "IllegalChar"),
            Self::UnexpectedHyphen => write!(f, "UnexpectedHyphen"),
            Self::EmptyDomain => write!(f, "EmptyDomain"),
            Self::NotFullyQualified => write!(f, "NotFullyQualified"),
        }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::EmptySubdomain => write!(f, "Empty subdomain (expected at least 1 char)"),
            Self::IllegalChar => write!(f, "Illegal character (expected a-z, A-Z, 0-9, -)"),
            Self::UnexpectedHyphen => write!(f, "Unexpected hyphen (should never prefix)"),
            Self::EmptyDomain => write!(f, "Expected non-empty partial qualified domain name"),
            Self::NotFullyQualified => {
                write!(f, "Domain is not a FQDN (Fully qualified domain name)")
            }
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match self {
            Self::EmptySubdomain => "Empty subdomain (expected at least 1 char)",
            Self::IllegalChar => "Illegal character (expected a-z, A-Z, 0-9, -)",
            Self::UnexpectedHyphen => "Unexpected hyphen (should never prefix)",
            Self::EmptyDomain => "Expected non-empty partial qualified domain name",
            Self::NotFullyQualified => "Domain is not a FQDN (Fully qualified domain name)",
        }
    }
    fn cause(&self) -> Option<&dyn StdError> {
        match self {
            Self::EmptySubdomain => None,
            Self::IllegalChar => None,
            Self::UnexpectedHyphen => None,
            Self::EmptyDomain => None,
            Self::NotFullyQualified => None,
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum SubdomainName {
    Wildcard,
    Value(String),
}

impl SubdomainName {
    fn regularize(subdomain: &str) -> Result<String, Error> {
        // subdomains must be composed of letters (case-insensitive) or digits
        //     or hyphens, and would never begin with a hyphen
        // rfc1034: 3.5 Preferred name syntax
        //     <domain> ::= <subdomain> | " "
        //     <subdomain> ::= <label> | <subdomain> "." <label>
        //     <label> ::= <letter> [ [ <ldh-str> ] <let-dig> ]
        //     <ldh-str> ::= <let-dig-hyp> | <let-dig-hyp> <ldh-str>
        //     <let-dig-hyp> ::= <let-dig> | "-"
        //     <let-dig> ::= <letter> | <digit>
        //     <letter> ::= any one of the 52 alphabetic characters A through Z
        //                  in upper case and a through z in lower case
        //     <digit> ::= any one of the ten digits 0 through 9
        if subdomain.len() == 0 {
            return Err(Error::EmptySubdomain);
        }
        let mut is_first_char = true;
        for ch in subdomain.chars() {
            let is_digit = ch >= '0' && ch <= '9';
            let is_alpha_lower = ch >= 'a' && ch <= 'z';
            let is_alpha_upper = ch >= 'A' && ch <= 'Z';
            let is_hyphen = ch == '-';
            let is_ld = is_alpha_lower || is_alpha_upper || is_digit;
            let is_ldh = is_ld || is_hyphen;
            if !is_ldh {
                return Err(Error::IllegalChar);
            }
            if is_first_char && !is_ld {
                return Err(Error::UnexpectedHyphen);
            }
            is_first_char = false;
        }
        let subdomain = subdomain
            .chars()
            .map(|ch| ch.to_ascii_lowercase())
            .collect();
        Ok(subdomain)
    }
    pub fn from_string(subdomain: &str) -> Result<Self, Error> {
        if subdomain == "*" {
            Ok(Self::Wildcard)
        } else {
            let subdomain = Self::regularize(&subdomain)?;
            Ok(Self::Value(subdomain))
        }
    }
    pub fn to_string(&self) -> String {
        match self {
            Self::Wildcard => String::from("*"),
            Self::Value(v) => v.to_string(),
        }
    }
}

impl fmt::Debug for SubdomainName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("SubdomainName({})", self.to_string()))
    }
}

#[derive(PartialEq, Eq)]
pub struct DomainName {
    _subdns: Vec<SubdomainName>,
}

impl DomainName {
    fn from_dn(dn: &str, is_fqdn: bool) -> Result<Self, Error> {
        let mut subdomains: Vec<SubdomainName> = vec![];
        let mut buffer = String::default();
        for ch in dn.chars() {
            if ch != '.' {
                buffer.push(ch);
            } else {
                subdomains.push(SubdomainName::from_string(&buffer)?);
                buffer.clear();
            }
        }
        // a fully qualified domain must end with a dot
        // and a pqdn would never end with a dot
        if is_fqdn {
            if buffer.len() > 0 {
                return Err(Error::NotFullyQualified);
            }
        } else {
            if buffer.len() == 0 {
                return Err(Error::EmptySubdomain);
            }
            subdomains.push(SubdomainName::from_string(&buffer)?);
        }
        Ok(Self {
            _subdns: subdomains,
        })
    }
    pub fn from_fqdn(fqdn: &str) -> Result<Self, Error> {
        if fqdn == "." {
            return Ok(Self { _subdns: vec![] });
        }
        Self::from_dn(fqdn, true)
    }
    pub fn from_pqdn(pqdn: &str) -> Result<Self, Error> {
        if pqdn.len() == 0 {
            return Err(Error::EmptyDomain);
        }
        Self::from_dn(pqdn, false)
    }
    pub fn to_pqdn(&self) -> String {
        let mut buffer = String::default();
        let mut is_first_char = true;
        for component in &self._subdns {
            if !is_first_char {
                buffer.push('.');
            }
            is_first_char = false;
            buffer += &component.to_string();
        }
        buffer
    }
    pub fn to_string(&self) -> String {
        self.to_pqdn()
    }
}

impl fmt::Debug for DomainName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("DomainName({})", self.to_pqdn()))
    }
}

#[cfg(test)]
mod tests_subdomain_ok {
    use crate::ns::SubdomainName;

    fn expect(origin: &str, target: &str) {
        let src = SubdomainName::from_string(origin).unwrap();
        let targ = SubdomainName::Value(String::from(target));
        assert_eq!(src, targ);
    }

    #[test]
    fn sld() {
        expect("example", "example");
    }

    #[test]
    fn tld_com() {
        expect("com", "com");
    }

    #[test]
    fn tld_net() {
        expect("net", "net");
    }

    #[test]
    fn tld_org() {
        expect("org", "org");
    }

    #[test]
    fn capitalized() {
        expect("RuStLaNG", "rustlang");
    }

    #[test]
    fn digits() {
        expect("0123456789", "0123456789");
    }

    #[test]
    fn alphanumeric() {
        expect("a2c4e6g8i", "a2c4e6g8i");
    }

    #[test]
    fn ldh() {
        expect("example-subdomain-1", "example-subdomain-1");
    }

    #[test]
    fn hyphens() {
        expect("x--------", "x--------");
    }
}

#[cfg(test)]
mod tests_subdomain_fail {
    use crate::ns::Error;
    use crate::ns::SubdomainName;

    fn expect(origin: &str, error: Error) {
        let src = SubdomainName::from_string(origin).unwrap_err();
        assert_eq!(src, error);
    }

    #[test]
    fn empty() {
        expect("", Error::EmptySubdomain);
    }

    #[test]
    fn hyphen() {
        expect("-name", Error::UnexpectedHyphen);
    }

    #[test]
    fn space() {
        expect("subdomain name", Error::IllegalChar);
    }

    #[test]
    fn other_ascii() {
        expect("subdomain(name)", Error::IllegalChar);
    }

    #[test]
    fn underscore() {
        expect("subdomain_name", Error::IllegalChar);
    }

    #[test]
    fn unicode() {
        expect("测试", Error::IllegalChar);
    }
}

#[cfg(test)]
mod tests_domain_ok {
    use crate::ns::DomainName;
    use crate::ns::SubdomainName;

    fn vec_str_to_dn(origin: Vec<&str>) -> DomainName {
        return DomainName {
            _subdns: origin
                .iter()
                .map(|&s| SubdomainName::Value(String::from(s)))
                .collect(),
        };
    }

    fn pqdn_expect(origin: &str, target: Vec<&str>) {
        let src = DomainName::from_pqdn(origin).unwrap();
        assert_eq!(src, vec_str_to_dn(target));
    }

    fn fqdn_expect(origin: &str, target: Vec<&str>) {
        let src = DomainName::from_fqdn(origin).unwrap();
        assert_eq!(src, vec_str_to_dn(target));
    }

    #[test]
    fn pqdn_example() {
        pqdn_expect("www.example.com", vec!["www", "example", "com"]);
    }

    #[test]
    fn pqdn_numeric() {
        pqdn_expect("123.456.789", vec!["123", "456", "789"]);
    }

    #[test]
    fn pqdn_ldh() {
        pqdn_expect(
            "123-server.name-234.3a3",
            vec!["123-server", "name-234", "3a3"],
        );
    }

    #[test]
    fn pqdn_dyno() {
        pqdn_expect(
            "xhttp.dyno-123.serviceprovider.com",
            vec!["xhttp", "dyno-123", "serviceprovider", "com"],
        );
    }

    #[test]
    fn pqdn_unicode_ext() {
        pqdn_expect("xn--0zwm56d.com", vec!["xn--0zwm56d", "com"]);
    }

    #[test]
    fn fqdn_empty() {
        fqdn_expect(".", vec![]);
    }

    #[test]
    fn fqdn_example() {
        fqdn_expect("www.example.com.", vec!["www", "example", "com"]);
    }

    #[test]
    fn fqdn_unicode() {
        fqdn_expect("xn--0zwm56d.com.", vec!["xn--0zwm56d", "com"]);
    }

    #[test]
    fn rfc1034_1() {
        pqdn_expect("A.ISI.EDU", vec!["a", "isi", "edu"]);
    }

    #[test]
    fn rfc1034_2() {
        pqdn_expect("XX.LCS.MIT.EDU", vec!["xx", "lcs", "mit", "edu"]);
    }

    #[test]
    fn rfc1034_3() {
        pqdn_expect("SRI-NIC.ARPA", vec!["sri-nic", "arpa"]);
    }
}

#[cfg(test)]
mod tests_domain_fail {
    use crate::ns::DomainName;
    use crate::ns::Error;

    fn expect_pqdn(origin: &str, error: Error) {
        let src = DomainName::from_pqdn(origin).unwrap_err();
        assert_eq!(src, error);
    }

    fn expect_fqdn(origin: &str, error: Error) {
        let src = DomainName::from_fqdn(origin).unwrap_err();
        assert_eq!(src, error);
    }

    #[test]
    fn pqdn_empty() {
        expect_pqdn("", Error::EmptyDomain);
    }

    #[test]
    fn pqdn_empty_component() {
        expect_pqdn("www..com", Error::EmptySubdomain);
    }

    #[test]
    fn pqdn_bad_hyphen() {
        expect_pqdn("server.-domain.com", Error::UnexpectedHyphen);
    }

    #[test]
    fn pqdn_unicode() {
        expect_pqdn("测试.com", Error::IllegalChar);
    }

    #[test]
    fn pqdn_slash() {
        expect_pqdn("www.example.com/url", Error::IllegalChar);
    }

    #[test]
    fn pqdn_space() {
        expect_pqdn("www example.com", Error::IllegalChar);
    }

    #[test]
    fn pqdn_not_fqdn() {
        expect_pqdn("www.example.com.", Error::EmptySubdomain);
    }

    #[test]
    fn fqdn_dots() {
        expect_fqdn("multi.dots.after..", Error::EmptySubdomain);
    }

    #[test]
    fn fqdn_illegal_char() {
        expect_fqdn("?.char.com.", Error::IllegalChar);
    }

    #[test]
    fn fqdn_not_pqdn() {
        expect_fqdn("www.example.com", Error::NotFullyQualified);
    }
}

#[cfg(test)]
mod tests_out {
    use crate::ns::DomainName;

    #[test]
    fn domain_name_out_example() {
        assert_eq!(
            DomainName::from_pqdn("www.example.com")
                .unwrap()
                .to_string(),
            String::from("www.example.com")
        );
    }

    #[test]
    fn domain_name_out_mixed() {
        assert_eq!(
            DomainName::from_fqdn("SERVER-1024.test.ORG.")
                .unwrap()
                .to_string(),
            String::from("server-1024.test.org")
        );
    }
}
