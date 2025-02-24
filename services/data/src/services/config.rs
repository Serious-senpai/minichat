use std::sync::Arc;

use rand::distr;
use rand::rngs;
use rand::Rng;
use rand::SeedableRng;
use scylla::macros;
use tokio::sync;

use super::p_config;
use super::p_config::config_service_server;
use super::p_config::PConfigType;

static SECRET_KEY: sync::OnceCell<String> = sync::OnceCell::const_new();

#[allow(dead_code)]
#[derive(Debug, macros::DeserializeRow)]
struct InsertCFGText {
    #[scylla(rename = "[applied]")]
    applied: bool,
    key: Option<String>,
    value: Option<String>,
}

async fn _get_secret_key(
    session: &Arc<scylla::Session>,
) -> Result<&String, Box<dyn std::error::Error>> {
    async fn _fetch_secret_key(
        session: Arc<scylla::Session>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut rng = rngs::StdRng::from_os_rng();
        let new_key = (0..32)
            .map(|_| rng.sample(distr::Alphanumeric) as char)
            .collect::<String>();
        let row = session
            .query_unpaged(
                r"INSERT INTO config.cfg_text (key, value)
                VALUES ('secret_key', ?)
                IF NOT EXISTS",
                (&new_key,),
            )
            .await?
            .into_rows_result()?
            .single_row::<InsertCFGText>()?;

        if row.applied {
            Ok(new_key)
        } else {
            row.value.ok_or("Unable to fetch secret key".into())
        }
    }

    SECRET_KEY
        .get_or_try_init(|| _fetch_secret_key(session.clone()))
        .await
}

#[tonic::async_trait]
impl config_service_server::ConfigService for super::ApplicationService {
    async fn string_config(
        &self,
        request: tonic::Request<p_config::PConfigRequest>,
    ) -> Result<tonic::Response<String>, tonic::Status> {
        let request = request.into_inner();
        let config_type = request
            .config_type
            .try_into()
            .map_err(|_| tonic::Status::invalid_argument("Invalid config type"))?;

        match config_type {
            PConfigType::SecretKey => Ok(tonic::Response::new(
                _get_secret_key(&self.session)
                    .await
                    .map_err(super::ApplicationService::error)?
                    .clone(),
            )),
        }
    }
}
