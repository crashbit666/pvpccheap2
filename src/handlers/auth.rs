use crate::{
    models::user::User,
    AppState,
};
use actix_session::Session;
use actix_web::{web, HttpResponse};
use chrono::{Duration, Utc};
use diesel::prelude::*;
use jsonwebtoken::{encode, decode, DecodingKey, EncodingKey, Header, Validation};
// OAuth2 imports (actualment no utilitzades per incompatibilitats de versió)
// Utilitzem l'API REST directament
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Estructures per OAuth
#[derive(Debug, Deserialize)]
pub struct GoogleCallbackQuery {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GoogleUserInfo {
    pub sub: String,
    pub email: String,
    pub name: String,
    pub picture: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub email: String,
    pub exp: i64,
    pub iat: i64,
}

// Handler per iniciar el flux OAuth
pub async fn google_login(
    session: Session,
    data: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    // Per ara utilitzem una implementació simplificada fins que resolguem OAuth2 v5
    // TODO: Implementar OAuth2 v5 correctament
    
    let client_id = &data.google_client_id;
    let redirect_uri = std::env::var("GOOGLE_REDIRECT_URL")
        .unwrap_or_else(|_| "http://localhost:8080/api/auth/google/callback".to_string());
    
    // Generar CSRF token
    let csrf_token = uuid::Uuid::new_v4().to_string();
    session.insert("csrf_token", &csrf_token)?;
    
    // Construir URL d'autorització de Google manualment
    let auth_url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?\
        client_id={}&\
        redirect_uri={}&\
        response_type=code&\
        scope=email%20profile%20openid&\
        state={}&\
        access_type=offline&\
        prompt=consent",
        urlencoding::encode(client_id),
        urlencoding::encode(&redirect_uri),
        urlencoding::encode(&csrf_token)
    );
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "auth_url": auth_url
    })))
}

// Handler per processar el callback d'OAuth
pub async fn google_callback(
    query: web::Query<GoogleCallbackQuery>,
    session: Session,
    data: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    // Verificar CSRF token
    let stored_csrf = session.get::<String>("csrf_token")?;
    if stored_csrf != Some(query.state.clone()) {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid CSRF token"
        })));
    }

    // Intercanviar codi per token usant l'API de Google directament
    let client = reqwest::Client::new();
    
    let token_response = client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", &query.code),
            ("client_id", &data.google_client_id),
            ("client_secret", &data.google_client_secret),
            ("redirect_uri", &std::env::var("GOOGLE_REDIRECT_URL")
                .unwrap_or_else(|_| "http://localhost:8080/api/auth/google/callback".to_string())),
        ])
        .send()
        .await
        .map_err(|e| {
            log::error!("Failed to exchange code: {:?}", e);
            actix_web::error::ErrorInternalServerError("Failed to exchange authorization code")
        })?;
    
    if !token_response.status().is_success() {
        let error_text = token_response.text().await.unwrap_or_default();
        log::error!("Token exchange failed: {}", error_text);
        return Err(actix_web::error::ErrorInternalServerError("Failed to exchange authorization code"));
    }
    
    #[derive(Deserialize)]
    struct TokenResponse {
        access_token: String,
        #[allow(dead_code)]
        token_type: String,
        #[allow(dead_code)]
        expires_in: Option<i64>,
        #[allow(dead_code)]
        refresh_token: Option<String>,
    }
    
    let token_data: TokenResponse = token_response.json().await
        .map_err(|e| {
            log::error!("Failed to parse token response: {:?}", e);
            actix_web::error::ErrorInternalServerError("Failed to parse token response")
        })?;

    // Obtenir informació de l'usuari de Google
    let user_info = get_google_user_info(&token_data.access_token).await?;

    // Buscar o crear usuari
    let pool = &data.db_pool;
    let user = upsert_user(pool, user_info).await?;

    // Crear JWT
    let jwt_token = create_jwt(&user, &data.jwt_secret)?;

    // Guardar user_id a la sessió
    session.insert("user_id", user.id.to_string())?;

    // Retornar HTML amb el token per tancar la finestra popup
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(format!(r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Login exitós</title>
            <style>
                body {{ 
                    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                    text-align: center;
                    padding: 50px;
                    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                    color: white;
                }}
                .success {{ 
                    font-size: 48px;
                    margin-bottom: 20px;
                }}
                .message {{
                    font-size: 24px;
                    margin-bottom: 10px;
                }}
                .info {{
                    background: rgba(255,255,255,0.1);
                    padding: 20px;
                    border-radius: 10px;
                    margin-top: 20px;
                    backdrop-filter: blur(10px);
                }}
                .token {{
                    font-family: monospace;
                    background: rgba(0,0,0,0.2);
                    padding: 10px;
                    border-radius: 5px;
                    word-break: break-all;
                    margin-top: 10px;
                }}
            </style>
        </head>
        <body>
            <div class="success">✅</div>
            <div class="message">Login exitós!</div>
            <p>Benvingut, <strong>{}</strong>!</p>
            <div class="info">
                <p>El teu token JWT s'ha generat correctament.</p>
                <div class="token">{}</div>
                <p style="margin-top: 20px; opacity: 0.8;">Aquesta finestra es tancarà automàticament en 3 segons...</p>
            </div>
            <script>
                // Guardar token al localStorage
                localStorage.setItem('jwt_token', '{}');
                localStorage.setItem('user_name', '{}');
                localStorage.setItem('user_email', '{}');
                
                // Enviar missatge a la finestra pare si existeix
                if (window.opener) {{
                    window.opener.postMessage({{
                        type: 'auth_success',
                        token: '{}',
                        user: {{
                            name: '{}',
                            email: '{}'
                        }}
                    }}, '*');
                }}
                
                // Tancar la finestra després de 3 segons
                setTimeout(() => {{
                    window.close();
                    // Si no es pot tancar, redirigir a la pàgina principal
                    window.location.href = '/';
                }}, 3000);
            </script>
        </body>
        </html>
    "#, 
        user.name, 
        format!("{}...", &jwt_token[..20.min(jwt_token.len())]),
        jwt_token,
        user.name,
        user.email,
        jwt_token,
        user.name,
        user.email
    )))
}

