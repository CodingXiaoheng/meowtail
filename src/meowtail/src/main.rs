use std::fs::File;
use std::process;

use actix_web::{web, App, HttpServer};
use daemonize::Daemonize;
use nix::unistd::getuid;

// 引入模块
mod handlers;
mod middleware;
mod models;
mod udhcpd_manager;

use crate::udhcpd_manager::UdhcpdManager;

fn main() {
    // 检查是否以 root 用户运行
    if !getuid().is_root() {
        eprintln!("Error: This program must be run as root.");
        process::exit(1);
    }

    // 为标准输出和标准错误创建日志文件
    let stdout = File::create("/tmp/daemon.out").unwrap();
    let stderr = File::create("/tmp/daemon.err").unwrap();

    // 配置守护进程
    let daemonize = Daemonize::new()
        .pid_file("/tmp/my_web_app.pid")
        .chown_pid_file(true)
        .working_directory("/tmp")
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

            // 在守护进程中初始化和运行 Web 服务
            // 使用 actix_web 的 System 来管理异步运行时
            let sys = actix_web::rt::System::new();
            
            // 在 System 中执行我们的异步 main 逻辑
            sys.block_on(async {
                // --- UdhcpdManager 初始化 ---
                let config_path = "./udhcpd.conf";
                let pid_path = "/tmp/meowtail_udhcpd.pid";
                let executable_path = "udhcpd"; // 依赖系统 PATH

                // 1. 初始化 Manager
                let manager = UdhcpdManager::new(executable_path, config_path, pid_path);

                // 2. 检查配置文件是否存在，不存在则创建
                // 使用 overwrite: false 来避免覆盖现有文件
                if let Err(e) = manager.create_config_with_defaults("eth0", false) {
                    // 如果错误不是 "AlreadyExists"，则打印错误
                    if e.to_string().contains("already exists") {
                        println!("Configuration file '{}' already exists, using it.", config_path);
                    } else {
                        eprintln!("Failed to create default config file: {}", e);
                        process::exit(1); // 创建失败则退出
                    }
                } else {
                    println!("Created default configuration file at '{}'.", config_path);
                }
                
                // 3. 将 manager 包装在 web::Data 中以进行全局共享
                let manager_data = web::Data::new(manager);

                println!("Starting web server at http://127.0.0.1:8080");

                // --- 启动 HTTP 服务器 ---
                if let Err(e) = HttpServer::new(move || {
                    App::new()
                        // 使用 app_data 注册全局共享状态
                        .app_data(manager_data.clone())
                        // 公开路由
                        .route("/login", web::post().to(handlers::auth::login))
                        // 受保护的路由组
                        .service(
                            web::scope("/api")
                                .wrap(middleware::jwt::JwtMiddleware) // 应用 JWT 中间件
                                .service(handlers::auth::profile)
                                .service(handlers::udhcpd::service()),
                        )
                })
                .bind(("127.0.0.1", 8080))
                .unwrap() // 在守护进程中，如果绑定失败，我们希望它 panic
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
