pub enum DomainComponent {
    Wildcard,
    Value(String),
}

impl DomainComponent {
    pub fn from_str(component: &str) -> Self {
        if component == "*" {
            return Self::Wildcard;
        } else {
            return Self::Value(String::from(component));
        }
    }
}

impl std::fmt::Display for DomainComponent {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Wildcard => formatter.write_fmt(format_args!("*")),
            Self::Value(s) => formatter.write_fmt(format_args!("{}", s)),
        }
    }
}

pub struct DomainName {
    _comps: Vec<DomainComponent>,
}

impl DomainName {
    // pub fn new() -> Self {
    //     Self { _comps: vec![] }
    // }
    pub fn from_fqdn(fqdn: &str) -> Self {
        let mut components: Vec<DomainComponent> = vec![];
        let mut buffer: String = String::from("");
        for ch in fqdn.chars() {
            if ch != '.' {
                buffer.push(ch);
            } else {
                components.push(DomainComponent::from_str(&buffer));
                buffer = String::from("");
            }
        }
        if buffer.len() > 0 {
            components.push(DomainComponent::from_str(&buffer));
        }
        Self { _comps: components }
    }
}

impl std::fmt::Display for DomainName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        for component in &self._comps {
            formatter.write_fmt(format_args!("{}.", component))?;
        }
        Ok(())
    }
}
