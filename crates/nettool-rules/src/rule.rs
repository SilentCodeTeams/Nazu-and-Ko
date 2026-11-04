use std::collections::HashSet;
use std::error::Error;
use std::fmt;

use nettool_proto::Protocol;

/// Direction of a captured chunk, independent of the proxy implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    ClientToServer,
    ServerToClient,
}

impl Direction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ClientToServer => "C->S",
            Self::ServerToClient => "S->C",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "c2s" | "c->s" | "client-to-server" | "client" | "serverbound" => {
                Some(Self::ClientToServer)
            }
            "s2c" | "s->c" | "server-to-client" | "server" | "clientbound" => {
                Some(Self::ServerToClient)
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Allow,
    Drop,
}

impl Action {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Drop => "drop",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "allow" => Some(Self::Allow),
            "drop" => Some(Self::Drop),
            _ => None,
        }
    }
}

/// One predicate rule. Every configured predicate must match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    name: String,
    action: Action,
    direction: Option<Direction>,
    protocol: Option<Protocol>,
    src_port: Option<u16>,
    dst_port: Option<u16>,
    min_len: Option<usize>,
    max_len: Option<usize>,
    contains: Option<Vec<u8>>,
    prefix: Option<Vec<u8>>,
    suffix: Option<Vec<u8>>,
}

impl Rule {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn action(&self) -> Action {
        self.action
    }

    pub fn direction(&self) -> Option<Direction> {
        self.direction
    }

    pub fn protocol(&self) -> Option<Protocol> {
        self.protocol
    }

    pub fn matches(&self, context: &RuleContext<'_>) -> bool {
        if self
            .direction
            .is_some_and(|direction| direction != context.direction)
        {
            return false;
        }
        if self
            .protocol
            .is_some_and(|protocol| protocol != context.protocol)
        {
            return false;
        }
        if self.src_port.is_some_and(|port| port != context.src_port) {
            return false;
        }
        if self.dst_port.is_some_and(|port| port != context.dst_port) {
            return false;
        }
        if self.min_len.is_some_and(|min| context.data.len() < min) {
            return false;
        }
        if self.max_len.is_some_and(|max| context.data.len() > max) {
            return false;
        }
        if let Some(prefix) = &self.prefix {
            if !context.data.starts_with(prefix) {
                return false;
            }
        }
        if let Some(suffix) = &self.suffix {
            if !context.data.ends_with(suffix) {
                return false;
            }
        }
        if let Some(needle) = &self.contains {
            let found = context
                .data
                .windows(needle.len())
                .any(|window| window == needle.as_slice());
            if !found {
                return false;
            }
        }
        true
    }

