use crate::{
    models::{command::*, device::*},
    schema::{commands, device_states, devices},
    AppState, DbPool,
};
use actix_web::{web, HttpResponse};
use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;

// Llistar tots els dispositius de l'usuari
pub async fn list_devices(
    // TODO: Add authentication
    data: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let pool = &data.db_pool;
    // TODO: Get user_id from authentication
    let user_id = Uuid::new_v4(); // Temporary for testing
    
    let conn = pool.get().await.map_err(|e| {
        log::error!("Failed to get DB connection: {:?}", e);
        actix_web::error::ErrorInternalServerError("Database connection error")
    })?;
    
    let devices = conn
        .interact(move |conn| {
            devices::table
                .filter(devices::user_id.eq(user_id))
                .order(devices::name.asc())
                .load::<Device>(conn)
        })
        .await
        .map_err(|e| {
            log::error!("Database interaction error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?
        .map_err(|e| {
            log::error!("Failed to list devices: {:?}", e);
            actix_web::error::ErrorInternalServerError("Failed to list devices")
        })?;
    
    Ok(HttpResponse::Ok().json(devices))
}

// Obtenir informació d'un dispositiu específic
pub async fn get_device(
    device_id: web::Path<Uuid>,
    // TODO: Add authentication
    data: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let pool = &data.db_pool;
    // TODO: Get user_id from authentication
    let user_id = Uuid::new_v4(); // Temporary for testing
    let device_id = device_id.into_inner();
    
    let conn = pool.get().await.map_err(|e| {
        log::error!("Failed to get DB connection: {:?}", e);
        actix_web::error::ErrorInternalServerError("Database connection error")
    })?;
    
    let device = conn
        .interact(move |conn| {
            devices::table
                .find(device_id)
                .filter(devices::user_id.eq(user_id))
                .first::<Device>(conn)
        })
        .await
        .map_err(|e| {
            log::error!("Database interaction error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?
        .map_err(|_| actix_web::error::ErrorNotFound("Device not found"))?;
    
    Ok(HttpResponse::Ok().json(device))
}

// Obtenir l'estat actual d'un dispositiu
pub async fn get_device_state(
    device_id: web::Path<Uuid>,
    // TODO: Add authentication
    data: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let pool = &data.db_pool;
    // TODO: Get user_id from authentication
    let user_id = Uuid::new_v4(); // Temporary for testing
    let device_id = device_id.into_inner();
    
    let conn = pool.get().await.map_err(|e| {
        log::error!("Failed to get DB connection: {:?}", e);
        actix_web::error::ErrorInternalServerError("Database connection error")
    })?;
    
    // Verificar que el dispositiu pertany a l'usuari i obtenir l'estat
    let result = conn
        .interact(move |conn| {
            // Primer verificar propietat
            devices::table
                .find(device_id)
                .filter(devices::user_id.eq(user_id))
                .select(devices::id)
                .first::<Uuid>(conn)?;
            
            // Després obtenir l'estat
            device_states::table
                .filter(device_states::device_id.eq(device_id))
                .first::<DeviceState>(conn)
        })
        .await
        .map_err(|e| {
            log::error!("Database interaction error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?
        .map_err(|_| actix_web::error::ErrorNotFound("Device state not found"))?;
    
    Ok(HttpResponse::Ok().json(result))
}

// Enviar una comanda a un dispositiu
pub async fn send_command(
    device_id: web::Path<Uuid>,
    web::Json(req): web::Json<CreateCommandRequest>,
    // TODO: Add authentication
    data: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let pool = &data.db_pool;
    // TODO: Get user_id from authentication
    let user_id = Uuid::new_v4(); // Temporary for testing
    let device_id = device_id.into_inner();
    
    // Verificar que el dispositiu pertany a l'usuari
    verify_device_ownership(pool, device_id, user_id).await?;
    
    // Crear la comanda
    let conn = pool.get().await.map_err(|e| {
        log::error!("Failed to get DB connection: {:?}", e);
        actix_web::error::ErrorInternalServerError("Database connection error")
    })?;
    
    let new_command = NewCommand {
        id: Uuid::new_v4(),
        user_id,
        device_id,
        command_type: req.command_type.clone(),
        payload_json: req.payload.clone(),
        status: CommandStatus::Queued.to_string(),
        retry_count: 0,
    };
    
    let command = conn
        .interact(move |conn| {
            diesel::insert_into(commands::table)
                .values(&new_command)
                .get_result::<Command>(conn)
        })
        .await
        .map_err(|e| {
            log::error!("Database interaction error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?
        .map_err(|e| {
            log::error!("Failed to create command: {:?}", e);
            actix_web::error::ErrorInternalServerError("Failed to create command")
        })?;
    
    // TODO: Enviar notificació FCM a l'app mòbil
    
    log::info!("Command {} created for device {}", command.id, device_id);
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "command_id": command.id,
        "status": command.status,
        "message": "Command queued successfully"
    })))
}

// Funció auxiliar per verificar propietat d'un dispositiu
async fn verify_device_ownership(
    pool: &DbPool,
    device_id: Uuid,
    user_id: Uuid,
) -> Result<(), actix_web::Error> {
    let conn = pool.get().await.map_err(|e| {
        log::error!("Failed to get DB connection: {:?}", e);
        actix_web::error::ErrorInternalServerError("Database connection error")
    })?;
    
    conn.interact(move |conn| {
        devices::table
            .find(device_id)
            .filter(devices::user_id.eq(user_id))
            .select(devices::id)
            .first::<Uuid>(conn)
    })
    .await
    .map_err(|e| {
        log::error!("Database interaction error: {:?}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?
    .map_err(|_| actix_web::error::ErrorForbidden("Device not found or access denied"))?;
    
    Ok(())
}
