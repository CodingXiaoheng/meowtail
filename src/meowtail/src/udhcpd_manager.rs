// src/udhcpd_manager.rs

use std::fs::{self, File};
use std::io::{self, BufRead};
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
// 新增: 引入 Mutex 来解决并发问题
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 自定义错误类型，用于封装模块中可能发生的所有错误。
#[derive(Error, Debug)]
pub enum UdhcpdError {
    #[error("I/O Error: {0}")]
    Io(#[from] io::Error),
    #[error("Process Error: {0}")]
    Process(String),
    #[error("PID file error: {0}")]
    PidFile(String),
    #[error("Configuration parsing error on line: {0}")]
    ConfigParse(String),
    #[error("Nix (Unix-like system call) error: {0}")]
    Nix(#[from] nix::Error),
    #[error("Invalid IP address: {0}")]
    InvalidIp(#[from] std::net::AddrParseError),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

// 实现 ResponseError 以便 Actix-web 可以自动将我们的错误转换为 HTTP 响应
impl ResponseError for UdhcpdError {
    fn status_code(&self) -> StatusCode {
        match *self {
            UdhcpdError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            UdhcpdError::Process(_) => StatusCode::CONFLICT,
            UdhcpdError::PidFile(_) => StatusCode::NOT_FOUND,
            UdhcpdError::ConfigParse(_) => StatusCode::INTERNAL_SERVER_ERROR,
            UdhcpdError::Nix(_) => StatusCode::INTERNAL_SERVER_ERROR,
            UdhcpdError::InvalidIp(_) => StatusCode::BAD_REQUEST,
            UdhcpdError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(serde_json::json!({ "error": self.to_string() }))
    }
}

pub type Result<T> = std::result::Result<T, UdhcpdError>;

/// 代表一个静态租约配置 (添加了 Serialize/Deserialize)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaticLease {
    pub mac: String,
    pub ip: Ipv4Addr,
}

/// 代表 udhcpd.conf 文件的完整配置 (添加了 Serialize/Deserialize)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UdhcpdConfig {
    pub start: Option<Ipv4Addr>,
    pub end: Option<Ipv4Addr>,
    pub interface: Option<String>,
    pub subnet_mask: Option<Ipv4Addr>,
    pub dns_servers: Vec<Ipv4Addr>,
    pub router: Option<Ipv4Addr>,
    pub static_leases: Vec<StaticLease>,
    #[serde(skip)]
    remaining_lines: Vec<String>,
}

impl UdhcpdConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        let reader = io::BufReader::new(file);
        let mut config = UdhcpdConfig::default();

        for line in reader.lines() {
            let line = line?;
            let trimmed_line = line.trim();
            if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed_line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "start" if parts.len() > 1 => config.start = Some(Ipv4Addr::from_str(parts[1])?),
                "end" if parts.len() > 1 => config.end = Some(Ipv4Addr::from_str(parts[1])?),
                "interface" if parts.len() > 1 => config.interface = Some(parts[1].to_string()),
                "option" if parts.len() > 2 => match parts[1] {
                    "subnet" => config.subnet_mask = Some(Ipv4Addr::from_str(parts[2])?),
                    "dns" => {
                        config.dns_servers = parts[2..]
                            .iter()
                            .map(|s| Ipv4Addr::from_str(s).unwrap())
                            .collect()
                    }
                    "router" => config.router = Some(Ipv4Addr::from_str(parts[2])?),
                    _ => config.remaining_lines.push(line.clone()),
                },
                "static_lease" if parts.len() > 2 => {
                    config.static_leases.push(StaticLease {
                        mac: parts[1].to_string(),
                        ip: Ipv4Addr::from_str(parts[2])?,
                    });
                }
                _ => config.remaining_lines.push(line.clone()),
            }
        }
        Ok(config)
    }

    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut content = String::new();

