use regex::Regex;
use std::sync::LazyLock;

struct RedactionRule {
    pattern: Regex,
    replacement: &'static str,
}

static RULES: LazyLock<Vec<RedactionRule>> = LazyLock::new(|| {
    vec![
        RedactionRule {
            pattern: Regex::new(
                r"(?i)(-p\s*|--password[= ]|password[= :])\s*\S+"
            ).unwrap(),
            replacement: "$1[REDACTED]",
        },
        RedactionRule {
            pattern: Regex::new(
                r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b"
            ).unwrap(),
            replacement: "[REDACTED_EMAIL]",
        },
        RedactionRule {
            pattern: Regex::new(
                r"\b(?:\d{1,3}\.){3}\d{1,3}\b"
            ).unwrap(),
            replacement: "[REDACTED_IP]",
        },
        RedactionRule {
            pattern: Regex::new(
                r"(?i)(api[_-]?key|token|secret|bearer)\s*[=: ]\s*[A-Za-z0-9+/=_\-]{20,}"
            ).unwrap(),
            replacement: "$1=[REDACTED_KEY]",
        },
        RedactionRule {
            pattern: Regex::new(
                r"\b[A-Fa-f0-9]{40,}\b"
            ).unwrap(),
            replacement: "[REDACTED_KEY]",
        },
    ]
});

pub fn redact_sensitive(text: &str) -> String {
    let mut result = text.to_string();
    for rule in RULES.iter() {
        result = rule.pattern.replace_all(&result, rule.replacement).to_string();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_ip() {
        let input = "ssh root@192.168.1.100";
        let output = redact_sensitive(input);
        assert!(output.contains("[REDACTED_IP]"));
        assert!(!output.contains("192.168.1.100"));
    }

    #[test]
    fn test_redact_email() {
        let input = "contact admin@example.com for help";
        let output = redact_sensitive(input);
        assert!(output.contains("[REDACTED_EMAIL]"));
        assert!(!output.contains("admin@example.com"));
    }

    #[test]
    fn test_redact_password() {
        let input = "mysql -p secretpass123";
        let output = redact_sensitive(input);
        assert!(output.contains("[REDACTED]"));
    }
}
