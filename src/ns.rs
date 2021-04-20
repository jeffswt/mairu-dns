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
    pub fn from_str(subdomain: &str) -> Result<Self, Error> {
        if subdomain == "*" {
            Ok(Self::Wildcard)
        } else {
            let subdomain = Self::regularize(&subdomain)?;
            Ok(Self::Value(subdomain))
        }
    }
}

impl fmt::Display for SubdomainName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Wildcard => f.write_fmt(format_args!("*")),
            Self::Value(s) => f.write_fmt(format_args!("{}", s)),
        }
    }
}

impl fmt::Debug for SubdomainName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Wildcard => f.write_fmt(format_args!("SubdomainName::Wildcard")),
            Self::Value(s) => f.write_fmt(format_args!("SubdomainName::Value({})", s)),
        }
    }
}

#[derive(PartialEq, Eq)]
pub struct DomainName {
    _subdns: Vec<SubdomainName>,
}

impl DomainName {
    fn from_dn(dn: &str, is_fqdn: bool) -> Result<Self, Error> {
        let mut subdomains: Vec<SubdomainName> = vec![];
        let mut buffer: String = String::from("");
        for ch in dn.chars() {
            if ch != '.' {
                buffer.push(ch);
            } else {
                subdomains.push(SubdomainName::from_str(&buffer)?);
                buffer = String::from("");
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
            subdomains.push(SubdomainName::from_str(&buffer)?);
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
}

impl fmt::Debug for DomainName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("DomainName(")?;
        let mut is_first_char = true;
        for component in &self._subdns {
            if !is_first_char {
                f.write_str(".")?;
            }
            is_first_char = false;
            f.write_fmt(format_args!("{}", component))?;
        }
        f.write_str(")")?;
        Ok(())
    }
}

impl fmt::Display for DomainName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut is_first_char = true;
        for component in &self._subdns {
            if !is_first_char {
                f.write_str(".")?;
            }
            is_first_char = false;
            f.write_fmt(format_args!("{}", component))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // TODO: expect values or errors
    use crate::ns::DomainName;
    use crate::ns::Error;
    use crate::ns::SubdomainName;

    fn expect_subdomain_ok(origin: &str, target: &str) {
        let src = SubdomainName::from_str(origin).unwrap();
        let targ = SubdomainName::Value(String::from(target));
        assert_eq!(src, targ);
    }

    fn expect_subdomain_error(origin: &str, error: Error) {
        let src = SubdomainName::from_str(origin).unwrap_err();
        assert_eq!(src, error);
    }

    fn expect_domain_pqdn_ok(origin: &str, target: DomainName) {
        let src = DomainName::from_pqdn(origin).unwrap();
        assert_eq!(src, target);
    }

    fn expect_domain_pqdn_error(origin: &str, error: Error) {
        let src = DomainName::from_pqdn(origin).unwrap_err();
        assert_eq!(src, error);
    }

    fn expect_domain_fqdn_ok(origin: &str, target: DomainName) {
        let src = DomainName::from_fqdn(origin).unwrap();
        assert_eq!(src, target);
    }

    fn expect_domain_fqdn_error(origin: &str, error: Error) {
        let src = DomainName::from_fqdn(origin).unwrap_err();
        assert_eq!(src, error);
    }

    // valid subdomain names
    #[test]
    fn subdomain_name_sld() {
        SubdomainName::from_str("example").unwrap();
    }
    #[test]
    fn subdomain_name_tld_com() {
        SubdomainName::from_str("com").unwrap();
    }
    #[test]
    fn subdomain_name_tld_net() {
        SubdomainName::from_str("net").unwrap();
    }
    #[test]
    fn subdomain_name_tld_org() {
        SubdomainName::from_str("org").unwrap();
    }
    #[test]
    fn subdomain_name_capitalized() {
        SubdomainName::from_str("RuStLaNG").unwrap();
    }
    #[test]
    fn subdomain_name_digits() {
        SubdomainName::from_str("0123456789").unwrap();
    }
    #[test]
    fn subdomain_name_alphanumeric() {
        SubdomainName::from_str("a2c4e6g8i").unwrap();
    }
    #[test]
    fn subdomain_name_ldh() {
        SubdomainName::from_str("example-subdomain-1").unwrap();
    }
    #[test]
    fn subdomain_name_hyphens() {
        SubdomainName::from_str("x--------").unwrap();
    }

    // invalid subdomain names failing
    #[test]
    fn subdomain_name_fail_empty() {
        SubdomainName::from_str("").unwrap_err();
    }
    #[test]
    fn subdomain_name_fail_hyphen() {
        SubdomainName::from_str("-name").unwrap_err();
    }
    #[test]
    fn subdomain_name_fail_space() {
        SubdomainName::from_str("subdomain name").unwrap_err();
    }
    #[test]
    fn subdomain_name_fail_other_ascii() {
        SubdomainName::from_str("subdomain(name)").unwrap_err();
    }
    #[test]
    fn subdomain_name_fail_underscore() {
        SubdomainName::from_str("subdomain_name").unwrap_err();
    }
    #[test]
    fn subdomain_name_fail_unicode() {
        expect_subdomain_error("测试", Error::IllegalChar);
    }

    // valid PQDN examples
    #[test]
    fn domain_name_example() {
        DomainName::from_pqdn("www.example.com").unwrap();
    }
    #[test]
    fn domain_name_numeric() {
        DomainName::from_pqdn("123.456.789").unwrap();
    }
    #[test]
    fn domain_name_ldh() {
        DomainName::from_pqdn("123-server.name-234.3a3").unwrap();
    }
    #[test]
    fn domain_name_dyno() {
        DomainName::from_pqdn("xhttp.dyno-123.serviceprovider.com").unwrap();
    }
    #[test]
    fn domain_name_unicode_ext() {
        DomainName::from_pqdn("xn--0zwm56d.com").unwrap();
    }

    // invalid pqdn failing
    #[test]
    fn domain_name_fail_empty() {
        DomainName::from_pqdn("").unwrap_err();
    }
    #[test]
    fn domain_name_fail_empty_component() {
        DomainName::from_pqdn("www..com").unwrap_err();
    }
    #[test]
    fn domain_name_fail_bad_hyphen() {
        DomainName::from_pqdn("server.-domain.com").unwrap_err();
    }
    #[test]
    fn domain_name_fail_unicode() {
        DomainName::from_pqdn("测试.com").unwrap_err();
    }
    #[test]
    fn domain_name_fail_slash() {
        DomainName::from_pqdn("www.example.com/url").unwrap_err();
    }
    #[test]
    fn domain_name_fail_space() {
        DomainName::from_pqdn("www example.com").unwrap_err();
    }

    // valid fqdn examples
    #[test]
    fn domain_name_fqdn_empty() {
        DomainName::from_fqdn(".").unwrap();
    }
    #[test]
    fn domain_name_fqdn_example() {
        DomainName::from_fqdn("www.example.com.").unwrap();
    }
    #[test]
    fn domain_name_fqdn_unicode() {
        DomainName::from_fqdn("xn--0zwm56d.com.").unwrap();
    }

    // certain fqdn that would fail
    #[test]
    fn domain_name_fqdn_fail_dots() {
        DomainName::from_fqdn("multi.dots.after..").unwrap_err();
    }
    #[test]
    fn domain_name_fqdn_fail_illegal_char() {
        DomainName::from_fqdn("?.char.com.").unwrap_err();
    }

    // issues considering fqdn and pqdn differences
    #[test]
    fn domain_name_fqdn_not_pqdn() {
        DomainName::from_fqdn("www.example.com").unwrap_err();
    }
    #[test]
    fn domain_name_pqdn_not_fqdn() {
        DomainName::from_pqdn("www.example.com.").unwrap_err();
    }

    // RFC1034 examples
    #[test]
    fn domain_name_rfc1034_1() {
        DomainName::from_pqdn("A.ISI.EDU").unwrap();
    }
    #[test]
    fn domain_name_rfc1034_2() {
        DomainName::from_pqdn("XX.LCS.MIT.EDU").unwrap();
    }
    #[test]
    fn domain_name_rfc1034_3() {
        DomainName::from_pqdn("SRI-NIC.ARPA").unwrap();
    }
}
