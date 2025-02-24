use std::collections;
use std::collections::hash_map::Entry;

use lapin::options;
use lapin::types;
use prost::Message;
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
    create_channel: prepared_statement::PreparedStatement,
    create_message1: prepared_statement::PreparedStatement,
    create_message2: prepared_statement::PreparedStatement,
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
struct _AppliedChannelRow {
    #[scylla(rename = "[applied]")]
    applied: bool,
    id: Option<i64>,
    name: Option<String>,
    description: Option<String>,
    owner_id: Option<i64>,
}

#[allow(dead_code)]
#[derive(Debug, macros::DeserializeRow)]
struct _MessageRow {
    id: i64,
    content: String,
    author_id: i64,
    channel_id: i64,
}

#[allow(dead_code)]
#[derive(Debug, macros::DeserializeRow)]
struct _AppliedMessageRow {
    #[scylla(rename = "[applied]")]
    applied: bool,
    id: Option<i64>,
    content: Option<String>,
    author_id: Option<i64>,
    channel_id: Option<i64>,
}

/// Constructs a [`_Statements`] to initialize the singleton [`_STATEMENTS`].
///
/// This function is automatically called by [`sync::OnceCell`], a reference to
/// [`_STATEMENTS`] can be retrieved via:
/// ```rust
/// let statements =_STATEMENTS.get_or_try_init(|| _prepare(application)).await?;
/// ```
async fn _prepare(
    application: &super::ApplicationService,
) -> Result<_Statements, Box<dyn std::error::Error>> {
    let mut create_channel = application
        .session
        .prepare(
            r"INSERT INTO data.channel_by_id (id, name, description, owner_id)
            VALUES (?, ?, ?, ?)
            IF NOT EXISTS",
        )
        .await?;
    create_channel.set_consistency(Consistency::Quorum);

    let mut create_message1 = application
        .session
        .prepare(
            r"INSERT INTO data.message_by_id (id, content, author_id, channel_id)
            VALUES (?, ?, ?, ?)
            IF NOT EXISTS",
        )
        .await?;
    create_message1.set_consistency(Consistency::Quorum);

    let mut create_message2 = application
        .session
        .prepare(
            r"INSERT INTO data.message_by_channel_id (id, content, author_id, channel_id)
            VALUES (?, ?, ?, ?)",
        )
        .await?;
    create_message2.set_consistency(Consistency::One);

    let mut fetch_user = application
        .session
        .prepare(
            r"SELECT id, username, permissions
            FROM accounts.info_by_id
            WHERE id = ?",
        )
        .await?;
    fetch_user.set_consistency(Consistency::One);

    let mut query = application
        .session
        .prepare(
            r"SELECT id, name, description, owner_id
            FROM data.channel_by_id",
        )
        .await?;
    query.set_consistency(Consistency::One);

    let mut history = Vec::new();
    for newest in [false, true] {
        let mut statement = application
            .session
            .prepare(format!(
                r"SELECT id, content, author_id, channel_id
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

    let mut channel = application
        .session
        .prepare(
            r"SELECT id, name, description, owner_id
            FROM data.channel_by_id
            WHERE id = ?",
        )
        .await?;
    channel.set_consistency(Consistency::One);

    Ok(_Statements {
        create_channel,
        create_message1,
        create_message2,
        fetch_user,
        query,
        history,
        channel,
    })
}

async fn _fetch_user(
    application: &super::ApplicationService,
    id: i64,
) -> Result<_UserRow, Box<dyn std::error::Error>> {
    let statements = _STATEMENTS
        .get_or_try_init(|| _prepare(application))
        .await?;
    let row = application
        .session
        .execute_unpaged(&statements.fetch_user, (&id,))
        .await?
        .into_rows_result()?
        .single_row::<_UserRow>()?;

    Ok(row)
}

async fn _fetch_channel(
    application: &super::ApplicationService,
    id: i64,
) -> Result<_ChannelRow, Box<dyn std::error::Error>> {
    let statements = _STATEMENTS
        .get_or_try_init(|| _prepare(application))
        .await?;
    let row = application
        .session
        .execute_unpaged(&statements.channel, (&id,))
        .await?
        .into_rows_result()?
        .single_row::<_ChannelRow>()?;

    Ok(row)
}

#[tonic::async_trait]
impl channel_service_server::ChannelService for super::ApplicationService {
    async fn create_channel(
        &self,
        request: tonic::Request<p_channels::PCreateChannelRequest>,
    ) -> Result<tonic::Response<p_channels::PChannel>, tonic::Status> {
        let request = request.into_inner();
        let statements = _STATEMENTS
            .get_or_try_init(|| _prepare(self))
            .await
            .map_err(super::ApplicationService::error)?;

        let mut id = self.generate_id();
        loop {
            let row = self
                .session
                .execute_unpaged(
                    &statements.create_channel,
                    (&id, &request.name, &request.description, &request.owner_id),
                )
                .await
                .map_err(super::ApplicationService::error)?
                .into_rows_result()
                .map_err(super::ApplicationService::error)?
                .single_row::<_AppliedChannelRow>()
                .map_err(super::ApplicationService::error)?;

            if row.applied {
                break;
            }

            id = self.generate_id();
        }

        Ok(tonic::Response::new(p_channels::PChannel {
            id,
            name: request.name,
            description: request.description,
            owner: Some(
                _fetch_user(self, request.owner_id)
                    .await
                    .map(|user| p_users::PUser {
                        id: user.id,
                        username: user.username,
                        permissions: user.permissions,
                    })
                    .map_err(super::ApplicationService::error)?,
            ),
        }))
    }

    async fn create_message(
        &self,
        request: tonic::Request<p_channels::PCreateMessageRequest>,
    ) -> Result<tonic::Response<p_channels::PMessage>, tonic::Status> {
        let request = request.into_inner();
        let statements = _STATEMENTS
            .get_or_try_init(|| _prepare(self))
            .await
            .map_err(super::ApplicationService::error)?;

        let author = _fetch_user(self, request.author_id)
            .await
            .map_err(super::ApplicationService::error)?;

        let channel = _fetch_channel(self, request.channel_id)
            .await
            .map_err(super::ApplicationService::error)?;

        let mut id = self.generate_id();
        loop {
            let row = self
                .session
                .execute_unpaged(
                    &statements.create_message1,
                    (
                        &id,
                        &request.content,
                        &request.author_id,
                        &request.channel_id,
                    ),
                )
                .await
                .map_err(super::ApplicationService::error)?
                .into_rows_result()
                .map_err(super::ApplicationService::error)?
                .single_row::<_AppliedMessageRow>()
                .map_err(super::ApplicationService::error)?;

            if row.applied {
                break;
            }

            id = self.generate_id();
        }

        self.session
            .execute_unpaged(
                &statements.create_message2,
                (
                    &id,
                    &request.content,
                    &request.author_id,
                    &request.channel_id,
                ),
            )
            .await
            .map_err(super::ApplicationService::error)?;

        let result = p_channels::PMessage {
            id,
            content: request.content,
            author: Some(p_users::PUser {
                id: author.id,
                username: author.username,
                permissions: author.permissions,
            }),
            channel: Some(p_channels::PChannel {
                id: channel.id,
                name: channel.name,
                description: channel.description,
                owner: Some(
                    _fetch_user(self, channel.owner_id)
                        .await
                        .map(|user| p_users::PUser {
                            id: user.id,
                            username: user.username,
                            permissions: user.permissions,
                        })
                        .map_err(super::ApplicationService::error)?,
                ),
            }),
        };

        self.rabbitmq
            .exchange_declare(
                "channel-messages",
                lapin::ExchangeKind::Direct,
                options::ExchangeDeclareOptions::default(),
                types::FieldTable::default(),
            )
            .await
            .map_err(super::ApplicationService::error)?;
        self.rabbitmq
            .basic_publish(
                "channel-messages",
                format!("channel-{}", &request.channel_id).as_str(),
                options::BasicPublishOptions::default(),
                result.encode_to_vec().as_slice(),
                lapin::BasicProperties::default(),
            )
            .await
            .map_err(super::ApplicationService::error)?;

        Ok(tonic::Response::new(result))
    }

    async fn history(
        &self,
        request: tonic::Request<p_channels::PHistoryQuery>,
    ) -> Result<tonic::Response<p_channels::PHistoryQueryResult>, tonic::Status> {
        let request = request.into_inner();
        let statements = _STATEMENTS
            .get_or_try_init(|| _prepare(self))
            .await
            .map_err(super::ApplicationService::error)?;

        let before_id = if request.before_id == 0 {
            i64::MAX
        } else {
            request.before_id
        };
        let after_id = request.after_id;

        let temp = self
            .session
            .execute_unpaged(
                &statements.history[request.newest as usize],
                (request.id, before_id, after_id, request.limit),
            )
            .await
            .map_err(super::ApplicationService::error)?
            .into_rows_result()
            .map_err(super::ApplicationService::error)?;

        let channel = _fetch_channel(self, request.id)
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
            if let Entry::Vacant(e) = authors.entry(row.author_id) {
                e.insert(
                    _fetch_user(self, row.author_id)
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

    async fn query(
        &self,
        request: tonic::Request<p_channels::PChannelQuery>,
    ) -> Result<tonic::Response<p_channels::PChannelQueryResult>, tonic::Status> {
        let request = request.into_inner();
        let statements = _STATEMENTS
            .get_or_try_init(|| _prepare(self))
            .await
            .map_err(super::ApplicationService::error)?;

        let temp = if request.id == 0 {
            self.session
                .execute_unpaged(&statements.query, ())
                .await
                .map_err(super::ApplicationService::error)?
                .into_rows_result()
                .map_err(super::ApplicationService::error)?
                .rows::<_ChannelRow>()
                .map_err(super::ApplicationService::error)?
                .flatten()
                .collect()
        } else {
            let channel = _fetch_channel(self, request.id).await;
            match channel {
                Ok(channel) => vec![channel],
                Err(_) => Vec::new(),
            }
        };

        let mut owners = collections::HashMap::new();
        let mut result = Vec::new();
        for row in temp {
            if let Entry::Vacant(e) = owners.entry(row.owner_id) {
                e.insert(
                    _fetch_user(self, row.owner_id)
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
}
