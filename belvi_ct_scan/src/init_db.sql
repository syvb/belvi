-- SPDX-License-Identifier: Apache-2.0
-- Run every time the SQLite database is loaded.

-- INITALIZE TABLES --

CREATE TABLE IF NOT EXISTS meta(
    k TEXT PRIMARY KEY,
    v TEXT
); -- WITH ROWID

-- if schema is changed in future, an actual system for migrating the DB will be implemented
-- for now, just assume we are always on the initial version
INSERT OR REPLACE into meta (k, v) values ("migration", "1.0.0");

CREATE TABLE IF NOT EXISTS certs (
    leaf_hash BLOB PRIMARY KEY NOT NULL,
    extra_hash BLOB NOT NULL,
    ts NUMBER NOT NULL
) WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS domains (
    -- TODO: multiple certs for same domain
    domain TEXT PRIMARY KEY NOT NULL,
    leaf_hash BLOB NOT NULL
) WITHOUT ROWID;

-- CONFIGURE SQLITE --
PRAGMA journal_mode = WAL;
PRAGMA encoding = 'UTF-8';
PRAGMA user_version = 1;

-- OPTIMIZE DB --
PRAGMA optimize;
VACUUM;
