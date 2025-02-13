use scylla::macros;
use scylla::prepared_statement;
use scylla::statement::Consistency;
use tokio::sync;

use super::p_authorization;
use super::p_authorization::account_service_server;
use super::p_status;
use super::p_users;

/// A singleton of [`_Statements`], initialized using [`sync::OnceCell`]
static _STATEMENTS: sync::OnceCell<_Statements> = sync::OnceCell::const_new();

/// Private struct holding prepared statements in this module.
///
/// Access the underlying statements via the singleton [`_STATEMENTS`].
/// See also: [`_prepare`].
struct _Statements {
    create1: prepared_statement::PreparedStatement,
    create2: prepared_statement::PreparedStatement,
    create3: prepared_statement::PreparedStatement,
    login: prepared_statement::PreparedStatement,
}

#[allow(dead_code)]
#[derive(Debug, macros::DeserializeRow)]
struct _AccountRow {
    id: i64,
    username: String,
    hashed_password: String,
    permissions: i64,
}

#[allow(dead_code)]
#[derive(Debug, macros::DeserializeRow)]
struct _InsertAccountRow {
    #[scylla(rename = "[applied]")]
    applied: bool,
    id: Option<i64>,
    username: Option<String>,
    hashed_password: Option<String>,
    permissions: Option<i64>,
}

/// Constructs a [`_Statements`] to initialize the singleton [`_STATEMENTS`].
///
/// This function is automatically called by [`sync::OnceCell`], a reference to
/// [`_STATEMENTS`] can be retrieved via:
/// ```rust
/// let statements = _STATEMENTS.get_or_try_init(|| _prepare(application)).await?;
/// ```
async fn _prepare(
    application: &super::ApplicationService,
) -> Result<_Statements, Box<dyn std::error::Error>> {
    let mut create1 = application
        .session
        .prepare(
            r"INSERT INTO accounts.info_by_username (id, username, hashed_password, permissions)
            VALUES (?, ?, ?, 0)
            IF NOT EXISTS",
        )
        .await?;
    create1.set_consistency(Consistency::All);

    let mut create2 = application
        .session
        .prepare(
            r"INSERT INTO accounts.info_by_id (id, username, hashed_password, permissions)
            VALUES (?, ?, ?, 0)
            IF NOT EXISTS",
        )
        .await?;
    create2.set_consistency(Consistency::All);

    let mut create3 = application
        .session
        .prepare(
            r"UPDATE accounts.info_by_username
            SET id = ?
            WHERE username = ?",
        )
        .await?;
    create3.set_consistency(Consistency::All);

    let mut login = application
        .session
        .prepare(
            r"SELECT id, username, hashed_password, permissions
            FROM accounts.info_by_username
            WHERE username = ?",
        )
        .await?;
    login.set_consistency(Consistency::All);

    Ok(_Statements {
        create1,
        create2,
        create3,
        login,
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
        .single_row::<_InsertAccountRow>()
        .map_err(super::ApplicationService::error)?
        .applied)
}

#[tonic::async_trait]
impl account_service_server::AccountService for super::ApplicationService {
    async fn create(
        &self,
        request: tonic::Request<p_authorization::PAuthInfo>,
    ) -> Result<tonic::Response<p_status::PStatus>, tonic::Status> {
        let message = request.into_inner();
        let statements = _STATEMENTS
            .get_or_try_init(|| _prepare(self))
            .await
            .map_err(super::ApplicationService::error)?;

        let id = self.generate_id();
        let hashed_password = self.hash(&message.password);
        let hashed_password = &hashed_password;

        let success = Ok::<tonic::Response<p_status::PStatus>, tonic::Status>(
            tonic::Response::new(p_status::PStatus {
                success: true,
                message: "Created a new account".to_string(),
            }),
        );

        if _create_helper(
            self,
            &id,
            &statements.create1,
            &message.username,
            hashed_password,
        )
        .await?
        {
            // Username is unique, insert into the second table
            if _create_helper(
                self,
                &id,
                &statements.create2,
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
                        &statements.create2,
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
                    .execute_unpaged(&statements.create3, (id, &message.username))
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
        request: tonic::Request<p_authorization::PAuthInfo>,
    ) -> Result<tonic::Response<p_users::PUser>, tonic::Status> {
        let message = request.into_inner();
        let statements = _STATEMENTS
            .get_or_try_init(|| _prepare(self))
            .await
            .map_err(super::ApplicationService::error)?;

        let row = self
            .session
            .execute_unpaged(&statements.login, (&message.username,))
            .await
            .map_err(super::ApplicationService::error)?
            .into_rows_result()
            .map_err(super::ApplicationService::error)?
            .single_row::<_AccountRow>()
            .map_err(|_| tonic::Status::unauthenticated("Invalid credentials"))?;

        if self.verify(&message.password, &row.hashed_password) {
            Ok(tonic::Response::new(p_users::PUser {
                id: row.id,
                username: row.username,
                permissions: row.permissions,
            }))
        } else {
            Err(tonic::Status::unauthenticated("Invalid credentials"))
        }
    }
}
