use actix_web::{delete, get, post, web, HttpResponse, Responder, Scope};
use serde::Deserialize;

use crate::portmap_manager::{PortMapManager, PortMapRule};

#[derive(Deserialize)]
struct RulePayload {
    protocol: String,
    external_port: u16,
    internal_ip: String,
    internal_port: u16,
}

#[derive(Deserialize)]
struct InterfacePayload {
    interface: String,
}

#[get("/config")]
async fn get_config(manager: web::Data<PortMapManager>) -> impl Responder {
    HttpResponse::Ok().json(manager.config())
}

#[post("/rule")]
async fn add_rule(
    manager: web::Data<PortMapManager>,
    payload: web::Json<RulePayload>,
) -> impl Responder {
    let rule = PortMapRule {
        protocol: payload.protocol.clone(),
        external_port: payload.external_port,
        internal_ip: payload.internal_ip.clone(),
        internal_port: payload.internal_port,
    };

    if let Err(e) = manager.add_rule(rule) {
        return HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": e.to_string()}));
    }
    HttpResponse::Ok().json(serde_json::json!({"status": "rule added"}))
}

#[delete("/rule")]
async fn delete_rule(
    manager: web::Data<PortMapManager>,
    payload: web::Json<RulePayload>,
) -> impl Responder {
    let rule = PortMapRule {
        protocol: payload.protocol.clone(),
        external_port: payload.external_port,
        internal_ip: payload.internal_ip.clone(),
        internal_port: payload.internal_port,
    };

    if let Err(e) = manager.delete_rule(rule) {
        return HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": e.to_string()}));
    }
    HttpResponse::Ok().json(serde_json::json!({"status": "rule removed"}))
}

#[post("/interface")]
async fn set_interface(
    manager: web::Data<PortMapManager>,
    payload: web::Json<InterfacePayload>,
) -> impl Responder {
    if let Err(e) = manager.set_interface(payload.interface.clone()) {
        return HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": e.to_string()}));
    }
    HttpResponse::Ok().json(serde_json::json!({"status": "interface updated"}))
}

pub fn service() -> Scope {
    web::scope("/portmap")
        .service(get_config)
        .service(add_rule)
        .service(delete_rule)
        .service(set_interface)
}