    /// Parse one rule specification.
    ///
    /// The first token is the action, followed by whitespace-separated
    /// key=value predicates. Values may be double-quoted; inside quotes the
    /// escapes \\n, \\r, \\t, \\\\ and \\" are accepted.
    pub fn parse(spec: &str, index: usize) -> Result<Self, RuleError> {
        let tokens = tokenize(spec)?;
        let action_token = tokens.first().ok_or_else(|| RuleError::new("empty rule"))?;
        let action = Action::parse(action_token).ok_or_else(|| {
            RuleError::new(format!(
                "unknown action '{action_token}' (expected drop or allow)"
            ))
        })?;

        let mut name = format!("rule-{index}");
        let mut direction = None;
        let mut protocol = None;
        let mut src_port = None;
        let mut dst_port = None;
        let mut min_len = None;
        let mut max_len = None;
        let mut contains = None;
        let mut prefix = None;
        let mut suffix = None;
        let mut predicates = 0usize;
        let mut seen = HashSet::new();

        for token in &tokens[1..] {
            let (raw_key, value) = token
                .split_once('=')
                .ok_or_else(|| RuleError::new(format!("expected key=value, got '{token}'")))?;
            if raw_key.is_empty() {
                return Err(RuleError::new(format!("empty key in '{token}'")));
            }

            let key = raw_key.to_ascii_lowercase();
            let canonical = match key.as_str() {
                "name" => "name",
                "dir" | "direction" => "direction",
                "proto" | "protocol" => "protocol",
                "src_port" | "src-port" | "source_port" => "src_port",
                "dst_port" | "dst-port" | "destination_port" => "dst_port",
                "min_len" | "min-len" => "min_len",
                "max_len" | "max-len" => "max_len",
                "contains" => "contains",
                "prefix" => "prefix",
                "suffix" => "suffix",
                _ => {
                    return Err(RuleError::new(format!(
                        "unknown key '{raw_key}' (supported: name, dir, proto, src_port, dst_port, min_len, max_len, contains, prefix, suffix)"
                    )))
                }
            };
            if !seen.insert(canonical.to_string()) {
                return Err(RuleError::new(format!("duplicate key '{raw_key}'")));
            }

            match canonical {
                "name" => {
                    if value.is_empty() {
                        return Err(RuleError::new("name cannot be empty"));
                    }
                    name = value.to_string();
                }
                "direction" => {
                    direction = Some(Direction::parse(value).ok_or_else(|| {
                        RuleError::new(format!("bad direction '{value}' (expected c2s or s2c)"))
                    })?);
                    predicates += 1;
                }
                "protocol" => {
                    protocol = Some(parse_protocol(value).ok_or_else(|| {
                        RuleError::new(format!(
                            "bad protocol '{value}' (expected mc, http, tls, dns, ws, quic or raw)"
                        ))
                    })?);
                    predicates += 1;
                }
                "src_port" => {
                    src_port = Some(parse_port(value).ok_or_else(|| {
                        RuleError::new(format!("bad src_port '{value}' (expected 1-65535)"))
                    })?);
                    predicates += 1;
                }
                "dst_port" => {
                    dst_port = Some(parse_port(value).ok_or_else(|| {
                        RuleError::new(format!("bad dst_port '{value}' (expected 1-65535)"))
                    })?);
                    predicates += 1;
                }
                "min_len" => {
                    min_len = Some(
                        value
                            .parse::<usize>()
                            .map_err(|_| RuleError::new(format!("bad min_len '{value}'")))?,
                    );
                    predicates += 1;
                }
                "max_len" => {
                    max_len = Some(
                        value
                            .parse::<usize>()
                            .map_err(|_| RuleError::new(format!("bad max_len '{value}'")))?,
                    );
                    predicates += 1;
                }
                "contains" => {
                    if value.is_empty() {
                        return Err(RuleError::new("contains cannot be empty"));
                    }
                    contains = Some(value.as_bytes().to_vec());
                    predicates += 1;
                }
                "prefix" => {
                    if value.is_empty() {
                        return Err(RuleError::new("prefix cannot be empty"));
                    }
                    prefix = Some(value.as_bytes().to_vec());
                    predicates += 1;
                }
                "suffix" => {
                    if value.is_empty() {
                        return Err(RuleError::new("suffix cannot be empty"));
                    }
                    suffix = Some(value.as_bytes().to_vec());
                    predicates += 1;
                }
                _ => unreachable!("canonical key is exhaustive"),
            }
        }

        if predicates == 0 {
            return Err(RuleError::new(
                "rule needs at least one predicate (for example dir=c2s or proto=mc)",
            ));
        }
        if let (Some(min), Some(max)) = (min_len, max_len) {
            if min > max {
                return Err(RuleError::new("min_len cannot be greater than max_len"));
            }
        }

        Ok(Self {
            name,
            action,
            direction,
            protocol,
            src_port,
            dst_port,
            min_len,
            max_len,
            contains,
            prefix,
            suffix,
        })
    }
}

fn parse_protocol(value: &str) -> Option<Protocol> {
    match value.to_ascii_lowercase().as_str() {
        "mc" | "minecraft" => Some(Protocol::Minecraft),
        "http" => Some(Protocol::Http),
        "tls" | "ssl" | "https" => Some(Protocol::Tls),
        "dns" => Some(Protocol::Dns),
        "ws" | "websocket" => Some(Protocol::WebSocket),
        "quic" | "h3" | "http3" => Some(Protocol::Quic),
        "raw" | "tcp" | "unknown" => Some(Protocol::Unknown),
        _ => None,
    }
}

fn parse_port(value: &str) -> Option<u16> {
    value.parse::<u16>().ok().filter(|port| *port != 0)
}

fn tokenize(input: &str) -> Result<Vec<String>, RuleError> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut escaped = false;

    for ch in input.chars() {
        if escaped {
            let decoded = match ch {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                '\\' => '\\',
                '"' => '"',
                other => return Err(RuleError::new(format!("unsupported escape '\\{other}'"))),
            };
            current.push(decoded);
            escaped = false;
            continue;
        }

        match ch {
            '\\' if in_quotes => escaped = true,
            '"' => in_quotes = !in_quotes,
            c if c.is_whitespace() && !in_quotes => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            c => current.push(c),
        }
    }

    if escaped {
        return Err(RuleError::new("trailing escape in rule"));
    }
    if in_quotes {
        return Err(RuleError::new("unterminated quote in rule"));
    }
    if !current.is_empty() {
        tokens.push(current);
    }

    Ok(tokens)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleError {
    message: String,
}

impl RuleError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for RuleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for RuleError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleContext<'a> {
    pub direction: Direction,
    pub protocol: Protocol,
    pub src_port: u16,
    pub dst_port: u16,
    pub data: &'a [u8],
}

