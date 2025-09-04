use crate::{models::user::*, schema::users, AppState, DbPool};
use actix_session::Session;
use actix_web::{web, HttpResponse};
use chrono::Utc;
use diesel::prelude::*;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
// OAuth2 imports commented out temporarily until we implement it properly for v5
// use oauth2::{
//     basic::BasicClient, AuthUrl, AuthorizationCode, ClientId,
//     ClientSecret, CsrfToken, RedirectUrl, Scope, TokenUrl,
// };
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub email: String,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, Deserialize)]
pub struct GoogleUserInfo {
    pub sub: String,
    pub email: String,
    pub name: String,
    pub picture: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GoogleCallbackQuery {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

pub async fn google_login(
    session: Session,
    _data: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    // TODO: Implementar OAuth amb oauth2 v5
    // Per ara retornem una URL hardcoded per development
    let auth_url = "https://accounts.google.com/o/oauth2/v2/auth?client_id=YOUR_CLIENT_ID&redirect_uri=http://localhost:8080/api/auth/google/callback&response_type=code&scope=email+profile+openid&state=random_state";
    
    session.insert("csrf_token", "random_state")?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "auth_url": auth_url
    })))
}

pub async fn google_callback(
    _query: web::Query<GoogleCallbackQuery>,
    session: Session,
    data: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    // TODO: Implementar OAuth callback amb oauth2 v5
    // Per ara simulem un usuari de prova
    
    // Crear usuari de prova
    let user_info = GoogleUserInfo {
        sub: "test-google-id-123".to_string(),
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
        picture: None,
    };

    // Buscar o crear usuari
    let pool = &data.db_pool;
    let user = upsert_user(pool, user_info).await?;

    // Crear JWT
    let jwt_token = create_jwt(&user, &data.jwt_secret)?;

    // Guardar user_id a la sessió
    session.insert("user_id", user.id.to_string())?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        token: jwt_token,
        user,
    }))
}

pub async fn logout(session: Session) -> Result<HttpResponse, actix_web::Error> {
    session.clear();
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Logged out successfully"
    })))
}

pub async fn get_current_user(
    session: Session,
    data: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = session
        .get::<String>("user_id")?
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Not authenticated"))?;

    let user_uuid = Uuid::parse_str(&user_id)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Invalid user ID"))?;

    let pool = &data.db_pool;
    let conn = pool.get().await
        .map_err(|e| {
            log::error!("Failed to get DB connection: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database connection error")
        })?;

    let user = conn
        .interact(move |conn| {
            users::table
                .find(user_uuid)
                .first::<User>(conn)
        })
        .await
        .map_err(|e| {
            log::error!("Database interaction error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?
        .map_err(|_| actix_web::error::ErrorNotFound("User not found"))?;

    Ok(HttpResponse::Ok().json(user))
}

// Funcions auxiliars
// TODO: Actualitzar per oauth2 v5
/*
fn create_oauth_client(_data: &web::Data<AppState>) -> BasicClient {
    todo!("Implementar OAuth client per oauth2 v5")
}
*/

async fn get_google_user_info(access_token: &str) -> Result<GoogleUserInfo, actix_web::Error> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://www.googleapis.com/oauth2/v3/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| {
            log::error!("Failed to get user info: {:?}", e);
            actix_web::error::ErrorInternalServerError("Failed to get user info from Google")
        })?;

    let user_info = response.json::<GoogleUserInfo>().await.map_err(|e| {
        log::error!("Failed to parse user info: {:?}", e);
        actix_web::error::ErrorInternalServerError("Failed to parse user info")
    })?;

    Ok(user_info)
}

async fn upsert_user(pool: &DbPool, user_info: GoogleUserInfo) -> Result<User, actix_web::Error> {
    let conn = pool.get().await.map_err(|e| {
        log::error!("Failed to get DB connection: {:?}", e);
        actix_web::error::ErrorInternalServerError("Database connection error")
    })?;

    let user = conn
        .interact(move |conn| {
            // Intentar trobar usuari existent
            match users::table
                .filter(users::google_sub.eq(&user_info.sub))
                .first::<User>(conn)
            {
                Ok(existing_user) => {
                    // Actualitzar informació de l'usuari
                    diesel::update(users::table.find(existing_user.id))
                        .set((
                            users::email.eq(&user_info.email),
                            users::name.eq(&user_info.name),
                            users::picture.eq(&user_info.picture),
                            users::updated_at.eq(Utc::now()),
                        ))
                        .get_result::<User>(conn)
                }
                Err(_) => {
                    // Crear nou usuari
                    let new_user = NewUser {
                        id: Uuid::new_v4(),
                        google_sub: user_info.sub,
                        email: user_info.email,
                        name: user_info.name,
                        picture: user_info.picture,
                    };

                    diesel::insert_into(users::table)
                        .values(&new_user)
                        .get_result::<User>(conn)
                }
            }
        })
        .await
        .map_err(|e| {
            log::error!("Database interaction error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?
        .map_err(|e| {
            log::error!("Failed to upsert user: {:?}", e);
            actix_web::error::ErrorInternalServerError("Failed to save user")
        })?;

    Ok(user)
}

fn create_jwt(user: &User, secret: &str) -> Result<String, actix_web::Error> {
    let now = Utc::now().timestamp();
    let claims = Claims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        exp: now + 86400 * 7, // 7 dies
        iat: now,
    };

    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| {
        log::error!("Failed to create JWT: {:?}", e);
        actix_web::error::ErrorInternalServerError("Failed to create authentication token")
    })
}

pub fn verify_jwt(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map(|data| data.claims)
}
