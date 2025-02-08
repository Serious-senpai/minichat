use std::collections;
use std::sync::Arc;

use scylla::macros;
use scylla::prepared_statement;
use scylla::statement::Consistency;
use tokio::sync;

use super::p_channels;
use super::p_channels::channel_service_server;
use super::p_users;

/// A singleton of [`_Statements`], initialized using [`sync::OnceCell`]
static _STATEMENTS: sync::OnceCell<_Statements> = sync::OnceCell::const_new();

/// Private struct holding prepared statements in this module.
///
/// Access the underlying statements via the singleton [`_STATEMENTS`].
/// See also: [`_prepare`].
struct _Statements {
    fetch_user: prepared_statement::PreparedStatement,
    query: prepared_statement::PreparedStatement,
    history: Vec<prepared_statement::PreparedStatement>,
    channel: prepared_statement::PreparedStatement,
}

#[allow(dead_code)]
#[derive(Debug, macros::DeserializeRow)]
struct _UserRow {
    id: i64,
    username: String,
    permissions: i64,
}

#[allow(dead_code)]
#[derive(Debug, macros::DeserializeRow)]
struct _ChannelRow {
    id: i64,
    name: String,
    description: String,
    owner_id: i64,
}

#[allow(dead_code)]
#[derive(Debug, macros::DeserializeRow)]
struct _MessageRow {
    id: i64,
    content: String,
    author_id: i64,
}

/// Constructs a [`_Statements`] to initialize the singleton [`_STATEMENTS`].
///
/// This function is automatically called by [`sync::OnceCell`], a reference to
/// [`_STATEMENTS`] can be retrieved via:
/// ```rust
/// let statements =_STATEMENTS.get_or_try_init(|| _prepare(session)).await?;
/// ```
async fn _prepare(
    session: Arc<scylla::Session>,
) -> Result<_Statements, Box<dyn std::error::Error>> {
    let mut fetch_user = session
        .prepare(
            r"SELECT id, username, permissions
            FROM accounts.info_by_id
            WHERE id = ?",
        )
        .await?;
    fetch_user.set_consistency(Consistency::One);

    let mut query = session
        .prepare(
            r"SELECT id, name, description, owner_id
            FROM data.channel_by_id",
        )
        .await?;
    query.set_consistency(Consistency::One);

    let mut history = Vec::new();
    for newest in [false, true] {
        let mut statement = session
            .prepare(format!(
                r"SELECT id, content, author_id
                FROM data.message_by_channel_id
                WHERE channel_id = ? AND id <= ? AND id >= ?
                ORDER BY id {}
                LIMIT ?",
                if newest { "DESC" } else { "ASC" }
            ))
            .await?;
        statement.set_consistency(Consistency::One);

        history.push(statement);
    }

    let mut channel = session
        .prepare(
            r"SELECT id, name, description, owner_id
            FROM data.channel_by_id
            WHERE id = ?",
        )
        .await?;
    channel.set_consistency(Consistency::One);

    Ok(_Statements {
        fetch_user,
        query,
        history,
        channel,
    })
}

async fn _fetch_user(
    session: Arc<scylla::Session>,
    id: i64,
) -> Result<_UserRow, Box<dyn std::error::Error>> {
    let statements = _STATEMENTS
        .get_or_try_init(|| _prepare(session.clone()))
        .await?;
    let row = session
        .execute_unpaged(&statements.fetch_user, (&id,))
        .await?
        .into_rows_result()?
        .single_row::<_UserRow>()?;

    Ok(row)
}

async fn _fetch_channel(
    session: Arc<scylla::Session>,
    id: i64,
) -> Result<_ChannelRow, Box<dyn std::error::Error>> {
    let statements = _STATEMENTS
        .get_or_try_init(|| _prepare(session.clone()))
        .await?;
    let row = session
        .execute_unpaged(&statements.channel, (&id,))
        .await?
        .into_rows_result()?
        .single_row::<_ChannelRow>()?;

    Ok(row)
}

#[tonic::async_trait]
impl channel_service_server::ChannelService for super::ApplicationService {
    async fn query(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<p_channels::PChannelQueryResult>, tonic::Status> {
        let statements = _STATEMENTS
            .get_or_try_init(|| _prepare(self.session.clone()))
            .await
            .map_err(super::ApplicationService::error)?;

        let temp = self
            .session
            .execute_unpaged(&statements.query, ())
            .await
            .map_err(super::ApplicationService::error)?
            .into_rows_result()
            .map_err(super::ApplicationService::error)?;

        let mut owners = collections::HashMap::new();
        let mut result = Vec::new();
        for row in temp
            .rows::<_ChannelRow>()
            .map_err(super::ApplicationService::error)?
            .flatten()
        {
            if !owners.contains_key(&row.owner_id) {
                owners.insert(
                    row.owner_id,
                    _fetch_user(self.session.clone(), row.owner_id)
                        .await
                        .map_err(super::ApplicationService::error)
                        .map(|user| p_users::PUser {
                            id: user.id,
                            username: user.username,
                            permissions: user.permissions,
                        })?,
                );
            }

            result.push(p_channels::PChannel {
                id: row.id,
                name: row.name,
                description: row.description,
                owner: Some(owners[&row.owner_id].clone()),
            });
        }

        Ok(tonic::Response::new(p_channels::PChannelQueryResult {
            channels: result,
        }))
    }

    async fn history(
        &self,
        request: tonic::Request<p_channels::PHistoryQuery>,
    ) -> Result<tonic::Response<p_channels::PHistoryQueryResult>, tonic::Status> {
        let message = request.into_inner();
        let statements = _STATEMENTS
            .get_or_try_init(|| _prepare(self.session.clone()))
            .await
            .map_err(super::ApplicationService::error)?;

        let before_id = message.before_id.unwrap_or(i64::MAX);
        let after_id = message.after_id.unwrap_or(i64::MIN);

        let temp = self
            .session
            .execute_unpaged(
                &statements.history[message.newest as usize],
                (message.id, before_id, after_id, message.limit),
            )
            .await
            .map_err(super::ApplicationService::error)?
            .into_rows_result()
            .map_err(super::ApplicationService::error)?;

        let channel = _fetch_channel(self.session.clone(), message.id)
            .await
            .map(|channel| p_channels::PChannel {
                id: channel.id,
                name: channel.name,
                description: channel.description,
                owner: None,
            })
            .map_err(super::ApplicationService::error)?;

        let mut authors = collections::HashMap::new();
        let mut result = Vec::new();
        for row in temp
            .rows::<_MessageRow>()
            .map_err(super::ApplicationService::error)?
            .flatten()
        {
            if !authors.contains_key(&row.author_id) {
                authors.insert(
                    row.author_id,
                    _fetch_user(self.session.clone(), row.author_id)
                        .await
                        .map_err(super::ApplicationService::error)
                        .map(|user| p_users::PUser {
                            id: user.id,
                            username: user.username,
                            permissions: user.permissions,
                        })?,
                );
            }

            result.push(p_channels::PMessage {
                id: row.id,
                content: row.content,
                author: Some(authors[&row.author_id].clone()),
                channel: Some(channel.clone()),
            });
        }

        Ok(tonic::Response::new(p_channels::PHistoryQueryResult {
            messages: result,
        }))
    }
}
