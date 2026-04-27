-- Migration number: 0001 	 2026-04-27T02:38:39.247Z

CREATE TABLE IF NOT EXISTS oauth_redirects (
    state TEXT NOT NULL,
    redirect_uri TEXT NOT NULL,
    expiration TEXT NOT NULL,
    CONSTRAINT pk_oauth_redirects PRIMARY KEY(state)
);
