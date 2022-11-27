mod auth;
mod post;
mod tag;
mod user;

use std::sync::Arc;

use axum::{
    async_trait,
    extract::FromRequestParts,
    headers::{authorization::Bearer, Authorization},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Router, TypedHeader,
};

use crate::{
    auth::{AuthClaims, AuthError, SessionClaims},
    server::BlazeBooruServer,
};

#[derive(Debug)]
struct Authorized {
    session: i64,
    claims: AuthClaims,
}

pub fn router() -> Router<Arc<BlazeBooruServer>> {
    let auth = auth::router();
    let post = post::router();
    let user = user::router();
    let tag = tag::router();

    Router::new()
        .nest("/auth", auth)
        .nest("/post", post)
        .nest("/user", user)
        .nest("/tag", tag)
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let status = match self {
            AuthError::ExpiredToken => StatusCode::UNAUTHORIZED,
            AuthError::TokenCreation => StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::InvalidToken => StatusCode::BAD_REQUEST,
        };

        (status, self.to_string()).into_response()
    }
}

#[async_trait]
impl FromRequestParts<Arc<BlazeBooruServer>> for Authorized {
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &Arc<BlazeBooruServer>) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
            .await
            .map_err(|_| AuthError::InvalidToken)?;

        let token = bearer.token();

        let SessionClaims { session, claims } = state.auth.verify::<SessionClaims>(token)?;

        Ok(Authorized { session, claims })
    }
}
