// src/handlers/udhcpd.rs

use crate::udhcpd_manager::{StaticLease, UdhcpdError, UdhcpdManager};
use actix_web::{delete, get, post, web, HttpResponse, Responder, Scope};
use serde::Deserialize;
use serde_json::json;
use std::net::Ipv4Addr;
use std::str::FromStr;

// --- 请求体 (Payloads) 定义 ---

#[derive(Deserialize)]
struct RangePayload {
    start: String,
    end: String,
}

#[derive(Deserialize)]
struct GatewayPayload {
    gateway: String,
}

#[derive(Deserialize)]
struct DnsPayload {
    servers: Vec<String>,
}

#[derive(Deserialize)]
struct LeasePayload {
    mac: String,
    ip: String,
}

#[derive(Deserialize)]
struct RemoveLeasePayload {
    mac: String,
}

#[derive(Deserialize)]
struct InterfacePayload {
    interface: String,
}

// 新增: 修改子网掩码的请求体
#[derive(Deserialize)]
struct SubnetPayload {
    subnet: String,
}


// --- 处理器 (Handlers) ---

#[post("/start")]
async fn start(manager: web::Data<UdhcpdManager>) -> Result<impl Responder, UdhcpdError> {
    web::block(move || manager.start())
        .await
        .map_err(|e| UdhcpdError::Process(e.to_string()))??;
    Ok(HttpResponse::Ok().json(json!({"status": "udhcpd service started"})))
}

#[post("/stop")]
async fn stop(manager: web::Data<UdhcpdManager>) -> Result<impl Responder, UdhcpdError> {
    web::block(move || manager.stop())
        .await
        .map_err(|e| UdhcpdError::Process(e.to_string()))??;
    Ok(HttpResponse::Ok().json(json!({"status": "udhcpd service stopped"})))
}

#[post("/restart")]
async fn restart(manager: web::Data<UdhcpdManager>) -> Result<impl Responder, UdhcpdError> {
    web::block(move || manager.restart())
        .await
        .map_err(|e| UdhcpdError::Process(e.to_string()))??;
    Ok(HttpResponse::Ok().json(json!({"status": "udhcpd service restarted"})))
}

#[get("/status")]
async fn status(manager: web::Data<UdhcpdManager>) -> Result<impl Responder, UdhcpdError> {
    let is_running = manager.is_running();
    Ok(HttpResponse::Ok().json(json!({ "running": is_running })))
}

#[get("/config")]
async fn get_config(manager: web::Data<UdhcpdManager>) -> Result<impl Responder, UdhcpdError> {
    let config = manager.read_config()?;
    Ok(HttpResponse::Ok().json(config))
}

#[post("/config/range")]
async fn set_range(
    manager: web::Data<UdhcpdManager>,
    payload: web::Json<RangePayload>,
) -> Result<impl Responder, UdhcpdError> {
    let start_ip = Ipv4Addr::from_str(&payload.start)
        .map_err(|_| UdhcpdError::InvalidInput("Invalid start IP address".to_string()))?;
    let end_ip = Ipv4Addr::from_str(&payload.end)
        .map_err(|_| UdhcpdError::InvalidInput("Invalid end IP address".to_string()))?;

    web::block(move || manager.set_dhcp_range(start_ip, end_ip))
        .await
        .map_err(|e| UdhcpdError::Process(e.to_string()))??;

    Ok(HttpResponse::Ok().json(json!({"status": "DHCP range updated"})))
}

#[post("/config/gateway")]
async fn set_gateway(
    manager: web::Data<UdhcpdManager>,
    payload: web::Json<GatewayPayload>,
) -> Result<impl Responder, UdhcpdError> {
    let gateway_ip = Ipv4Addr::from_str(&payload.gateway)
        .map_err(|_| UdhcpdError::InvalidInput("Invalid gateway IP address".to_string()))?;

    web::block(move || manager.set_gateway(gateway_ip))
        .await
        .map_err(|e| UdhcpdError::Process(e.to_string()))??;

    Ok(HttpResponse::Ok().json(json!({"status": "Gateway updated"})))
}

