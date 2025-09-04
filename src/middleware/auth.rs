use crate::{handlers::auth::verify_jwt, AppState};
use actix_web::{
    dev::Payload,
    error::ErrorUnauthorized,
    web, Error, FromRequest, HttpMessage, HttpRequest,
};
use futures_util::future::{ready, Ready};
use uuid::Uuid;

pub struct RequireAuth;

impl FromRequest for RequireAuth {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        // TODO: Implementar autenticació completa
        // Per ara només comprovem que hi ha un user_id a les extensions
        if req.extensions().get::<Uuid>().is_some() {
            ready(Ok(RequireAuth))
        } else {
            ready(Err(ErrorUnauthorized("Not authenticated")))
        }
    }
}

// Extractor per obtenir el user_id de les extensions
pub struct AuthUser(pub Uuid);

impl FromRequest for AuthUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        if let Some(user_id) = req.extensions().get::<Uuid>() {
            ready(Ok(AuthUser(*user_id)))
        } else {
            ready(Err(ErrorUnauthorized("Not authenticated")))
        }
    }
}