use scylla::macros;
use scylla::prepared_statement;
use scylla::statement::Consistency;
use tokio::sync;

use super::authorization_proto;
use super::authorization_proto::account_service_server;
use super::status_proto;

/// A singleton of [`Statements`], initialized using [`sync::OnceCell`]
static _STATEMENTS: sync::OnceCell<Statements> = sync::OnceCell::const_new();

/// Private struct holding prepared statements in this module.
///
/// Access the underlying statements via the singleton [`_STATEMENTS`].
/// See also: [`prepare`].
struct Statements {
    account_create_1: prepared_statement::PreparedStatement,
    account_create_2: prepared_statement::PreparedStatement,
    account_login: prepared_statement::PreparedStatement,
}

#[allow(dead_code)]
#[derive(Debug, macros::DeserializeRow)]
struct AccountRow {
    id: i64,
    username: String,
    hashed_password: String,
    permissions: i64,
}

#[allow(dead_code)]
#[derive(Debug, macros::DeserializeRow)]
struct InsertAccountRow {
    #[scylla(rename = "[applied]")]
    applied: bool,
    id: Option<i64>,
    username: Option<String>,
    hashed_password: Option<String>,
    permissions: Option<i64>,
}

/// Constructs a [`Statements`] to initialize the singleton [`_STATEMENTS`].
///
/// This function is automatically called by [`sync::OnceCell`], a reference to
/// [`_STATEMENTS`] can be retrieved via:
/// ```rust
/// let statements = _STATEMENTS.get_or_try_init(|| prepare(session)).await?;
/// ```
async fn prepare(session: &scylla::Session) -> Result<Statements, Box<dyn std::error::Error>> {
    let mut account_create_1 = session
        .prepare(
            r"
            INSERT INTO accounts.info_by_username (id, username, hashed_password, permissions)
            VALUES (?, ?, ?, 0)
            IF NOT EXISTS
            ",
        )
        .await?;
    account_create_1.set_consistency(Consistency::All);

    let mut account_create_2 = session
        .prepare(
            r"
            INSERT INTO accounts.info_by_id (id, username, hashed_password, permissions)
            VALUES (?, ?, ?, 0)
            IF NOT EXISTS
            ",
        )
        .await?;
    account_create_2.set_consistency(Consistency::All);

    let mut account_login = session
        .prepare(
            r"
            SELECT id, username, hashed_password, permissions
            FROM accounts.info_by_username
            WHERE username = ?
            ",
        )
        .await?;
    account_login.set_consistency(Consistency::All);

    Ok(Statements {
        account_create_1,
        account_create_2,
        account_login,
    })
}

#[tonic::async_trait]
impl account_service_server::AccountService for super::ApplicationService {
    async fn create(
        &self,
        request: tonic::Request<authorization_proto::InfoMessage>,
    ) -> Result<tonic::Response<status_proto::StatusMessage>, tonic::Status> {
        let message = request.into_inner();
        let statements = _STATEMENTS
            .get_or_try_init(|| prepare(&self.session))
            .await
            .map_err(super::ApplicationService::error)?;

        let id = self.generate_id();
        let hashed_password = self.hash(&message.password);

        let try_insert = |statement: &'static prepared_statement::PreparedStatement| async {
            Ok::<bool, tonic::Status>(
                self.session
                    .execute_unpaged(
                        &statement.clone(),
                        (&id, &message.username, &hashed_password),
                    )
                    .await
                    .map_err(super::ApplicationService::error)?
                    .into_rows_result()
                    .map_err(super::ApplicationService::error)?
                    .single_row::<InsertAccountRow>()
                    .map_err(super::ApplicationService::error)?
                    .applied,
            )
        };

        if try_insert(&statements.account_create_1).await?
            && try_insert(&statements.account_create_2).await?
        {
            Ok(tonic::Response::new(status_proto::StatusMessage {
                success: true,
                message: "Created a new account".to_string(),
            }))
        } else {
            Err(tonic::Status::already_exists("Username already exists"))
        }
    }

    async fn login(
        &self,
        request: tonic::Request<authorization_proto::InfoMessage>,
    ) -> Result<tonic::Response<authorization_proto::AccountMessage>, tonic::Status> {
        let message = request.into_inner();
        let statements = _STATEMENTS
            .get_or_try_init(|| prepare(&self.session))
            .await
            .map_err(super::ApplicationService::error)?;

        let row = self
            .session
            .execute_unpaged(&statements.account_login, (&message.username,))
            .await
            .map_err(super::ApplicationService::error)?
            .into_rows_result()
            .map_err(super::ApplicationService::error)?
            .single_row::<AccountRow>()
            .map_err(|_| tonic::Status::unauthenticated("Invalid credentials"))?;

        if self.verify(&message.password, &row.hashed_password) {
            Ok(tonic::Response::new(authorization_proto::AccountMessage {
                id: row.id,
                username: row.username,
                permissions: row.permissions,
            }))
        } else {
            Err(tonic::Status::unauthenticated("Invalid credentials"))
        }
    }
}
