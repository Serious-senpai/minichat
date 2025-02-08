CREATE KEYSPACE IF NOT EXISTS accounts
WITH replication = {'class': 'NetworkTopologyStrategy', 'datacenter1': 3}
AND tablets = {'enabled': false};

CREATE TABLE IF NOT EXISTS accounts.info_by_username (
    id BIGINT,
    username TEXT,
    hashed_password TEXT,
    permissions BIGINT,
    PRIMARY KEY (username)
);

CREATE TABLE IF NOT EXISTS accounts.info_by_id (
    id BIGINT,
    username TEXT,
    hashed_password TEXT,
    permissions BIGINT,
    PRIMARY KEY (id)
);

CREATE KEYSPACE IF NOT EXISTS config
WITH replication = {'class': 'NetworkTopologyStrategy', 'datacenter1': 3}
AND tablets = {'enabled': false};

CREATE TABLE IF NOT EXISTS config.cfg_text (
    key TEXT,
    value TEXT,
    PRIMARY KEY (key)
);

CREATE KEYSPACE IF NOT EXISTS data
WITH replication = {'class': 'NetworkTopologyStrategy', 'datacenter1': 3}
AND tablets = {'enabled': false};

CREATE TABLE IF NOT EXISTS data.channel_by_id (
    id BIGINT,
    name TEXT,
    description TEXT,
    owner_id BIGINT,
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS data.message_by_id (
    id BIGINT,
    content TEXT,
    author_id BIGINT,
    channel_id BIGINT,
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS data.message_by_channel_id (
    id BIGINT,
    content TEXT,
    author_id BIGINT,
    channel_id BIGINT,
    PRIMARY KEY (channel_id, id)
);
