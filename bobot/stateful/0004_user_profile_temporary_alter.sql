-- Migration number: 0004 	 2026-05-05T10:44:25.678Z

DROP TABLE IF EXISTS user_profile_temporary;
CREATE TABLE IF NOT EXISTS user_profile_temporary (
    token TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    expiration TEXT NOT NULL,
    oauth_id TEXT,
    union_id TEXT,
    fetched INTEGER NOT NULL DEFAULT 0,
    CONSTRAINT pk_user_profile_temporary PRIMARY KEY(token),
    CONSTRAINT chk_user_profile_temporary_fetched_bool CHECK (fetched IN (0, 1))
);