        if let Some(iface) = &self.interface {
            content.push_str(&format!("interface {}\n", iface));
        }
        if let Some(start) = self.start {
            content.push_str(&format!("start {}\n", start));
        }
        if let Some(end) = self.end {
            content.push_str(&format!("end {}\n", end));
        }
        if let Some(subnet) = self.subnet_mask {
            content.push_str(&format!("option subnet {}\n", subnet));
        }
        if let Some(router) = self.router {
            content.push_str(&format!("option router {}\n", router));
        }
        if !self.dns_servers.is_empty() {
            let dns_list: Vec<String> = self.dns_servers.iter().map(|ip| ip.to_string()).collect();
            content.push_str(&format!("option dns {}\n", dns_list.join(" ")));
        }
        for lease in &self.static_leases {
            content.push_str(&format!("static_lease {} {}\n", lease.mac, lease.ip));
        }
        for line in &self.remaining_lines {
            content.push_str(&format!("{}\n", line));
        }

        fs::write(path, content)?;
        Ok(())
    }
}

pub struct UdhcpdManager {
    executable_path: String,
    config_path: PathBuf,
    pid_path: PathBuf,
    // 修改: 增加一个互斥锁来保护对配置文件的并发访问
    config_lock: Mutex<()>,
}

impl UdhcpdManager {
    pub fn new<P1: Into<PathBuf>, P2: Into<PathBuf>>(
        executable_path: &str,
        config_path: P1,
        pid_path: P2,
    ) -> Self {
        UdhcpdManager {
            executable_path: executable_path.to_string(),
            config_path: config_path.into(),
            pid_path: pid_path.into(),
            // 修改: 初始化互斥锁
            config_lock: Mutex::new(()),
        }
    }

    pub fn start(&self) -> Result<()> {
        if self.is_running() {
            return Err(UdhcpdError::Process(
                "udhcpd is already running.".to_string(),
            ));
        }
        if !self.config_path.exists() {
            return Err(UdhcpdError::Io(io::Error::new(
                io::ErrorKind::NotFound,
                "Configuration file not found. Cannot start service.",
            )));
        }

        let child = Command::new(&self.executable_path)
            .arg(self.config_path.to_str().unwrap())
            .spawn()
            .map_err(|e| UdhcpdError::Io(e))?;

        let pid = child.id();

        fs::write(&self.pid_path, pid.to_string())
            .map_err(|e| UdhcpdError::PidFile(format!("Failed to write PID file: {}", e)))?;

        println!("Started udhcpd with PID: {}. PID file created at {:?}", pid, self.pid_path);
        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        let pid = match self.get_pid() {
            Ok(pid) => pid,
            Err(_) => return Ok(()),
        };

        signal::kill(pid, Signal::SIGTERM)?;

        for _ in 0..30 {
            if !self.is_process_alive(pid) {
                let _ = fs::remove_file(&self.pid_path);
                return Ok(());
            }
            thread::sleep(Duration::from_millis(100));
        }

        if self.is_process_alive(pid) {
            println!("Process did not respond to SIGTERM, sending SIGKILL...");
            signal::kill(pid, Signal::SIGKILL)?;
        }

        let _ = fs::remove_file(&self.pid_path);
        Ok(())
    }

    pub fn restart(&self) -> Result<()> {
        self.stop()?;
        thread::sleep(Duration::from_millis(200));
        self.start()
    }

    pub fn is_running(&self) -> bool {
        match self.get_pid() {
            Ok(pid) => self.is_process_alive(pid),
            Err(_) => false,
        }
    }

    fn get_pid(&self) -> Result<Pid> {
        if !self.pid_path.exists() {
            return Err(UdhcpdError::PidFile("PID file not found.".to_string()));
        }
        let content = fs::read_to_string(&self.pid_path)?;
        let pid_val = content
            .trim()
            .parse::<i32>()
            .map_err(|_| UdhcpdError::PidFile("Failed to parse PID from file.".to_string()))?;
        Ok(Pid::from_raw(pid_val))
    }

    fn is_process_alive(&self, pid: Pid) -> bool {
        signal::kill(pid, None).is_ok()
    }

    pub fn read_config(&self) -> Result<UdhcpdConfig> {
        UdhcpdConfig::from_file(&self.config_path)
    }

