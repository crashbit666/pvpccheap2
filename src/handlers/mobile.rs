use crate::{
    models::{command::*, device::*, user::*},
    schema::{commands, device_states, devices, mobile_sessions, structures},
    AppState, DbPool,
};
use actix_web::{web, HttpResponse};
use chrono::Utc;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct HeartbeatRequest {
    pub device_token: String,
    pub platform: String,
    pub app_version: String,
}

#[derive(Debug, Serialize)]
pub struct HeartbeatResponse {
    pub pending_commands: Vec<Command>,
    pub server_time: chrono::DateTime<Utc>,
}

// Handler per sincronitzar dispositius des de l'app
pub async fn sync_devices(
    web::Json(sync_req): web::Json<DeviceSyncRequest>,
    // TODO: Add authentication
    data: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let pool = &data.db_pool;
    // TODO: Get user_id from authentication
    let user_id = Uuid::new_v4(); // Temporary for testing
    
    // Processar structures
    for structure in sync_req.structures {
        upsert_structure(pool, user_id, structure).await?;
    }
    
    // Processar dispositius
    let mut synced_devices = Vec::new();
    for device in sync_req.devices {
        let synced = upsert_device(pool, user_id, device).await?;
        synced_devices.push(synced);
    }
    
    log::info!("User {} synced {} devices", user_id, synced_devices.len());
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Devices synced successfully",
        "devices_count": synced_devices.len(),
        "timestamp": Utc::now()
    })))
}

// Handler per al heartbeat de l'app
pub async fn heartbeat(
    web::Json(req): web::Json<HeartbeatRequest>,
    // TODO: Add authentication
    data: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let pool = &data.db_pool;
    // TODO: Get user_id from authentication
    let user_id = Uuid::new_v4(); // Temporary for testing
    
    // Actualitzar o crear sessió mòbil
    update_mobile_session(pool, user_id, &req).await?;
    
    // Obtenir comandes pendents
    let pending_commands = get_pending_commands(pool, user_id).await?;
    
    Ok(HttpResponse::Ok().json(HeartbeatResponse {
        pending_commands,
        server_time: Utc::now(),
    }))
}

// Handler per reportar resultats de comandes
pub async fn command_result(
    web::Json(result): web::Json<CommandResult>,
    // TODO: Add authentication
    data: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let pool = &data.db_pool;
    let conn = pool.get().await.map_err(|e| {
        log::error!("Failed to get DB connection: {:?}", e);
        actix_web::error::ErrorInternalServerError("Database connection error")
    })?;
    
    let command_id = result.command_id;
    let status = if result.success { "acked" } else { "failed" };
    
    // Actualitzar estat de la comanda
    conn.interact(move |conn| {
        diesel::update(commands::table.find(command_id))
            .set((
                commands::status.eq(status),
                commands::error_message.eq(&result.error_message),
                commands::executed_at.eq(Some(Utc::now())),
                commands::updated_at.eq(Utc::now()),
            ))
            .execute(conn)
    })
    .await
    .map_err(|e| {
        log::error!("Failed to update command: {:?}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?
    .map_err(|e| {
        log::error!("Failed to update command: {:?}", e);
        actix_web::error::ErrorInternalServerError("Failed to update command")
    })?;
    
    // Si hi ha nou estat, actualitzar-lo
    if let Some(new_state) = result.new_state {
        update_device_state(pool, result.command_id, new_state).await?;
    }
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Command result recorded",
        "command_id": result.command_id
    })))
}

