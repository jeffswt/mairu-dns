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
mod tests {
    use crate::ns::DomainName;
    use crate::ns::Error;
    use crate::ns::SubdomainName;

    fn expect_subdomain_ok(origin: &str, target: &str) {
        let src = SubdomainName::from_string(origin).unwrap();
        let targ = SubdomainName::Value(String::from(target));
        assert_eq!(src, targ);
    }

    fn expect_subdomain_error(origin: &str, error: Error) {
        let src = SubdomainName::from_string(origin).unwrap_err();
        assert_eq!(src, error);
    }

    fn expect_vec_str_to_dn(origin: Vec<&str>) -> DomainName {
        return DomainName {
            _subdns: origin
                .iter()
                .map(|&s| SubdomainName::Value(String::from(s)))
                .collect(),
        };
    }

    fn expect_domain_pqdn_ok(origin: &str, target: Vec<&str>) {
        let src = DomainName::from_pqdn(origin).unwrap();
        assert_eq!(src, expect_vec_str_to_dn(target));
    }

    fn expect_domain_pqdn_error(origin: &str, error: Error) {
        let src = DomainName::from_pqdn(origin).unwrap_err();
        assert_eq!(src, error);
    }

    fn expect_domain_fqdn_ok(origin: &str, target: Vec<&str>) {
        let src = DomainName::from_fqdn(origin).unwrap();
        assert_eq!(src, expect_vec_str_to_dn(target));
    }

    fn expect_domain_fqdn_error(origin: &str, error: Error) {
        let src = DomainName::from_fqdn(origin).unwrap_err();
        assert_eq!(src, error);
    }

    #[test]
    fn subdomain_name_sld() {
        expect_subdomain_ok("example", "example");
    }

    #[test]
    fn subdomain_name_tld_com() {
        expect_subdomain_ok("com", "com");
    }

    #[test]
    fn subdomain_name_tld_net() {
        expect_subdomain_ok("net", "net");
    }

    #[test]
    fn subdomain_name_tld_org() {
        expect_subdomain_ok("org", "org");
    }

    #[test]
    fn subdomain_name_capitalized() {
        expect_subdomain_ok("RuStLaNG", "rustlang");
    }

    #[test]
    fn subdomain_name_digits() {
        expect_subdomain_ok("0123456789", "0123456789");
    }

    #[test]
    fn subdomain_name_alphanumeric() {
        expect_subdomain_ok("a2c4e6g8i", "a2c4e6g8i");
    }

    #[test]
    fn subdomain_name_ldh() {
        expect_subdomain_ok("example-subdomain-1", "example-subdomain-1");
    }

    #[test]
    fn subdomain_name_hyphens() {
        expect_subdomain_ok("x--------", "x--------");
    }

    #[test]
    fn subdomain_name_fail_empty() {
        expect_subdomain_error("", Error::EmptySubdomain);
    }

    #[test]
    fn subdomain_name_fail_hyphen() {
        expect_subdomain_error("-name", Error::UnexpectedHyphen);
    }

    #[test]
    fn subdomain_name_fail_space() {
        expect_subdomain_error("subdomain name", Error::IllegalChar);
    }

    #[test]
    fn subdomain_name_fail_other_ascii() {
        expect_subdomain_error("subdomain(name)", Error::IllegalChar);
    }

    #[test]
    fn subdomain_name_fail_underscore() {
        expect_subdomain_error("subdomain_name", Error::IllegalChar);
    }

    #[test]
    fn subdomain_name_fail_unicode() {
        expect_subdomain_error("测试", Error::IllegalChar);
    }

    #[test]
    fn domain_name_example() {
        expect_domain_pqdn_ok("www.example.com", vec!["www", "example", "com"]);
    }

    #[test]
    fn domain_name_numeric() {
        expect_domain_pqdn_ok("123.456.789", vec!["123", "456", "789"]);
    }

    #[test]
    fn domain_name_ldh() {
        expect_domain_pqdn_ok(
            "123-server.name-234.3a3",
            vec!["123-server", "name-234", "3a3"],
        );
    }

    #[test]
    fn domain_name_dyno() {
        expect_domain_pqdn_ok(
            "xhttp.dyno-123.serviceprovider.com",
            vec!["xhttp", "dyno-123", "serviceprovider", "com"],
        );
    }

    #[test]
    fn domain_name_unicode_ext() {
        expect_domain_pqdn_ok("xn--0zwm56d.com", vec!["xn--0zwm56d", "com"]);
    }

    #[test]
    fn domain_name_fail_empty() {
        expect_domain_pqdn_error("", Error::EmptyDomain);
    }

    #[test]
    fn domain_name_fail_empty_component() {
        expect_domain_pqdn_error("www..com", Error::EmptySubdomain);
    }

    #[test]
    fn domain_name_fail_bad_hyphen() {
        expect_domain_pqdn_error("server.-domain.com", Error::UnexpectedHyphen);
    }

    #[test]
    fn domain_name_fail_unicode() {
        expect_domain_pqdn_error("测试.com", Error::IllegalChar);
    }

    #[test]
    fn domain_name_fail_slash() {
        expect_domain_pqdn_error("www.example.com/url", Error::IllegalChar);
    }

    #[test]
    fn domain_name_fail_space() {
        expect_domain_pqdn_error("www example.com", Error::IllegalChar);
    }

    #[test]
    fn domain_name_fqdn_empty() {
        expect_domain_fqdn_ok(".", vec![]);
    }

    #[test]
    fn domain_name_fqdn_example() {
        expect_domain_fqdn_ok("www.example.com.", vec!["www", "example", "com"]);
    }

    #[test]
    fn domain_name_fqdn_unicode() {
        expect_domain_fqdn_ok("xn--0zwm56d.com.", vec!["xn--0zwm56d", "com"]);
    }

    #[test]
    fn domain_name_fqdn_fail_dots() {
        expect_domain_fqdn_error("multi.dots.after..", Error::EmptySubdomain);
    }

    #[test]
    fn domain_name_fqdn_fail_illegal_char() {
        expect_domain_fqdn_error("?.char.com.", Error::IllegalChar);
    }

    #[test]
    fn domain_name_fqdn_not_pqdn() {
        expect_domain_fqdn_error("www.example.com", Error::NotFullyQualified);
    }

    #[test]
    fn domain_name_pqdn_not_fqdn() {
        expect_domain_pqdn_error("www.example.com.", Error::EmptySubdomain);
    }

    #[test]
    fn domain_name_rfc1034_1() {
        expect_domain_pqdn_ok("A.ISI.EDU", vec!["a", "isi", "edu"]);
    }

    #[test]
    fn domain_name_rfc1034_2() {
        expect_domain_pqdn_ok("XX.LCS.MIT.EDU", vec!["xx", "lcs", "mit", "edu"]);
        DomainName::from_pqdn("XX.LCS.MIT.EDU").unwrap();
    }

    #[test]
    fn domain_name_rfc1034_3() {
        expect_domain_pqdn_ok("SRI-NIC.ARPA", vec!["sri-nic", "arpa"]);
    }

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