// 新增: 修改子网掩码的处理器
#[post("/config/subnet")]
async fn set_subnet_mask(
    manager: web::Data<UdhcpdManager>,
    payload: web::Json<SubnetPayload>,
) -> Result<impl Responder, UdhcpdError> {
    let subnet_mask = Ipv4Addr::from_str(&payload.subnet)
        .map_err(|_| UdhcpdError::InvalidInput("Invalid subnet mask".to_string()))?;

    web::block(move || manager.set_subnet_mask(subnet_mask))
        .await
        .map_err(|e| UdhcpdError::Process(e.to_string()))??;

    Ok(HttpResponse::Ok().json(json!({"status": "Subnet mask updated"})))
}


#[post("/config/interface")]
async fn set_interface(
    manager: web::Data<UdhcpdManager>,
    payload: web::Json<InterfacePayload>,
) -> Result<impl Responder, UdhcpdError> {
    let interface_name = payload.interface.clone();
    if interface_name.is_empty() {
        return Err(UdhcpdError::InvalidInput(
            "Interface name cannot be empty".to_string(),
        ));
    }

    web::block(move || manager.set_interface(interface_name))
        .await
        .map_err(|e| UdhcpdError::Process(e.to_string()))??;

    Ok(HttpResponse::Ok().json(json!({ "status": "Interface updated" })))
}

#[post("/config/dns")]
async fn set_dns(
    manager: web::Data<UdhcpdManager>,
    payload: web::Json<DnsPayload>,
) -> Result<impl Responder, UdhcpdError> {
    let servers: Vec<Ipv4Addr> = payload
        .servers
        .iter()
        .map(|s| {
            Ipv4Addr::from_str(s)
                .map_err(|_| UdhcpdError::InvalidInput(format!("Invalid DNS IP address: {}", s)))
        })
        .collect::<Result<_, _>>()?;

    web::block(move || manager.set_dns_servers(servers))
        .await
        .map_err(|e| UdhcpdError::Process(e.to_string()))??;

    Ok(HttpResponse::Ok().json(json!({"status": "DNS servers updated"})))
}

#[post("/config/lease")]
async fn add_lease(
    manager: web::Data<UdhcpdManager>,
    payload: web::Json<LeasePayload>,
) -> Result<impl Responder, UdhcpdError> {
    let ip = Ipv4Addr::from_str(&payload.ip)
        .map_err(|_| UdhcpdError::InvalidInput("Invalid lease IP address".to_string()))?;
    let lease = StaticLease {
        mac: payload.mac.clone(),
        ip,
    };

    web::block(move || manager.add_or_update_static_lease(lease))
        .await
        .map_err(|e| UdhcpdError::Process(e.to_string()))??;

    Ok(HttpResponse::Ok().json(json!({"status": "Static lease added/updated"})))
}

#[delete("/config/lease")]
async fn remove_lease(
    manager: web::Data<UdhcpdManager>,
    payload: web::Json<RemoveLeasePayload>,
) -> Result<impl Responder, UdhcpdError> {
    let mac = payload.mac.clone();
    web::block(move || manager.remove_static_lease(&mac))
        .await
        .map_err(|e| UdhcpdError::Process(e.to_string()))??;

    Ok(HttpResponse::Ok().json(json!({"status": "Static lease removed"})))
}

pub fn service() -> Scope {
    web::scope("/udhcpd")
        .service(start)
        .service(stop)
        .service(restart)
        .service(status)
        .service(get_config)
        .service(set_range)
        .service(set_gateway)
        .service(set_interface)
        .service(set_subnet_mask) // 新增: 注册接口路由
        .service(set_dns)
        .service(add_lease)
        .service(remove_lease)
}