// Funcions auxiliars
async fn upsert_structure(
    pool: &DbPool,
    user_id: Uuid,
    structure: StructureSync,
) -> Result<Structure, actix_web::Error> {
    let conn = pool.get().await.map_err(|e| {
        log::error!("Failed to get DB connection: {:?}", e);
        actix_web::error::ErrorInternalServerError("Database connection error")
    })?;
    
    let user_id_copy = user_id;
    let result = conn
        .interact(move |conn| {
            use crate::schema::structures::dsl::*;
            
            // Intentar trobar structure existent
            match structures
                .filter(google_structure_id.eq(&structure.google_structure_id))
                .filter(crate::schema::structures::user_id.eq(&user_id_copy))
                .first::<Structure>(conn)
            {
                Ok(existing) => {
                    // Actualitzar
                    diesel::update(structures.find(existing.id))
                        .set((
                            name.eq(&structure.name),
                            updated_at.eq(Utc::now()),
                        ))
                        .get_result::<Structure>(conn)
                }
                Err(_) => {
                    // Crear nova
                    let new_structure = NewStructure {
                        id: Uuid::new_v4(),
                        user_id: user_id_copy,
                        google_structure_id: structure.google_structure_id,
                        name: structure.name,
                    };
                    
                    diesel::insert_into(structures)
                        .values(&new_structure)
                        .get_result::<Structure>(conn)
                }
            }
        })
        .await
        .map_err(|e| {
            log::error!("Database interaction error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?
        .map_err(|e| {
            log::error!("Failed to upsert structure: {:?}", e);
            actix_web::error::ErrorInternalServerError("Failed to save structure")
        })?;
    
    Ok(result)
}

async fn upsert_device(
    pool: &DbPool,
    user_id: Uuid,
    device: DeviceSync,
) -> Result<Device, actix_web::Error> {
    let conn = pool.get().await.map_err(|e| {
        log::error!("Failed to get DB connection: {:?}", e);
        actix_web::error::ErrorInternalServerError("Database connection error")
    })?;
    
    let user_id_copy = user_id;
    let result = conn
        .interact(move |conn| {
            
            // Obtenir structure_id si es proporciona
            let struct_id = if let Some(google_struct_id) = &device.structure_id {
                use crate::schema::structures;
                structures::table
                    .filter(structures::google_structure_id.eq(google_struct_id))
                    .filter(structures::user_id.eq(&user_id_copy))
                    .select(structures::id)
                    .first::<Uuid>(conn)
                    .ok()
            } else {
                None
            };
            
            // Intentar trobar dispositiu existent
            use crate::schema::devices;
            match devices::table
                .filter(devices::google_device_id.eq(&device.google_device_id))
                .filter(devices::user_id.eq(&user_id_copy))
                .first::<Device>(conn)
            {
                Ok(existing) => {
                    // Actualitzar
                    diesel::update(devices::table.find(existing.id))
                        .set((
                            devices::name.eq(&device.name),
                            devices::device_type.eq(&device.device_type),
                            devices::room.eq(&device.room),
                            devices::structure_id.eq(&struct_id),
                            devices::capabilities_json.eq(&device.capabilities),
                            devices::last_seen_at.eq(Utc::now()),
                            devices::updated_at.eq(Utc::now()),
                        ))
                        .get_result::<Device>(conn)?;
                    
                    // Actualitzar estat
                    update_device_state_sync(conn, existing.id, device.state)?;
                    
                    Ok(existing)
                }
                Err(_) => {
                    // Crear nou
                    let new_device = NewDevice {
                        id: Uuid::new_v4(),
                        user_id: user_id_copy,
                        structure_id: struct_id,
                        google_device_id: device.google_device_id,
                        name: device.name,
                        device_type: device.device_type,
                        room: device.room,
                        capabilities_json: device.capabilities,
                        last_seen_at: Utc::now(),
                    };
                    
                    let created = diesel::insert_into(devices::table)
                        .values(&new_device)
                        .get_result::<Device>(conn)?;
                    
                    // Crear estat inicial
                    let new_state = NewDeviceState {
                        id: Uuid::new_v4(),
                        device_id: created.id,
                        state_json: device.state,
                        updated_at: Utc::now(),
                    };
                    
                    use crate::schema::device_states;
                    diesel::insert_into(device_states::table)
                        .values(&new_state)
                        .execute(conn)?;
                    
                    Ok(created)
                }
            }
        })
        .await
        .map_err(|e| {
            log::error!("Database interaction error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?
        .map_err(|e: diesel::result::Error| {
            log::error!("Failed to upsert device: {:?}", e);
            actix_web::error::ErrorInternalServerError("Failed to save device")
        })?;
    
    Ok(result)
}

fn update_device_state_sync(
    conn: &mut PgConnection,
    device_id_param: Uuid,
    new_state: serde_json::Value,
) -> Result<(), diesel::result::Error> {
    use crate::schema::device_states::dsl::*;
    
    diesel::update(device_states.filter(device_id.eq(&device_id_param)))
        .set((
            state_json.eq(&new_state),
            updated_at.eq(Utc::now()),
        ))
        .execute(conn)?;
    
    Ok(())
}

async fn update_device_state(
    pool: &DbPool,
    command_id: Uuid,
    new_state: serde_json::Value,
) -> Result<(), actix_web::Error> {
    let conn = pool.get().await.map_err(|e| {
        log::error!("Failed to get DB connection: {:?}", e);
        actix_web::error::ErrorInternalServerError("Database connection error")
    })?;
    
    conn.interact(move |conn| {
        // Obtenir device_id de la comanda
        let device_id = commands::table
            .find(command_id)
            .select(commands::device_id)
            .first::<Uuid>(conn)?;
        
        update_device_state_sync(conn, device_id, new_state)
    })
    .await
    .map_err(|e| {
        log::error!("Database interaction error: {:?}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?
    .map_err(|e| {
        log::error!("Failed to update device state: {:?}", e);
        actix_web::error::ErrorInternalServerError("Failed to update device state")
    })?;
    
    Ok(())
}

async fn update_mobile_session(
    pool: &DbPool,
    user_id: Uuid,
    req: &HeartbeatRequest,
) -> Result<(), actix_web::Error> {
    let conn = pool.get().await.map_err(|e| {
        log::error!("Failed to get DB connection: {:?}", e);
        actix_web::error::ErrorInternalServerError("Database connection error")
    })?;
    
    let device_token_clone = req.device_token.clone();
    let platform_clone = req.platform.clone();
    let app_version_clone = req.app_version.clone();
    
    conn.interact(move |conn| {
        use crate::schema::mobile_sessions;
        
        match mobile_sessions::table
            .filter(mobile_sessions::user_id.eq(&user_id))
            .filter(mobile_sessions::device_token.eq(&device_token_clone))
            .first::<MobileSession>(conn)
        {
            Ok(existing) => {
                diesel::update(mobile_sessions::table.find(existing.id))
                    .set((
                        mobile_sessions::platform.eq(&platform_clone),
                        mobile_sessions::app_version.eq(&app_version_clone),
                        mobile_sessions::last_heartbeat.eq(Utc::now()),
                        mobile_sessions::updated_at.eq(Utc::now()),
                    ))
                    .execute(conn)
            }
            Err(_) => {
                let new_session = NewMobileSession {
                    id: Uuid::new_v4(),
                    user_id,
                    device_token: device_token_clone,
                    platform: platform_clone,
                    app_version: app_version_clone,
                };
                
                diesel::insert_into(mobile_sessions::table)
                    .values(&new_session)
                    .execute(conn)
            }
        }
    })
    .await
    .map_err(|e| {
        log::error!("Database interaction error: {:?}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?
    .map_err(|e| {
        log::error!("Failed to update mobile session: {:?}", e);
        actix_web::error::ErrorInternalServerError("Failed to update mobile session")
    })?;
    
    Ok(())
}

async fn get_pending_commands(pool: &DbPool, user_id: Uuid) -> Result<Vec<Command>, actix_web::Error> {
    let conn = pool.get().await.map_err(|e| {
        log::error!("Failed to get DB connection: {:?}", e);
        actix_web::error::ErrorInternalServerError("Database connection error")
    })?;
    
    let commands = conn
        .interact(move |conn| {
            commands::table
                .filter(commands::user_id.eq(user_id))
                .filter(commands::status.eq("queued"))
                .order(commands::created_at.asc())
                .limit(10)
                .load::<Command>(conn)
        })
        .await
        .map_err(|e| {
            log::error!("Database interaction error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?
        .map_err(|e| {
            log::error!("Failed to get pending commands: {:?}", e);
            actix_web::error::ErrorInternalServerError("Failed to get pending commands")
        })?;
    
    Ok(commands)
}
