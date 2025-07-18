// src/middleware/jwt.rs

use std::future::{ready, Ready};
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{decode, DecodingKey, Validation};
use crate::models::Claims; // 引用 Claims 模型

// ... (JwtMiddleware 和 JwtMiddlewareService 的完整代码)
// 注意：需要将之前示例中的 JWT_SECRET 常量也移到这里或一个共享的配置模块中
const JWT_SECRET: &[u8] = b"your-secret-key";

pub struct JwtMiddleware;

// ...

pub struct JwtMiddlewareService<S> {
    service: S,
}

// ... impl Service ...
impl<S, B> Service<ServiceRequest> for JwtMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    // Define the required associated types
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    // Place the forward_ready! macro inside the impl block
    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let auth_header = req.headers().get("Authorization");

        if let Some(auth_header) = auth_header {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token = &auth_str[7..];
                    let decoding_key = DecodingKey::from_secret(JWT_SECRET);
                    let validation = Validation::default();

                    if let Ok(token_data) = decode::<Claims>(token, &decoding_key, &validation) {
                        req.extensions_mut().insert(token_data.claims);
                        let fut = self.service.call(req);
                        return Box::pin(async move {
                            let res = fut.await?;
                            Ok(res)
                        });
                    }
                }
            }
        }

        Box::pin(async move {
            Err(actix_web::error::ErrorUnauthorized("Invalid token or authorization header."))
        })
    }
}

// ... impl Transform ...
impl<S, B> Transform<S, ServiceRequest> for JwtMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtMiddlewareService { service }))
    }
}