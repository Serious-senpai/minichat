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
    account_create_3: prepared_statement::PreparedStatement,
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
/// let statements = _STATEMENTS.get_or_try_init(|| _prepare(session)).await?;
/// ```
async fn _prepare(session: &scylla::Session) -> Result<Statements, Box<dyn std::error::Error>> {
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

    let mut account_create_3 = session
        .prepare(
            r"
            UPDATE accounts.info_by_username
            SET id = ?
            WHERE username = ?
            ",
        )
        .await?;
    account_create_3.set_consistency(Consistency::All);

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
        account_create_3,
        account_login,
    })
}

async fn _create_helper(
    service: &super::ApplicationService,
    id: &i64,
    statement: &prepared_statement::PreparedStatement,
    username: &str,
    hashed_password: &str,
) -> Result<bool, tonic::Status> {
    Ok(service
        .session
        .execute_unpaged(&statement.clone(), (&id, username, hashed_password))
        .await
        .map_err(super::ApplicationService::error)?
        .into_rows_result()
        .map_err(super::ApplicationService::error)?
        .single_row::<InsertAccountRow>()
        .map_err(super::ApplicationService::error)?
        .applied)
}

#[tonic::async_trait]
impl account_service_server::AccountService for super::ApplicationService {
    async fn create(
        &self,
        request: tonic::Request<authorization_proto::InfoMessage>,
    ) -> Result<tonic::Response<status_proto::StatusMessage>, tonic::Status> {
        let message = request.into_inner();
        let statements = _STATEMENTS
            .get_or_try_init(|| _prepare(&self.session))
            .await
            .map_err(super::ApplicationService::error)?;

        let id = self.generate_id();
        let hashed_password = self.hash(&message.password);
        let hashed_password = &hashed_password;

        let success = Ok::<tonic::Response<status_proto::StatusMessage>, tonic::Status>(
            tonic::Response::new(status_proto::StatusMessage {
                success: true,
                message: "Created a new account".to_string(),
            }),
        );

        if _create_helper(
            self,
            &id,
            &statements.account_create_1,
            &message.username,
            hashed_password,
        )
        .await?
        {
            // Username is unique, insert into the second table
            if _create_helper(
                self,
                &id,
                &statements.account_create_2,
                &message.username,
                hashed_password,
            )
            .await?
            {
                // ID is unique
                success
            } else {
                // ID already exists (this is very unlikely, but may happen during extremely high concurrency scenarios).
                // In this case, we repeatedly generate a new ID and try inserting.
                let mut id = self.generate_id();
                loop {
                    if _create_helper(
                        self,
                        &id,
                        &statements.account_create_2,
                        &message.username,
                        hashed_password,
                    )
                    .await?
                    {
                        break;
                    }

                    id = self.generate_id();
                }

                // At this point, the ID inserted to the second table is guaranteed to be unique.
                // We now update the first table.
                self.session
                    .execute_unpaged(&statements.account_create_3, (id, &message.username))
                    .await
                    .map_err(super::ApplicationService::error)?;
                success
            }
        } else {
            // Username already exists
            Err(tonic::Status::already_exists("Username already exists"))
        }
    }

    async fn login(
        &self,
        request: tonic::Request<authorization_proto::InfoMessage>,
    ) -> Result<tonic::Response<authorization_proto::AccountMessage>, tonic::Status> {
        let message = request.into_inner();
        let statements = _STATEMENTS
            .get_or_try_init(|| _prepare(&self.session))
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
