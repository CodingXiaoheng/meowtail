use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortMapRule {
    pub protocol: String,
    pub external_port: u16,
    pub internal_ip: String,
    pub internal_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PortMapConfig {
    pub external_interface: String,
    pub rules: Vec<PortMapRule>,
}

pub struct PortMapManager {
    config: Mutex<PortMapConfig>,
    file_path: PathBuf,
}

impl PortMapManager {
    pub fn new<P: Into<PathBuf>>(path: P) -> io::Result<Self> {
        let file_path = path.into();
        let config = if file_path.exists() {
            let content = fs::read_to_string(&file_path)?;
            toml::from_str(&content).unwrap_or_default()
        } else {
            PortMapConfig::default()
        };
        Ok(Self { config: Mutex::new(config), file_path })
    }

    fn save(&self) -> io::Result<()> {
        let cfg = self.config.lock().unwrap();
        let content = toml::to_string_pretty(&*cfg).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let mut file = fs::File::create(&self.file_path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    fn run_iptables(args: &[&str]) -> io::Result<()> {
        Command::new("iptables").args(args).status()?;
        Ok(())
    }

    fn apply_rule_internal(rule: &PortMapRule, iface: &str, action: &str) -> io::Result<()> {
        let prerouting = [
            "-t", "nat", action, "PREROUTING",
            "-i", iface,
            "-p", &rule.protocol,
            "--dport", &rule.external_port.to_string(),
            "-j", "DNAT",
            "--to-destination",
            &format!("{}:{}", rule.internal_ip, rule.internal_port),
        ];
        Self::run_iptables(&prerouting)?;
        let postrouting = [
            "-t", "nat", action, "POSTROUTING",
            "-o", iface,
            "-p", &rule.protocol,
            "--dport", &rule.internal_port.to_string(),
            "-d", &rule.internal_ip,
            "-j", "MASQUERADE",
        ];
        Self::run_iptables(&postrouting)
    }

    pub fn apply_rule(&self, rule: &PortMapRule) -> io::Result<()> {
        let cfg = self.config.lock().unwrap();
        Self::apply_rule_internal(rule, &cfg.external_interface, "-A")
    }

    pub fn remove_rule(&self, rule: &PortMapRule) -> io::Result<()> {
        let cfg = self.config.lock().unwrap();
        Self::apply_rule_internal(rule, &cfg.external_interface, "-D")
    }

    pub fn apply_all(&self) -> io::Result<()> {
        let cfg = self.config.lock().unwrap();
        for r in &cfg.rules {
            let _ = Self::apply_rule_internal(r, &cfg.external_interface, "-A");
        }
        Ok(())
    }

    pub fn add_rule(&self, rule: PortMapRule) -> io::Result<()> {
        {
            let mut cfg = self.config.lock().unwrap();
            Self::apply_rule_internal(&rule, &cfg.external_interface, "-A")?;
            cfg.rules.push(rule);
        }
        self.save()
    }

    pub fn delete_rule(&self, rule: PortMapRule) -> io::Result<()> {
        {
            let mut cfg = self.config.lock().unwrap();
            if let Some(pos) = cfg.rules.iter().position(|r| r == &rule) {
                Self::apply_rule_internal(&rule, &cfg.external_interface, "-D")?;
                cfg.rules.remove(pos);
            }
        }
        self.save()
    }

    pub fn set_interface(&self, iface: String) -> io::Result<()> {
        let old_iface: String;
        let rules_clone: Vec<PortMapRule>;

        {
            let mut cfg = self.config.lock().unwrap();

            // 1. If the interface is not changing, there's nothing to do.
            if cfg.external_interface == iface {
                return Ok(());
            }

            // 2. Store the old interface name and a copy of the rules before changing them.
            old_iface = cfg.external_interface.clone();
            rules_clone = cfg.rules.clone();

            // 3. Update the config with the new interface name.
            cfg.external_interface = iface;
        } // The mutex lock is released here.

        // 4. If an old interface was configured, remove all its rules.
        //    This prevents leaving orphaned rules in iptables.
        if !old_iface.is_empty() {
            for rule in &rules_clone {
                // Ignore errors during removal, as a rule might already be gone.
                // This makes the cleanup process more robust.
                let _ = Self::apply_rule_internal(rule, &old_iface, "-D");
            }
        }

        // 5. Save the new configuration to the file.
        self.save()?;

        // 6. Apply all rules using the new interface.
        self.apply_all()
    }

    pub fn config(&self) -> PortMapConfig {
        self.config.lock().unwrap().clone()
    }
}
