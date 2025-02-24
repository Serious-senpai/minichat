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
    /// The host to bind the gRPC server to
    #[arg(long, default_value_t = String::from("0.0.0.0"))]
    host: String,

    /// The port to bind the gRPC server to
    #[arg(long, default_value_t = 16000)]
    port: u16,

    /// A comma-separated list of ScyllaDB clustered hosts to connect to
    #[arg(long, default_value_t = String::from("minichat-scylla-1,minichat-scylla-2,minichat-scylla-3"))]
    scylla_hosts: String,

    /// A RabbitMQ URL to connect to
    #[arg(long, default_value_t = String::from("amqp://guest:guest@rabbitmq-proxy:5672"))]
    amqp_host: String,
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
    let rabbitmq = Arc::new(
        lapin::Connection::connect(
            arguments.amqp_host.as_str(),
            lapin::ConnectionProperties::default()
                .with_executor(tokio_executor_trait::Tokio::current())
                .with_reactor(tokio_reactor_trait::Tokio),
        )
        .await?
        .create_channel()
        .await?,
    );

    println!("Listening on {}:{}", arguments.host, arguments.port);
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
        .serve(format!("{}:{}", arguments.host, arguments.port).parse::<SocketAddr>()?)
        .await?;

    Ok(())
}
