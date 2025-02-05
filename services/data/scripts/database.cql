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
