-- Migration number: 0002 	 2026-05-03T09:54:21.620Z

CREATE TABLE IF NOT EXISTS redirect_uri_allow_list (
    id INTEGER NOT NULL,
    redirect_uri TEXT NOT NULL,
    CONSTRAINT pk_redirect_uri_allow_list PRIMARY KEY (id AUTOINCREMENT),
    CONSTRAINT uq_redirect_uri_allow_list_uri UNIQUE (redirect_uri),
    CONSTRAINT chk_redirect_uri_allow_list_uri_scheme CHECK (
        redirect_uri LIKE 'https://%' OR
        redirect_uri LIKE 'http://localhost%' OR
        redirect_uri LIKE 'http://127.0.0.1%'
    )
);
