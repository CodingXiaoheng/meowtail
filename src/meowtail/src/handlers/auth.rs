// src/handlers/auth.rs

use actix_web::{web, get, post, HttpRequest, HttpResponse, Responder, HttpMessage};
use jsonwebtoken::{encode, EncodingKey, Header};
use chrono::{Utc, Duration};
use crate::models::{User, Claims, ChangePasswordPayload};
use crate::config::AppConfig; // 引入 AppConfig

// POST /login
#[post("/login")]
pub async fn login(
    user: web::Json<User>,
    config: web::Data<AppConfig>,
) -> impl Responder {
    let app_config = config.lock().unwrap();
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

        // 这里用 match 显式处理错误，而不是 `?`
        let token = match encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(app_config.jwt_secret.as_ref()),
        ) {
            Ok(t) => t,
            Err(_) => return HttpResponse::InternalServerError().finish(),
        };

        HttpResponse::Ok().json(serde_json::json!({ "token": token }))
    } else {
        HttpResponse::Unauthorized().finish()
    }
}

// GET /logined
#[get("/logined")]
pub async fn logined() -> impl Responder {
    // 由于 JWT 中间件会在未通过鉴权时截断请求，
    // 能执行到这里的都已登录
    HttpResponse::Ok().json(serde_json::json!({
        "result": true
    }))
}

// POST /change-password
#[post("/change-password")]
pub async fn change_password(
    req: HttpRequest,
    payload: web::Json<ChangePasswordPayload>,
    config: web::Data<AppConfig>,
) -> impl Responder {
    // 1. 验证用户是否已登录 (通过 JWT 中间件)
    if let Some(claims) = req.extensions().get::<Claims>() {
        let mut app_config = config.lock().unwrap();

        // 2. 确保是管理员自己修改密码
        if claims.sub != app_config.admin_username {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "error": "You can only change your own password."
            }));
        }

        // 3. 更新密码 (生产环境应哈希新密码)
        app_config.admin_password_hash = payload.new_password.clone();

        // 4. 保存到配置文件
        if let Err(e) = app_config.save() {
            eprintln!("Failed to save configuration: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to save new password."
            }));
        }

        HttpResponse::Ok().json(serde_json::json!({
            "message": "Password updated successfully."
        }))
    } else {
        // JWT 中间件已拦截，这里作为双重保险
        HttpResponse::Unauthorized().finish()
    }
}