    pub fn write_config(&self, config: &UdhcpdConfig) -> Result<()> {
        config.write_to_file(&self.config_path)
    }

    pub fn create_config_with_defaults(&self, interface: &str, overwrite: bool) -> Result<()> {
        // 修改: 在修改文件前获取锁
        let _guard = self.config_lock.lock().map_err(|e| UdhcpdError::Process(format!("Failed to acquire config lock: {}", e)))?;

        if self.config_path.exists() && !overwrite {
            return Err(UdhcpdError::Io(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "Configuration file already exists. Set overwrite to true to replace it.",
            )));
        }

        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let default_config = UdhcpdConfig {
            interface: Some(interface.to_string()),
            start: Some(Ipv4Addr::new(192, 168, 1, 100)),
            end: Some(Ipv4Addr::new(192, 168, 1, 200)),
            subnet_mask: Some(Ipv4Addr::new(255, 255, 255, 0)),
            router: Some(Ipv4Addr::new(192, 168, 1, 1)),
            dns_servers: vec![Ipv4Addr::new(8, 8, 8, 8), Ipv4Addr::new(1, 1, 1, 1)],
            static_leases: vec![],
            remaining_lines: vec!["# Auto-generated by UdhcpdManager".to_string()],
        };

        self.write_config(&default_config)
    }

    // --- 以下所有修改配置的方法都增加了锁保护 ---

    pub fn set_dhcp_range(&self, start: Ipv4Addr, end: Ipv4Addr) -> Result<()> {
        let _guard = self.config_lock.lock().map_err(|e| UdhcpdError::Process(format!("Failed to acquire config lock: {}", e)))?;
        let mut config = self.read_config()?;
        config.start = Some(start);
        config.end = Some(end);
        self.write_config(&config)
    }

    pub fn set_subnet_mask(&self, mask: Ipv4Addr) -> Result<()> {
        let _guard = self.config_lock.lock().map_err(|e| UdhcpdError::Process(format!("Failed to acquire config lock: {}", e)))?;
        let mut config = self.read_config()?;
        config.subnet_mask = Some(mask);
        self.write_config(&config)
    }

    pub fn set_dns_servers(&self, servers: Vec<Ipv4Addr>) -> Result<()> {
        let _guard = self.config_lock.lock().map_err(|e| UdhcpdError::Process(format!("Failed to acquire config lock: {}", e)))?;
        let mut config = self.read_config()?;
        config.dns_servers = servers;
        self.write_config(&config)
    }

    pub fn set_gateway(&self, gateway: Ipv4Addr) -> Result<()> {
        let _guard = self.config_lock.lock().map_err(|e| UdhcpdError::Process(format!("Failed to acquire config lock: {}", e)))?;
        let mut config = self.read_config()?;
        config.router = Some(gateway);
        self.write_config(&config)
    }

    pub fn set_interface(&self, interface: String) -> Result<()> {
        let _guard = self.config_lock.lock().map_err(|e| UdhcpdError::Process(format!("Failed to acquire config lock: {}", e)))?;
        let mut config = self.read_config()?;
        config.interface = Some(interface);
        self.write_config(&config)
    }

    pub fn add_or_update_static_lease(&self, lease: StaticLease) -> Result<()> {
        let _guard = self.config_lock.lock().map_err(|e| UdhcpdError::Process(format!("Failed to acquire config lock: {}", e)))?;
        let mut config = self.read_config()?;
        if let Some(existing_lease) = config.static_leases.iter_mut().find(|l| l.mac == lease.mac)
        {
            *existing_lease = lease;
        } else {
            config.static_leases.push(lease);
        }
        self.write_config(&config)
    }

    pub fn remove_static_lease(&self, mac_address: &str) -> Result<()> {
        let _guard = self.config_lock.lock().map_err(|e| UdhcpdError::Process(format!("Failed to acquire config lock: {}", e)))?;
        let mut config = self.read_config()?;
        config.static_leases.retain(|l| l.mac != mac_address);
        self.write_config(&config)
    }
}