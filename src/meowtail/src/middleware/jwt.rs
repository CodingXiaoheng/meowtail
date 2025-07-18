// src/middleware/jwt.rs

use std::future::{ready, Ready};
use std::rc::Rc;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, web,
};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{decode, DecodingKey, Validation};
use crate::models::Claims;
use crate::config::AppConfig;

pub struct JwtMiddleware;

impl<S, B> Transform<S, ServiceRequest> for JwtMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtMiddlewareService { service: Rc::new(service) }))
    }
}

pub struct JwtMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for JwtMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // --- FIX STARTS HERE ---
        // Clone the necessary data from `req` before the async block.
        // `web::Data` is an Arc, so cloning is cheap.
        let config = req.app_data::<web::Data<AppConfig>>().cloned();
        let auth_header = req.headers().get("Authorization").cloned();
        let service = self.service.clone();

        Box::pin(async move {
            // Now we use the cloned, owned data inside the future.
            let config = match config {
                Some(c) => c,
                None => {
                    eprintln!("Critical: AppConfig not found in application state.");
                    return Err(actix_web::error::ErrorInternalServerError("Server configuration error."));
                }
            };
            
            let jwt_secret = {
                let app_config = config.lock().unwrap();
                app_config.jwt_secret.clone()
            };

            if let Some(auth_header) = auth_header {
                if let Ok(auth_str) = auth_header.to_str() {
                    if auth_str.starts_with("Bearer ") {
                        let token = &auth_str[7..];
                        let decoding_key = DecodingKey::from_secret(jwt_secret.as_ref());
                        let validation = Validation::default();

                        if let Ok(token_data) = decode::<Claims>(token, &decoding_key, &validation) {
                            req.extensions_mut().insert(token_data.claims);
                            // `req` is moved into service.call() here, which is valid.
                            return service.call(req).await;
                        }
                    }
                }
            }

            Err(actix_web::error::ErrorUnauthorized("Invalid token or authorization header."))
        })
        // --- FIX ENDS HERE ---
    }
}