/// Ordered rule list. The first matching rule decides; no match means allow.
#[derive(Debug, Clone, Default)]
pub struct RuleSet {
    rules: Vec<Rule>,
}

impl RuleSet {
    pub fn parse(specs: &[String]) -> Result<Self, RuleError> {
        let mut rules = Vec::with_capacity(specs.len());
        for (index, spec) in specs.iter().enumerate() {
            let rule = Rule::parse(spec, index + 1)
                .map_err(|error| RuleError::new(format!("rule {}: {error}", index + 1)))?;
            rules.push(rule);
        }
        Ok(Self { rules })
    }

    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }

    pub fn len(&self) -> usize {
        self.rules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    pub fn evaluate(&self, context: &RuleContext<'_>) -> Option<&Rule> {
        self.rules.iter().find(|rule| rule.matches(context))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(specs: &[&str]) -> RuleSet {
        let owned: Vec<String> = specs.iter().map(|spec| spec.to_string()).collect();
        RuleSet::parse(&owned).expect("valid rules")
    }

    fn context<'a>(
        direction: Direction,
        protocol: Protocol,
        src_port: u16,
        dst_port: u16,
        data: &'a [u8],
    ) -> RuleContext<'a> {
        RuleContext {
            direction,
            protocol,
            src_port,
            dst_port,
            data,
        }
    }

    #[test]
    fn parses_drop_rule_with_predicates() {
        let rules = set(&["drop dir=c2s proto=mc contains=hello"]);
        assert_eq!(rules.len(), 1);
        let rule = &rules.rules()[0];
        assert_eq!(rule.name(), "rule-1");
        assert_eq!(rule.action(), Action::Drop);
        assert_eq!(rule.direction(), Some(Direction::ClientToServer));
        assert_eq!(rule.protocol(), Some(Protocol::Minecraft));
        assert!(rule.matches(&context(
            Direction::ClientToServer,
            Protocol::Minecraft,
            5000,
            25565,
            b"say hello",
        )));
    }

    #[test]
    fn quoted_values_keep_spaces_and_escapes() {
        let rules = set(&["drop contains=\"hello world\\n\""]);
        assert!(rules.rules()[0].matches(&context(
            Direction::ClientToServer,
            Protocol::Unknown,
            1,
            2,
            b"hello world\n",
        )));
        assert!(!rules.rules()[0].matches(&context(
            Direction::ClientToServer,
            Protocol::Unknown,
            1,
            2,
            b"hello world",
        )));
    }

    #[test]
    fn first_matching_rule_wins() {
        let rules = set(&["allow dir=c2s", "drop dir=c2s"]);
        let matched = rules
            .evaluate(&context(
                Direction::ClientToServer,
                Protocol::Unknown,
                1,
                2,
                b"x",
            ))
            .expect("allow matches");
        assert_eq!(matched.action(), Action::Allow);
    }

    #[test]
    fn ports_and_lengths_are_inclusive() {
        let rules = set(&["drop src_port=1234 dst_port=80 min_len=3 max_len=5"]);
        assert!(rules.rules()[0].matches(&context(
            Direction::ClientToServer,
            Protocol::Unknown,
            1234,
            80,
            b"abc",
        )));
        assert!(rules.rules()[0].matches(&context(
            Direction::ClientToServer,
            Protocol::Unknown,
            1234,
            80,
            b"abcde",
        )));
        assert!(!rules.rules()[0].matches(&context(
            Direction::ClientToServer,
            Protocol::Unknown,
            1234,
            80,
            b"ab",
        )));
    }

    #[test]
    fn no_match_is_none() {
        let rules = set(&["drop proto=http"]);
        assert!(rules
            .evaluate(&context(
                Direction::ClientToServer,
                Protocol::Minecraft,
                1,
                2,
                b"x",
            ))
            .is_none());
    }

    #[test]
    fn rejects_invalid_rules() {
        let cases = [
            "",
            "drop",
            "explode dir=c2s",
            "drop nope=1",
            "drop proto=nope",
            "drop dir=c2s dir=s2c",
            "drop min_len=5 max_len=2",
            "drop contains=",
            "drop contains=\"unterminated",
            "drop name=only",
            "drop src_port=0",
        ];
        for case in cases {
            let owned = vec![case.to_string()];
            assert!(
                RuleSet::parse(&owned).is_err(),
                "expected invalid rule: {case}"
            );
        }
    }

    #[test]
    fn custom_name_is_kept() {
        let rules = set(&["drop name=block-secret proto=http"]);
        assert_eq!(rules.rules()[0].name(), "block-secret");
    }
}
