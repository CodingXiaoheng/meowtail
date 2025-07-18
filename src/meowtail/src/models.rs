use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Subject (用户名)
    pub exp: usize,  // Expiration Time
}

// 新增: 用于修改密码请求的结构体
#[derive(Debug, Deserialize)]
pub struct ChangePasswordPayload {
    pub new_password: String,
}