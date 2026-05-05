-- Runs as the function owner (superuser), so RLS is bypassed
CREATE OR REPLACE FUNCTION bobot.my_permission()
    RETURNS public.user_permission
    LANGUAGE sql
    SECURITY DEFINER
    SET search_path = ''
    STABLE
AS $$
    SELECT * FROM public.user_permission WHERE id = auth.uid();
$$;

DROP POLICY "only verified user have access" ON public.user_permission;
CREATE POLICY "only verified user have access"
    ON public.user_permission
    AS RESTRICTIVE
    FOR ALL
    TO authenticated
USING ((bobot.my_permission()).verified = true);

DROP POLICY "moderator can view every user's permission" ON public.user_permission;
CREATE POLICY "moderator can view every user's permission"
    ON public.user_permission
    AS PERMISSIVE
    FOR SELECT
    TO authenticated
USING ((bobot.my_permission()).moderate = true);

DROP POLICY "moderator can edit every user's permission" ON public.user_permission;
CREATE POLICY "moderator can edit every user's permission"
    ON public.user_permission
    AS PERMISSIVE
    FOR UPDATE
    TO authenticated
USING ((bobot.my_permission()).moderate = true);