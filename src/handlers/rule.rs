use crate::{
    models::rule::{Rule, NewRule},
    AppState,
};
use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateRuleRequest {
    pub device_id: Uuid,
    pub rule_type: String,
    pub params: serde_json::Value,
    pub active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRuleRequest {
    pub rule_type: Option<String>,
    pub params: Option<serde_json::Value>,
    pub active: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct RuleResponse {
    pub id: Uuid,
    pub device_id: Uuid,
    pub rule_type: String,
    pub params: serde_json::Value,
    pub active: bool,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

// Llistar totes les regles de l'usuari
pub async fn list_rules(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    // Obtenir user_id del JWT token
    let auth_header = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Missing or invalid token"))?;
    
    let claims = crate::handlers::auth::verify_jwt(auth_header, &data.jwt_secret)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid user ID in token"))?;
    
    let pool = &data.db_pool;
    let conn = pool.get().await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Database connection failed"))?;
    
    let rules = conn.interact(move |conn| {
        use crate::schema::{rules, devices};
        
        // Obtenir totes les regles dels dispositius de l'usuari
        rules::table
            .inner_join(devices::table.on(devices::id.eq(rules::device_id)))
            .filter(devices::user_id.eq(user_id))
            .select(rules::all_columns)
            .load::<Rule>(conn)
    })
    .await
    .map_err(|_| actix_web::error::ErrorInternalServerError("Database query failed"))?
    .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to get rules"))?;
    
    Ok(HttpResponse::Ok().json(json!({
        "rules": rules
    })))
}

// Crear una nova regla
pub async fn create_rule(
    req: HttpRequest,
    payload: web::Json<CreateRuleRequest>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    // Obtenir user_id del JWT token
    let auth_header = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Missing or invalid token"))?;
    
    let claims = crate::handlers::auth::verify_jwt(auth_header, &data.jwt_secret)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid user ID in token"))?;
    
    let pool = &data.db_pool;
    let conn = pool.get().await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Database connection failed"))?;
    
    // Verificar que el dispositiu pertany a l'usuari
    let device_id = payload.device_id;
    let device_exists = conn.interact(move |conn| {
        use crate::schema::devices;
        
        devices::table
            .filter(devices::id.eq(device_id))
            .filter(devices::user_id.eq(user_id))
            .count()
            .get_result::<i64>(conn)
    })
    .await
    .map_err(|_| actix_web::error::ErrorInternalServerError("Database query failed"))?
    .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to verify device"))?;
    
    if device_exists == 0 {
        return Err(actix_web::error::ErrorNotFound("Device not found"));
    }
    
    // Crear la regla
    let new_rule = NewRule {
        id: Uuid::new_v4(),
        user_id,
        device_id: payload.device_id,
        rule_type: payload.rule_type.clone(),
        params_json: payload.params.clone(),
        timezone: "Europe/Madrid".to_string(), // TODO: Obtenir de l'usuari
        priority: 1,
        enabled: payload.active.unwrap_or(true),
    };
    
    let rule = conn.interact(move |conn| {
        use crate::schema::rules;
        
        diesel::insert_into(rules::table)
            .values(&new_rule)
            .get_result::<Rule>(conn)
    })
    .await
    .map_err(|_| actix_web::error::ErrorInternalServerError("Database insert failed"))?
    .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to create rule"))?;
    
    log::info!("User {} created rule {} for device {}", user_id, rule.id, rule.device_id);
    
    Ok(HttpResponse::Created().json(RuleResponse {
        id: rule.id,
        device_id: rule.device_id,
        rule_type: rule.rule_type,
        params: rule.params_json,
        active: rule.enabled,
        created_at: rule.created_at,
        updated_at: rule.updated_at,
    }))
}

// Obtenir una regla específica
pub async fn get_rule(
    req: HttpRequest,
    path: web::Path<Uuid>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    // Obtenir user_id del JWT token
    let auth_header = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Missing or invalid token"))?;
    
    let claims = crate::handlers::auth::verify_jwt(auth_header, &data.jwt_secret)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid user ID in token"))?;
    
    let pool = &data.db_pool;
    let conn = pool.get().await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Database connection failed"))?;
    
    let rule_id = path.into_inner();
    
    let rule = conn.interact(move |conn| {
        use crate::schema::{rules, devices};
        
        rules::table
            .inner_join(devices::table.on(devices::id.eq(rules::device_id)))
            .filter(rules::id.eq(rule_id))
            .filter(devices::user_id.eq(user_id))
            .select(rules::all_columns)
            .first::<Rule>(conn)
    })
    .await
    .map_err(|_| actix_web::error::ErrorInternalServerError("Database query failed"))?
    .map_err(|_| actix_web::error::ErrorNotFound("Rule not found"))?;
    
    Ok(HttpResponse::Ok().json(RuleResponse {
        id: rule.id,
        device_id: rule.device_id,
        rule_type: rule.rule_type,
        params: rule.params_json,
        active: rule.enabled,
        created_at: rule.created_at,
        updated_at: rule.updated_at,
    }))
}

// Actualitzar una regla
pub async fn update_rule(
    req: HttpRequest,
    path: web::Path<Uuid>,
    payload: web::Json<UpdateRuleRequest>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    // Obtenir user_id del JWT token
    let auth_header = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Missing or invalid token"))?;
    
    let claims = crate::handlers::auth::verify_jwt(auth_header, &data.jwt_secret)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid user ID in token"))?;
    
    let pool = &data.db_pool;
    let conn = pool.get().await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Database connection failed"))?;
    
    let rule_id = path.into_inner();
    
    // Verificar que la regla pertany a un dispositiu de l'usuari
    let rule_exists = conn.interact(move |conn| {
        use crate::schema::{rules, devices};
        
        rules::table
            .inner_join(devices::table.on(devices::id.eq(rules::device_id)))
            .filter(rules::id.eq(rule_id))
            .filter(devices::user_id.eq(user_id))
            .count()
            .get_result::<i64>(conn)
    })
    .await
    .map_err(|_| actix_web::error::ErrorInternalServerError("Database query failed"))?
    .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to verify rule"))?;
    
    if rule_exists == 0 {
        return Err(actix_web::error::ErrorNotFound("Rule not found"));
    }
    
    // Actualitzar la regla
    let rule = conn.interact(move |conn| {
        use crate::schema::rules;
        
        // Construir els camps a actualitzar dinàmicament
        match (&payload.rule_type, &payload.params, payload.active) {
            (Some(rt), Some(p), Some(a)) => {
                diesel::update(rules::table.find(rule_id))
                    .set((
                        rules::rule_type.eq(rt),
                        rules::params_json.eq(p),
                        rules::enabled.eq(a),
                        rules::updated_at.eq(Utc::now()),
                    ))
                    .get_result::<Rule>(conn)
            },
            (Some(rt), Some(p), None) => {
                diesel::update(rules::table.find(rule_id))
                    .set((
                        rules::rule_type.eq(rt),
                        rules::params_json.eq(p),
                        rules::updated_at.eq(Utc::now()),
                    ))
                    .get_result::<Rule>(conn)
            },
            (Some(rt), None, Some(a)) => {
                diesel::update(rules::table.find(rule_id))
                    .set((
                        rules::rule_type.eq(rt),
                        rules::enabled.eq(a),
                        rules::updated_at.eq(Utc::now()),
                    ))
                    .get_result::<Rule>(conn)
            },
            (None, Some(p), Some(a)) => {
                diesel::update(rules::table.find(rule_id))
                    .set((
                        rules::params_json.eq(p),
                        rules::enabled.eq(a),
                        rules::updated_at.eq(Utc::now()),
                    ))
                    .get_result::<Rule>(conn)
            },
            (Some(rt), None, None) => {
                diesel::update(rules::table.find(rule_id))
                    .set((
                        rules::rule_type.eq(rt),
                        rules::updated_at.eq(Utc::now()),
                    ))
                    .get_result::<Rule>(conn)
            },
            (None, Some(p), None) => {
                diesel::update(rules::table.find(rule_id))
                    .set((
                        rules::params_json.eq(p),
                        rules::updated_at.eq(Utc::now()),
                    ))
                    .get_result::<Rule>(conn)
            },
            (None, None, Some(a)) => {
                diesel::update(rules::table.find(rule_id))
                    .set((
                        rules::enabled.eq(a),
                        rules::updated_at.eq(Utc::now()),
                    ))
                    .get_result::<Rule>(conn)
            },
            (None, None, None) => {
                // Si no hi ha res a actualitzar, només actualitzem updated_at
                diesel::update(rules::table.find(rule_id))
                    .set(rules::updated_at.eq(Utc::now()))
                    .get_result::<Rule>(conn)
            },
        }
    })
    .await
    .map_err(|_| actix_web::error::ErrorInternalServerError("Database update failed"))?
    .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to update rule"))?;
    
    log::info!("User {} updated rule {}", user_id, rule.id);
    
    Ok(HttpResponse::Ok().json(RuleResponse {
        id: rule.id,
        device_id: rule.device_id,
        rule_type: rule.rule_type,
        params: rule.params_json,
        active: rule.enabled,
        created_at: rule.created_at,
        updated_at: rule.updated_at,
    }))
}

// Eliminar una regla
pub async fn delete_rule(
    req: HttpRequest,
    path: web::Path<Uuid>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    // Obtenir user_id del JWT token
    let auth_header = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Missing or invalid token"))?;
    
    let claims = crate::handlers::auth::verify_jwt(auth_header, &data.jwt_secret)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid user ID in token"))?;
    
    let pool = &data.db_pool;
    let conn = pool.get().await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Database connection failed"))?;
    
    let rule_id = path.into_inner();
    
    // Verificar i eliminar la regla
    let deleted = conn.interact(move |conn| -> Result<bool, diesel::result::Error> {
        use crate::schema::{rules, devices};
        
        // Primer verificar que pertany a l'usuari
        let rule_exists = rules::table
            .inner_join(devices::table.on(devices::id.eq(rules::device_id)))
            .filter(rules::id.eq(rule_id))
            .filter(devices::user_id.eq(user_id))
            .count()
            .get_result::<i64>(conn)?;
        
        if rule_exists == 0 {
            return Ok(false);
        }
        
        // Eliminar la regla
        diesel::delete(rules::table.find(rule_id))
            .execute(conn)?;
        
        Ok(true)
    })
    .await
    .map_err(|_| actix_web::error::ErrorInternalServerError("Database operation failed"))?
    .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to delete rule"))?;
    
    if !deleted {
        return Err(actix_web::error::ErrorNotFound("Rule not found"));
    }
    
    log::info!("User {} deleted rule {}", user_id, rule_id);
    
    Ok(HttpResponse::Ok().json(json!({
        "message": "Rule deleted successfully"
    })))
}