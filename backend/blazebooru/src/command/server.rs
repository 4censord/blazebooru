use std::env;

use anyhow::Context;

use blazebooru_core::BlazeBooruCore;

use crate::{auth::BlazeBooruAuth, server::BlazeBooruServer};

pub async fn server(core: BlazeBooruCore) -> Result<(), anyhow::Error> {
    let jwt_secret = env::var("BLAZEBOORU_JWT_SECRET").context("BLAZEBOORU_JWT_SECRET is not set")?;

    let auth = BlazeBooruAuth::new(jwt_secret.as_bytes());
    let server = BlazeBooruServer::new(auth, core).context("Error creating server")?;

    let shutdown = || async {
        tokio::signal::ctrl_c().await.expect("Error awaiting Ctrl-C signal");
    };

    server.run_server(shutdown()).await?;

    Ok(())
}
