use std::sync::atomic;
use std::sync::Arc;

use chrono::{DateTime, Utc};

// Import `impl`s for `ApplicationService`.
mod authorization;
mod channel;
mod config;

pub mod p_authorization {
    tonic::include_proto!("p_authorization");
}

pub mod p_channels {
    tonic::include_proto!("p_channels");
}

pub mod p_config {
    tonic::include_proto!("p_config");
}

pub mod p_status {
    tonic::include_proto!("p_status");
}

pub mod p_users {
    tonic::include_proto!("p_users");
}

#[derive(serde::Deserialize)]
struct SettingsJson {
    #[serde(rename = "bcrypt-cost")]
    bcrypt_cost: u32,
    epoch: i64,
}

pub struct ApplicationService {
    bcrypt_cost: u32,
    epoch: DateTime<Utc>,
    rabbitmq: Arc<lapin::Channel>,
    session: Arc<scylla::Session>,
}

static _ID_COUNTER: atomic::AtomicI16 = atomic::AtomicI16::new(0);

impl ApplicationService {
    pub async fn new(
        rabbitmq: Arc<lapin::Channel>,
        session: Arc<scylla::Session>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        for mut statement in include_str!("../../scripts/database.cql").split(";") {
            statement = statement.trim();
            if !statement.is_empty() {
                session.query_unpaged(statement, ()).await?;
            }
        }

        let json =
            serde_json::from_str::<SettingsJson>(include_str!("../../../../setup.json")).unwrap();

        Ok(Self {
            bcrypt_cost: json.bcrypt_cost,
            epoch: DateTime::from_timestamp_millis(json.epoch).expect("Invalid epoch"),
            rabbitmq,
            session,
        })
    }

    fn generate_id(&self) -> i64 {
        let timedelta = chrono::Utc::now().signed_duration_since(self.epoch);
        let counter = _ID_COUNTER.fetch_add(1, atomic::Ordering::SeqCst); // This operation wraps around on overflow.
        timedelta.num_milliseconds() << 16 | (counter as i64)
    }

    fn hash(&self, password: &str) -> String {
        bcrypt::hash(password, self.bcrypt_cost).unwrap()
    }

    fn verify(&self, password: &str, hash: &str) -> bool {
        bcrypt::verify(password, hash).unwrap_or(false)
    }

    /// Convert an error into a [`tonic::Status`].
    fn error<E>(error: E) -> tonic::Status
    where
        E: std::fmt::Debug,
    {
        tonic::Status::internal(format!("{:?}", error))
    }
}
