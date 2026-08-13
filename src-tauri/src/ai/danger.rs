use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DangerWarning {
    pub level: String,
    pub message: String,
}

struct DangerRule {
    pattern: Regex,
    level: &'static str,
    message: &'static str,
}

static DANGER_RULES: LazyLock<Vec<DangerRule>> = LazyLock::new(|| {
    vec![
        DangerRule {
            pattern: Regex::new(r"rm\s+(-[a-zA-Z]*f[a-zA-Z]*\s+)?(-[a-zA-Z]*r[a-zA-Z]*\s+)?(/|/\s)").unwrap(),
            level: "critical",
            message: "This command will recursively delete files from the root filesystem. This is almost certainly destructive and irreversible.",
        },
        DangerRule {
            pattern: Regex::new(r"rm\s+-[a-zA-Z]*r[a-zA-Z]*f|rm\s+-[a-zA-Z]*f[a-zA-Z]*r").unwrap(),
            level: "warning",
            message: "Recursive force-delete detected. Make sure you have the correct path — this cannot be undone.",
        },
        DangerRule {
            pattern: Regex::new(r"dd\s+.*of=/dev/[a-z]").unwrap(),
            level: "critical",
            message: "Writing directly to a block device with dd. This will overwrite all data on the target disk.",
        },
        DangerRule {
            pattern: Regex::new(r"mkfs\.\w+\s+/dev/").unwrap(),
            level: "critical",
            message: "Formatting a disk device. All data on the target partition will be permanently erased.",
        },
        DangerRule {
            pattern: Regex::new(r"chmod\s+777\s+/(etc|usr|var|bin|sbin|boot)").unwrap(),
            level: "warning",
            message: "Setting world-writable permissions on a sensitive system directory. This is a security risk.",
        },
        DangerRule {
            pattern: Regex::new(r">\s*/etc/").unwrap(),
            level: "warning",
            message: "Redirecting output to a file in /etc/. This may overwrite critical system configuration.",
        },
        DangerRule {
            pattern: Regex::new(r"kill\s+-9\s+1\b").unwrap(),
            level: "warning",
            message: "Sending SIGKILL to PID 1 (init/systemd). This may crash or reboot the system.",
        },
        DangerRule {
            pattern: Regex::new(r":\(\)\s*\{\s*:\s*\|\s*:\s*&\s*\}\s*;\s*:").unwrap(),
            level: "critical",
            message: "Fork bomb detected. This will consume all system resources and likely require a hard reboot.",
        },
        DangerRule {
            pattern: Regex::new(r">\s*/dev/sd[a-z]").unwrap(),
            level: "critical",
            message: "Writing directly to a raw disk device. This will corrupt the filesystem.",
        },
        DangerRule {
            pattern: Regex::new(r"shutdown|reboot|init\s+[06]|poweroff").unwrap(),
            level: "warning",
            message: "System shutdown or reboot command detected. The remote machine will become temporarily unreachable.",
        },
    ]
});

pub fn check_danger(command: &str) -> Option<DangerWarning> {
    for rule in DANGER_RULES.iter() {
        if rule.pattern.is_match(command) {
            return Some(DangerWarning {
                level: rule.level.to_string(),
                message: rule.message.to_string(),
            });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rm_rf_root() {
        let w = check_danger("rm -rf /").unwrap();
        assert_eq!(w.level, "critical");
    }

    #[test]
    fn test_dd_device() {
        let w = check_danger("dd if=/dev/zero of=/dev/sda bs=4M").unwrap();
        assert_eq!(w.level, "critical");
    }

    #[test]
    fn test_safe_command() {
        assert!(check_danger("ls -la /home").is_none());
    }

    #[test]
    fn test_fork_bomb() {
        let w = check_danger(":(){ :|:& };:").unwrap();
        assert_eq!(w.level, "critical");
    }

    #[test]
    fn test_shutdown() {
        let w = check_danger("sudo shutdown -h now").unwrap();
        assert_eq!(w.level, "warning");
    }
}
