use std::fs::File;
use std::process;
use std::env;
use std::path::Path;
use std::sync::Mutex;

use actix_web::{web, App, HttpServer};
use daemonize::Daemonize;
use nix::unistd::getuid;
use actix_files as fs;

// 引入模块
mod handlers;
mod middleware;
mod models;
mod udhcpd_manager;
mod config; // 引入新的 config 模块

use crate::udhcpd_manager::UdhcpdManager;
use crate::config::Config;

fn main() {
    // 检查是否以 root 用户运行
    if !getuid().is_root() {
        eprintln!("Error: This program must be run as root.");
        process::exit(1);
    }

    // --- 工作目录切换 ---
    // 获取可执行文件所在的目录
    if let Ok(mut exe_path) = env::current_exe() {
        exe_path.pop(); // 移除文件名,剩下路径
        // 切换工作目录到可执行文件所在的目录
        if let Err(e) = env::set_current_dir(&exe_path) {
            eprintln!("Failed to change working directory to {:?}: {}", exe_path, e);
            process::exit(1);
        }
        println!("Working directory changed to: {:?}", exe_path);
    } else {
        eprintln!("Failed to get current executable path.");
        process::exit(1);
    }

    // --- 加载配置 ---
    let app_config = match Config::load_or_create() {
        Ok(config) => web::Data::new(Mutex::new(config)),
        Err(e) => {
            eprintln!("Failed to load or create configuration: {}", e);
            process::exit(1);
        }
    };
    
    // --- 守护进程设置 (保持不变) ---
    let stdout = File::create("/tmp/daemon.out").unwrap();
    let stderr = File::create("/tmp/daemon.err").unwrap();

    let daemonize = Daemonize::new()
        .pid_file("/tmp/my_web_app.pid")
        .chown_pid_file(true)
        .working_directory(env::current_dir().unwrap()) // 使用新的工作目录
        .user("root")
        .group("root")
        .umask(0o027)
        .stdout(stdout)
        .stderr(stderr)
        .privileged_action(|| {
            println!("Privileged action executed before dropping privileges.");
        });

    // 启动守护进程
    match daemonize.start() {
        Ok(_) => {
            println!("Success, daemonized process started.");

            let sys = actix_web::rt::System::new();
            
            sys.block_on(async {
                // --- UdhcpdManager 初始化 (保持不变) ---
                let config_path = "./udhcpd.conf";
                let pid_path = "/tmp/meowtail_udhcpd.pid";
                let executable_path = "udhcpd"; 

                let manager = UdhcpdManager::new(executable_path, config_path, pid_path);
                if let Err(e) = manager.create_config_with_defaults("eth0", false) {
                    if e.to_string().contains("already exists") {
                        println!("Configuration file '{}' already exists, using it.", config_path);
                    } else {
                        eprintln!("Failed to create default config file: {}", e);
                        process::exit(1); 
                    }
                } else {
                    println!("Created default configuration file at '{}'.", config_path);
                }
                
                let manager_data = web::Data::new(manager);

                println!("Starting web server at http://127.0.0.1:8080");

                if let Err(e) = HttpServer::new(move || {
                    App::new()
                        .app_data(manager_data.clone())
                        .app_data(app_config.clone())
                        // 公开的 API 路由
                        .route("/login", web::post().to(handlers::auth::login))
                        // 受保护的 API 路由组
                        .service(
                            web::scope("/api")
                                .wrap(middleware::jwt::JwtMiddleware)
                                .service(handlers::auth::profile)
                                .service(handlers::auth::change_password)
                                .service(handlers::udhcpd::service()),
                        )
                        // --- 关键修改：在这里添加静态文件服务 ---
                        // 这个服务应该在所有 API 路由之后注册，以避免冲突
                        .service(fs::Files::new("/", "./static").index_file("index.html"))
                })
                .bind(("172.16.0.1", 81))
                .unwrap()
                .run()
                .await
                {
                    eprintln!("Web server failed to start: {}", e);
                }
            });
        }
        Err(e) => eprintln!("Error, {}", e),
    }
}