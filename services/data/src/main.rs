use std::net::SocketAddr;
use std::sync::Arc;

use tonic::transport::Server;

use crate::services::authorization_proto::account_service_server;
use crate::services::config_proto::config_service_server;

mod services;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use a single database session for all services
    let session = Arc::new(
        scylla::SessionBuilder::new()
            .known_nodes(["scylla1", "scylla2", "scylla3"])
            .build()
            .await?,
    );

    Server::builder()
        .add_service(account_service_server::AccountServiceServer::new(
            services::ApplicationService::new(session.clone()).await?,
        ))
        .add_service(config_service_server::ConfigServiceServer::new(
            services::ApplicationService::new(session.clone()).await?,
        ))
        .serve("0.0.0.0:16000".parse::<SocketAddr>()?)
        .await?;

    Ok(())
}