// Handler per tancar sessió
pub async fn logout(session: Session) -> Result<HttpResponse, actix_web::Error> {
    session.clear();
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Logged out successfully"
    })))
}

// Handler per obtenir l'usuari actual
pub async fn get_current_user(
    session: Session,
    data: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = session
        .get::<String>("user_id")?
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Not logged in"))?;

    let uuid = Uuid::parse_str(&user_id)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid user ID"))?;

    let pool = &data.db_pool;
    let conn = pool.get().await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Database connection failed"))?;

    use crate::schema::users::dsl;
    
    // Utilitzar interact per queries síncrones
    let user = conn
        .interact(move |conn| {
            dsl::users
                .find(uuid)
                .first::<User>(conn)
        })
        .await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Database interaction failed"))?
        .map_err(|_| actix_web::error::ErrorNotFound("User not found"))?;

    Ok(HttpResponse::Ok().json(user))
}

// Obtenir informació de l'usuari de Google
pub async fn get_google_user_info(access_token: &str) -> Result<GoogleUserInfo, actix_web::Error> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://www.googleapis.com/oauth2/v3/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| {
            log::error!("Failed to get user info: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to get user info")
        })?;

    if !response.status().is_success() {
        log::error!("Google API returned error: {}", response.status());
        return Err(actix_web::error::ErrorInternalServerError("Failed to get user info"));
    }

    response
        .json::<GoogleUserInfo>()
        .await
        .map_err(|e| {
            log::error!("Failed to parse user info: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to parse user info")
        })
}

// Crear o actualitzar usuari
async fn upsert_user(
    pool: &deadpool_diesel::postgres::Pool,
    user_info: GoogleUserInfo,
) -> Result<User, actix_web::Error> {
    let conn = pool.get().await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Database connection failed"))?;

    use crate::schema::users::dsl;
    
    // Buscar usuari existent
    let google_sub = user_info.sub.clone();
    let existing_user = conn
        .interact(move |conn| {
            dsl::users
                .filter(dsl::google_sub.eq(&google_sub))
                .first::<User>(conn)
                .optional()
        })
        .await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Database interaction failed"))?
        .map_err(|_| actix_web::error::ErrorInternalServerError("Database query failed"))?;

    match existing_user {
        Some(user) => Ok(user),
        None => {
            // Crear nou usuari utilitzant el mètode User::new
            let new_user = User::new(
                user_info.sub,
                user_info.email,
                user_info.name,
                user_info.picture,
            );

            conn.interact(move |conn| {
                diesel::insert_into(dsl::users)
                    .values(&new_user)
                    .get_result::<User>(conn)
            })
            .await
            .map_err(|_| actix_web::error::ErrorInternalServerError("Database insert failed"))?
            .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to create user"))
        }
    }
}

// Crear JWT token
pub fn create_jwt(user: &User, secret: &str) -> Result<String, actix_web::Error> {
    let expiration = Utc::now() + Duration::hours(24);
    
    let claims = Claims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        exp: expiration.timestamp(),
        iat: Utc::now().timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|e| {
        log::error!("Failed to create JWT: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to create token")
    })
}

// Verificar JWT token
#[allow(dead_code)] // S'utilitzarà en el middleware d'autenticació
pub fn verify_jwt(token: &str, secret: &str) -> Result<Claims, actix_web::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .map(|token_data| token_data.claims)
    .map_err(|e| {
        log::error!("Failed to verify JWT: {}", e);
        actix_web::error::ErrorUnauthorized("Invalid token")
    })
}