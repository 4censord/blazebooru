mod api;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Router;
use futures::Future;
use thiserror::Error;
use tracing::{error, info};

use blazebooru_core::BlazeBooruCore;

use crate::auth::{AuthError, BlazeBooruAuth};

pub struct BlazeBooruServer {
    pub auth: BlazeBooruAuth,
    pub core: BlazeBooruCore,
}

#[derive(Debug, Error)]
enum ApiError {
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
    #[error(transparent)]
    AuthError(#[from] AuthError),
    #[error("Bad request")]
    BadRequest,
    #[error("Not found")]
    NotFound,
    #[error("Unauthorized")]
    Unauthorized,
}

impl BlazeBooruServer {
    pub fn new(auth: BlazeBooruAuth, core: BlazeBooruCore) -> Result<Self, anyhow::Error> {
        Ok(Self { auth, core })
    }

    pub async fn run_server(self, shutdown: impl Future<Output = ()>) -> Result<(), anyhow::Error> {
        let server = Arc::new(self);

        let api = api::router(server.clone());

        let app = Router::new()
            .nest("/api", api)
            .merge(axum_extra::routing::SpaRouter::new("/f", &server.core.public_path))
            .layer(tower_http::trace::TraceLayer::new_for_http());

        let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

        info!("Web server listening on: {addr}");
        axum::Server::bind(&addr)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .with_graceful_shutdown(shutdown)
            .await?;

        Ok(())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Anyhow(err) => {
                error!("{err:#}");
                (StatusCode::INTERNAL_SERVER_ERROR, format!("{err:#}")).into_response()
            }
            Self::AuthError(AuthError::ExpiredToken) => (StatusCode::UNAUTHORIZED, ()).into_response(),
            Self::AuthError(err) => (StatusCode::BAD_REQUEST, format!("{err:#}")).into_response(),
            Self::BadRequest => (StatusCode::BAD_REQUEST, ()).into_response(),
            Self::NotFound => (StatusCode::NOT_FOUND, ()).into_response(),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, ()).into_response(),
        }
    }
}
