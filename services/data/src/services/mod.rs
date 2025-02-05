use std::sync::atomic;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use rand::distr;
use rand::rngs;
use rand::Rng;
use rand::SeedableRng;
use scylla::macros;
use tokio::sync;

// Import `impl`s for `ApplicationService`.
mod account;
mod config;

pub mod authorization_proto {
    tonic::include_proto!("authorization_proto");
}

pub mod config_proto {
    tonic::include_proto!("config_proto");
}

pub mod scalar_proto {
    tonic::include_proto!("scalar_proto");
}

pub mod status_proto {
    tonic::include_proto!("status_proto");
}

#[derive(serde::Deserialize)]
struct SettingsJson {
    #[serde(rename = "bcrypt-cost")]
    bcrypt_cost: u32,
    epoch: i64,
}

#[allow(dead_code)]
#[derive(Debug, macros::DeserializeRow)]
struct InsertCFGText {
    #[scylla(rename = "[applied]")]
    applied: bool,
    key: Option<String>,
    value: Option<String>,
}

pub struct ApplicationService {
    _secret_key: sync::OnceCell<String>,

    bcrypt_cost: u32,
    epoch: DateTime<Utc>,
    session: Arc<scylla::Session>,
}

static _ID_COUNTER: atomic::AtomicI16 = atomic::AtomicI16::new(0);

impl ApplicationService {
    pub async fn new(session: Arc<scylla::Session>) -> Result<Self, Box<dyn std::error::Error>> {
        for mut statement in include_str!("../../scripts/database.cql").split(";") {
            statement = statement.trim();
            if !statement.is_empty() {
                session.query_unpaged(statement, ()).await?;
            }
        }

        let json =
            serde_json::from_str::<SettingsJson>(include_str!("../../../../setup.json")).unwrap();

        Ok(Self {
            _secret_key: sync::OnceCell::new(),
            bcrypt_cost: json.bcrypt_cost,
            epoch: DateTime::from_timestamp_millis(json.epoch).expect("Invalid epoch"),
            session,
        })
    }

    async fn get_secret_key(&self) -> Result<&String, Box<dyn std::error::Error>> {
        async fn _fetch_secret_key(
            session: Arc<scylla::Session>,
        ) -> Result<String, Box<dyn std::error::Error>> {
            let mut rng = rngs::StdRng::from_os_rng();
            let new_key = (0..32)
                .map(|_| rng.sample(distr::Alphanumeric) as char)
                .collect::<String>();
            let row = session
                .query_unpaged(
                    r"
                    INSERT INTO config.cfg_text (key, value)
                    VALUES ('secret_key', ?)
                    IF NOT EXISTS
                    ",
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

        self._secret_key
            .get_or_try_init(|| _fetch_secret_key(self.session.clone()))
            .await
    }

    fn generate_id(&self) -> i64 {
        let timedelta = chrono::Utc::now().signed_duration_since(self.epoch);
        let counter = _ID_COUNTER.fetch_add(1, atomic::Ordering::SeqCst); // This operation wraps around on overflow.
        timedelta.num_milliseconds() << 16 | (counter as i64)
    }

    fn snowflake_time(&self, id: &i64) -> DateTime<Utc> {
        self.epoch + chrono::Duration::milliseconds(id >> 16)
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
