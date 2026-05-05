-- Migration number: 0003 	 2026-05-05T07:22:22.243Z

CREATE TABLE IF NOT EXISTS user_profile_temporary (
    token TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    expiration TEXT NOT NULL,
    oauth_id TEXT,
    union_id TEXT,
    fetched INTEGER NOT NULL DEFAULT 0,
    CONSTRAINT pk_user_profile_temporary PRIMARY KEY(token),
    CONSTRAINT uq_user_profile_temporary_oauth_id UNIQUE(oauth_id),
    CONSTRAINT uq_user_profile_temporary_union_id UNIQUE(union_id),
    CONSTRAINT chk_user_profile_temporary_fetched_bool CHECK (fetched IN (0, 1))
);
