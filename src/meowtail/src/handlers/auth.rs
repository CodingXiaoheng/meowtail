// src/handlers/auth.rs

use actix_web::{web, get, post, HttpRequest, HttpResponse, Responder, HttpMessage};
use jsonwebtoken::{encode, EncodingKey, Header};
use chrono::{Utc, Duration};
use crate::models::{User, Claims, ChangePasswordPayload};
use crate::config::AppConfig; // 引入 AppConfig

// login 函数修改为从配置中获取凭据
pub async fn login(
    user: web::Json<User>,
    config: web::Data<AppConfig>, // 注入配置
) -> impl Responder {
    let app_config = config.lock().unwrap(); // 获取配置锁

    // 警告：这里的密码比较是明文的。在生产中，应该比较密码的哈希值。
    // let is_password_correct = bcrypt::verify(&user.password, &app_config.admin_password_hash).unwrap_or(false);
    let is_password_correct = user.password == app_config.admin_password_hash;

    if user.username == app_config.admin_username && is_password_correct {
        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(1))
            .expect("valid timestamp")
            .timestamp();

        let claims = Claims {
            sub: user.username.clone(),
            exp: expiration as usize,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(app_config.jwt_secret.as_ref()), // 使用配置中的 secret
        )
        .unwrap();

        HttpResponse::Ok().json(serde_json::json!({ "token": token }))
    } else {
        HttpResponse::Unauthorized().finish()
    }
}

// profile 函数保持不变
#[get("/profile")]
pub async fn profile(req: HttpRequest) -> impl Responder {
    if let Some(claims) = req.extensions().get::<Claims>() {
        HttpResponse::Ok().json(serde_json::json!({
            "message": format!("Welcome {}!", claims.sub)
        }))
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

// 新增: 修改密码的接口
#[post("/change-password")]
pub async fn change_password(
    req: HttpRequest,
    payload: web::Json<ChangePasswordPayload>,
    config: web::Data<AppConfig>,
) -> impl Responder {
    // 1. 验证用户是否已登录 (通过 JWT)
    if let Some(claims) = req.extensions().get::<Claims>() {
        let mut app_config = config.lock().unwrap();

        // 2. 确保是管理员自己修改密码
        if claims.sub != app_config.admin_username {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "error": "You can only change your own password."
            }));
        }

        // 3. 更新密码 (同样，生产环境应哈希新密码)
        println!("Password for '{}' changed.", app_config.admin_username);
        app_config.admin_password_hash = payload.new_password.clone();

        // 4. 保存到配置文件
        if let Err(e) = app_config.save() {
            eprintln!("Failed to save configuration after password change: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to save new password."
            }));
        }

        HttpResponse::Ok().json(serde_json::json!({
            "message": "Password updated successfully."
        }))
    } else {
        // 如果没有 token 或 token 无效，中间件已经拦截了，但作为双重保障
        HttpResponse::Unauthorized().finish()
    }
}