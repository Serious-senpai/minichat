use std::net::SocketAddr;
use std::sync::Arc;

use clap::Parser;
use tonic::transport::Server;

use crate::services::p_authorization::account_service_server;
use crate::services::p_channels::channel_service_server;
use crate::services::p_config::config_service_server;

mod services;

#[derive(Parser)]
#[command(name = "data-service")]
#[command(long_about = None)]
struct _Arguments {
    /// A comma-separated list of ScyllaDB clustered hosts to connect to
    #[arg(long, short)]
    scylla_hosts: String,

    /// A comma-separated list of AMQP clustered hosts to connect to
    #[arg(long, short)]
    amqp_hosts: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = _Arguments::parse();

    // Use a single database session for all services
    let session = Arc::new(
        scylla::SessionBuilder::new()
            .known_nodes(arguments.scylla_hosts.split(",").collect::<Vec<&str>>())
            .build()
            .await?,
    );

    // Use a single RabbitMQ channel for all services
    let aqmp_hosts = arguments.amqp_hosts.split(",").collect::<Vec<&str>>();
    let rabbitmq = Arc::new(
        lapin::Connection::connect(
            format!("amqp://{}:5672", aqmp_hosts[0]).as_str(),
            lapin::ConnectionProperties::default()
                .with_executor(tokio_executor_trait::Tokio::current())
                .with_reactor(tokio_reactor_trait::Tokio),
        )
        .await?
        .create_channel()
        .await?,
    );

    Server::builder()
        .add_service(account_service_server::AccountServiceServer::new(
            services::ApplicationService::new(rabbitmq.clone(), session.clone()).await?,
        ))
        .add_service(channel_service_server::ChannelServiceServer::new(
            services::ApplicationService::new(rabbitmq.clone(), session.clone()).await?,
        ))
        .add_service(config_service_server::ConfigServiceServer::new(
            services::ApplicationService::new(rabbitmq.clone(), session.clone()).await?,
        ))
        .serve("0.0.0.0:16000".parse::<SocketAddr>()?)
        .await?;

    Ok(())
}
