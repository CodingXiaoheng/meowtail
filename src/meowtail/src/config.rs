// src/meowtail/src/config.rs

use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub admin_username: String,
    pub admin_password_hash: String, // 存储密码的哈希值而非明文
    pub jwt_secret: String,
    pub listen_address: String,
    pub listen_port: u16,
    pub udhcpd_enabled: bool,
}

impl Config {
    // 从文件加载配置，如果文件不存在则创建默认配置
    pub fn load_or_create() -> io::Result<Self> {
        let config_path = Self::get_config_path()?;

        if config_path.exists() {
            let config_str = fs::read_to_string(config_path)?;
            toml::from_str(&config_str)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        } else {
            println!("Configuration file not found. Creating a default one at {:?}", config_path);
            let default_config = Self::default();
            default_config.save()?;
            Ok(default_config)
        }
    }

    // 保存配置到文件
    pub fn save(&self) -> io::Result<()> {
        let config_path = Self::get_config_path()?;
        let config_str = toml::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        let mut file = fs::File::create(config_path)?;
        file.write_all(config_str.as_bytes())?;
        Ok(())
    }
    
    // 获取配置文件的路径 (与可执行文件同目录)
    fn get_config_path() -> io::Result<PathBuf> {
        let mut exe_path = env::current_exe()?;
        exe_path.pop(); // 移除可执行文件名
        Ok(exe_path.join("meowtail.toml"))
    }
}

impl Default for Config {
    fn default() -> Self {
        // 生成一个复杂的随机字符串作为 JWT Secret
        let jwt_secret: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();
        
        // 在实际应用中，这里应该使用像 argon2 或 bcrypt 这样的强哈希算法
        // 为简化示例，我们暂时还是用明文，但在注释中强调其风险
        println!("WARNING: Storing password in plain text. This is not secure for production.");
        println!("Default credentials: admin / Change_ME");

        Config {
            admin_username: "admin".to_string(),
            // 生产环境中应该存储哈希值: e.g., hash("Change_ME")
            admin_password_hash: "Change_ME".to_string(),
            jwt_secret,
            listen_address: "0.0.0.0".to_string(),
            listen_port: 81,
            udhcpd_enabled: true,
        }
    }
}

// 将配置包装在 Mutex 中，以便在多线程环境中安全地修改
pub type AppConfig = Mutex<Config>;