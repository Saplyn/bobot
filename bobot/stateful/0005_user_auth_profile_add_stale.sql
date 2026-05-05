-- Migration number: 0005 	 2026-05-05T16:02:44.910Z

DROP TABLE IF EXISTS user_profile_temporary;
CREATE TABLE IF NOT EXISTS user_auth_profile (
    token TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    expiration TEXT NOT NULL,
    oauth_id TEXT,
    union_id TEXT,
    stale TEXT NOT NULL,
    CONSTRAINT pk_user_auth_profile PRIMARY KEY (token)
);
