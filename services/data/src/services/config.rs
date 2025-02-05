use super::config_proto;
use super::config_proto::config_service_server;
use super::config_proto::ConfigType;
use super::scalar_proto;

#[tonic::async_trait]
impl config_service_server::ConfigService for super::ApplicationService {
    async fn string_config(
        &self,
        request: tonic::Request<config_proto::ConfigRequestMessage>,
    ) -> Result<tonic::Response<scalar_proto::StringMessage>, tonic::Status> {
        let message = request.into_inner();
        let config_type = message
            .config_type
            .try_into()
            .map_err(|_| tonic::Status::invalid_argument("Invalid config type"))?;

        match config_type {
            ConfigType::SecretKey => Ok(tonic::Response::new(scalar_proto::StringMessage {
                value: self
                    .get_secret_key()
                    .await
                    .map_err(super::ApplicationService::error)?
                    .clone(),
            })),
        }
    }
}
