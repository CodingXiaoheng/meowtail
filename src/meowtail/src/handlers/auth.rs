// src/handlers/auth.rs

use actix_web::{web, get, HttpRequest, HttpResponse, Responder,HttpMessage};
use jsonwebtoken::{encode, EncodingKey, Header};
use chrono::{Utc, Duration};
use crate::models::{User, Claims}; // 使用 crate:: 路径引用项目内的模块

// 在实际应用中，密钥应该从配置或环境变量中获取
const JWT_SECRET: &[u8] = b"your-secret-key";

// login 函数...
pub async fn login(user: web::Json<User>) -> impl Responder {
    // ... (登录逻辑代码)
    if user.username == "admin" && user.password == "password" {
        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(1))
            .expect("valid timestamp")
            .timestamp();

        let claims = Claims {
            sub: user.username.clone(),
            exp: expiration as usize,
        };

        let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(JWT_SECRET))
            .unwrap();

        HttpResponse::Ok().json(serde_json::json!({ "token": token }))
    } else {
        HttpResponse::Unauthorized().finish()
    }
}

// profile 函数...
#[get("/profile")]
pub async fn profile(req: HttpRequest) -> impl Responder {
    // ... (受保护的逻辑代码)
    if let Some(claims) = req.extensions().get::<Claims>() {
        HttpResponse::Ok().json(serde_json::json!({
            "message": format!("Welcome {}!", claims.sub)
        }))
    } else {
        HttpResponse::InternalServerError().finish()
    }
}