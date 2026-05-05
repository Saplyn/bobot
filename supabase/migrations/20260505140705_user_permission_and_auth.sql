-- permission specifier, each row represents a capability
CREATE TABLE IF NOT EXISTS public.user_permission (
    id UUID NOT NULL,
    verified BOOLEAN NOT NULL DEFAULT false,  -- foundational permission
    moderate BOOLEAN NOT NULL DEFAULT false,  -- managing user's permission
    CONSTRAINT user_permission_pkey PRIMARY KEY (id),
    CONSTRAINT user_permission_id_fkey FOREIGN KEY (id) REFERENCES auth.users(id) ON UPDATE CASCADE ON DELETE CASCADE
) TABLESPACE pg_default;

CREATE POLICY "only verified user have access"
    ON public.user_permission
    AS RESTRICTIVE
    FOR ALL
    TO authenticated
USING (
    EXISTS (
        SELECT 1
        FROM public.user_permission
        WHERE id = auth.uid() AND verified = true
    )
);

CREATE POLICY "user can view their own permission"
    ON public.user_permission
    AS PERMISSIVE
    FOR SELECT
    TO authenticated
USING ((select auth.uid()) = id);

CREATE POLICY "moderator can view every user's permission"
    ON public.user_permission
    AS PERMISSIVE
    FOR SELECT
    TO authenticated
USING (
    EXISTS (
        SELECT 1
        FROM public.user_permission
        WHERE id = auth.uid() AND moderate = true
    )
);

CREATE POLICY "moderator can edit every user's permission"
    ON public.user_permission
    AS PERMISSIVE
    FOR UPDATE
    TO authenticated
USING (
    EXISTS (
        SELECT 1
        FROM public.user_permission
        WHERE id = auth.uid() AND moderate = true
    )
);

-- protected schema for internal information and methods
CREATE SCHEMA IF NOT EXISTS bobot;

-- (protected) custom user auth table, for identifying user across apps
CREATE TABLE bobot.user_auth (
    id UUID NOT NULL,
    oauth_id TEXT NOT NULL,
    oauth_token TEXT NOT NULL,
    oauth_refresh TEXT NOT NULL,
    oauth_expiration TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    union_id TEXT NOT NULL,
    qqbot_id TEXT NULL,
    CONSTRAINT user_auth_pkey PRIMARY KEY (id),
    CONSTRAINT user_auth_id_fkey FOREIGN KEY (id) REFERENCES auth.users(id) ON UPDATE CASCADE ON DELETE CASCADE
) TABLESPACE pg_default;

-- enable `ext:http` for making network requests
CREATE EXTENSION IF NOT EXISTS http WITH SCHEMA extensions;

-- (protected) populate custom user auth info and initialize all user related tables
CREATE OR REPLACE FUNCTION bobot.handle_new_user()
    RETURNS trigger
    LANGUAGE plpgsql
    SECURITY DEFINER
    SET search_path = ''
AS $$
DECLARE
    oauth_id     text;
    lyn_key      text;
    http_status  integer;
    http_body    jsonb;
BEGIN
    oauth_id := new.raw_user_meta_data->>'sub';
    IF oauth_id IS NULL THEN
        RAISE EXCEPTION 'missing oauth_id in `raw_user_meta_data.sub`';
    END IF;
    RAISE LOG 'handle new user signup with oauth_id=%', oauth_id;

    SELECT decrypted_secret
    INTO lyn_key
    FROM vault.decrypted_secrets
    WHERE name = 'LYN_KEY_SUPABASE';

    SELECT status, content::jsonb
    INTO http_status, http_body
    FROM extensions.http((
        'GET',
        'https://oauth.bobo.saplyn.site/profile?oauth_id=' || oauth_id,
        ARRAY[extensions.http_header('Authorization', 'Bearer ' || lyn_key)],
        NULL,
        NULL
    )::extensions.http_request);

    RAISE LOG 'received response from `/profile`=%', http_body;

    IF http_status <> 200 THEN
        RAISE EXCEPTION 'OAuth profile fetch failed: HTTP %', http_status;
    END IF;

    INSERT INTO bobot.user_auth (
        id,
        oauth_id,
        oauth_token,
        oauth_refresh,
        oauth_expiration,
        union_id
    ) VALUES (
        new.id,
        http_body->>'oauth_id',
        http_body->>'token',
        http_body->>'refresh_token',
        (http_body->>'expiration')::timestamp,
        http_body->>'union_id'
    );

    INSERT INTO public.user_permission (id)
    VALUES (new.id);

    RETURN new;
END;
$$;
-- trigger `fn:handle_new_user()` upon new sign up
CREATE TRIGGER handle_new_user
    AFTER INSERT ON auth.users
    FOR EACH ROW
    EXECUTE FUNCTION bobot.handle_new_user();

-- protect `fn:rls_auto_enable()` so warning in dashboard goes away
ALTER FUNCTION public.rls_auto_enable()
    SET SCHEMA bobot;
