use crate::AppState;
use actix_web::{web, HttpResponse};
use uuid::Uuid;

pub async fn list_schedules(
    // TODO: Add authentication
    _data: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    // TODO: Implementar llistat d'horaris
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "schedules": [],
        "message": "List schedules endpoint to be implemented"
    })))
}

pub async fn get_today_schedules(
    // TODO: Add authentication
    _data: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    // TODO: Implementar obtenció d'horaris d'avui
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "schedules": [],
        "message": "Today schedules endpoint to be implemented"
    })))
}

pub async fn rebuild_schedules(
    // TODO: Add authentication
    _data: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    // TODO: Implementar reconstrucció d'horaris
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Rebuild schedules endpoint to be implemented"
    })))
}
