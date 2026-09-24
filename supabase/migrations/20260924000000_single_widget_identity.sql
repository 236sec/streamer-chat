-- Only the authenticated resolver may create an active widget identity.
ALTER TABLE public.widgets ADD CONSTRAINT widgets_user_id_id_unique UNIQUE (user_id, id);
CREATE TABLE public.widget_identities (
  user_id UUID PRIMARY KEY REFERENCES auth.users(id) ON DELETE CASCADE,
  widget_id UUID NOT NULL UNIQUE,
  CONSTRAINT widget_identity_owned_row FOREIGN KEY (user_id, widget_id)
    REFERENCES public.widgets(user_id, id) DEFERRABLE INITIALLY DEFERRED
);

ALTER TABLE public.widget_identities ENABLE ROW LEVEL SECURITY;
REVOKE ALL ON public.widget_identities FROM PUBLIC, anon, authenticated;

CREATE FUNCTION public.current_account_widget_id()
RETURNS UUID LANGUAGE sql STABLE SECURITY DEFINER SET search_path = '' AS $$
  SELECT i.widget_id FROM public.widget_identities i WHERE i.user_id = auth.uid();
$$;
REVOKE ALL ON FUNCTION public.current_account_widget_id() FROM PUBLIC, anon;
GRANT EXECUTE ON FUNCTION public.current_account_widget_id() TO authenticated;

DROP POLICY "Public read access for widgets using id" ON public.widgets;
DROP POLICY "Restricted updates to authenticated owner" ON public.widgets;
DROP POLICY "Restricted inserts to authenticated owner" ON public.widgets;
DROP POLICY "Restricted deletes to authenticated owner" ON public.widgets;

CREATE POLICY "Owner reads canonical widget" ON public.widgets FOR SELECT TO authenticated
  USING (user_id = (SELECT auth.uid()) AND id = (SELECT public.current_account_widget_id()));
CREATE POLICY "Owner updates canonical widget" ON public.widgets FOR UPDATE TO authenticated
  USING (user_id = (SELECT auth.uid()) AND id = (SELECT public.current_account_widget_id()))
  WITH CHECK (user_id = (SELECT auth.uid()) AND id = (SELECT public.current_account_widget_id()));
REVOKE ALL ON public.widgets FROM PUBLIC, anon, authenticated;
GRANT SELECT ON public.widgets TO authenticated;
GRANT UPDATE (theme, font_size, background_color, auto_hide_seconds, layout_style)
  ON public.widgets TO authenticated;

-- A valid pin must be a normalized message belonging to its source row.
CREATE FUNCTION public.widget_pin_valid(message JSONB, source_id UUID)
RETURNS BOOLEAN LANGUAGE plpgsql IMMUTABLE SET search_path = '' AS $$
DECLARE fragment JSONB;
BEGIN
  IF message IS NULL OR jsonb_typeof(message) <> 'object' THEN RETURN false; END IF;
  IF message->>'type' IS DISTINCT FROM 'chat_message'
    OR message->>'widget_id' IS DISTINCT FROM source_id::text
    OR COALESCE(message->>'platform' IN ('twitch', 'youtube', 'kick'), false) = false
    OR COALESCE(length(message->>'id') BETWEEN 1 AND 256, false) = false
    OR COALESCE(length(message->>'author') BETWEEN 1 AND 128, false) = false
    OR COALESCE(length(message->>'content') BETWEEN 1 AND 2000, false) = false
    OR jsonb_typeof(message->'fragments') IS DISTINCT FROM 'array'
  THEN RETURN false; END IF;
  IF jsonb_array_length(message->'fragments') > 100 THEN RETURN false; END IF;
  FOR fragment IN SELECT value FROM jsonb_array_elements(message->'fragments') LOOP
    IF (fragment->>'type' = 'text' AND jsonb_typeof(fragment->'text') = 'string')
      OR (fragment->>'type' = 'emote' AND jsonb_typeof(fragment->'text') = 'string'
        AND jsonb_typeof(fragment->'emote_id') = 'string')
    THEN CONTINUE; END IF;
    RETURN false;
  END LOOP;
  RETURN true;
END;
$$;
REVOKE ALL ON FUNCTION public.widget_pin_valid(JSONB, UUID) FROM PUBLIC, anon, authenticated;

CREATE FUNCTION public.resolve_account_widget(copied_chat_hint UUID DEFAULT NULL)
RETURNS SETOF public.widgets LANGUAGE plpgsql SECURITY DEFINER SET search_path = '' AS $$
DECLARE
  account_id UUID := auth.uid();
  chosen_id UUID;
  source_pin JSONB;
  greatest_revision BIGINT;
BEGIN
  IF account_id IS NULL THEN RAISE EXCEPTION 'Authentication required' USING ERRCODE = '42501'; END IF;
  -- Lock the account row so concurrent first visits cannot create two identities.
  PERFORM 1 FROM auth.users WHERE id = account_id FOR UPDATE;
  SELECT i.widget_id INTO chosen_id FROM public.widget_identities i WHERE i.user_id = account_id;
  IF chosen_id IS NULL THEN
    SELECT w.id INTO chosen_id FROM public.widgets w
      WHERE w.user_id = account_id AND w.id = copied_chat_hint;
    IF chosen_id IS NULL THEN
      SELECT w.id INTO chosen_id FROM public.widgets w WHERE w.user_id = account_id ORDER BY w.id LIMIT 1;
    END IF;
    IF chosen_id IS NULL THEN
      INSERT INTO public.widgets (user_id, auto_hide_seconds, layout_style)
        VALUES (account_id, 0, 'card') RETURNING id INTO chosen_id;
    END IF;
    INSERT INTO public.widget_identities (user_id, widget_id) VALUES (account_id, chosen_id);

    IF NOT EXISTS (
      SELECT 1 FROM public.widgets w WHERE w.id = chosen_id
        AND public.widget_pin_valid(w.pinned_message, w.id)
    ) THEN
      SELECT w.pinned_message, w.pin_revision INTO source_pin, greatest_revision
        FROM public.widgets w WHERE w.user_id = account_id AND w.id <> chosen_id
          AND public.widget_pin_valid(w.pinned_message, w.id)
        ORDER BY w.id LIMIT 1;
      IF source_pin IS NOT NULL THEN
        SELECT GREATEST(COALESCE(MAX(w.pin_revision), 0), greatest_revision)
          INTO greatest_revision FROM public.widgets w WHERE w.user_id = account_id;
        UPDATE public.widgets SET
          pinned_message = jsonb_set(source_pin, '{widget_id}', to_jsonb(chosen_id::text)),
          pin_revision = greatest_revision + 1
          WHERE id = chosen_id;
      END IF;
    END IF;
  END IF;
  RETURN QUERY SELECT w.* FROM public.widgets w WHERE w.id = chosen_id AND w.user_id = account_id;
END;
$$;
REVOKE ALL ON FUNCTION public.resolve_account_widget(UUID) FROM PUBLIC, anon;
GRANT EXECUTE ON FUNCTION public.resolve_account_widget(UUID) TO authenticated;

-- The caller can learn only the overlay fields for one UUID it already knows.
CREATE FUNCTION public.lookup_public_widget(known_id UUID)
RETURNS TABLE (id UUID, theme TEXT, font_size TEXT, background_color TEXT,
  auto_hide_seconds INTEGER, layout_style TEXT)
LANGUAGE sql STABLE SECURITY DEFINER SET search_path = '' AS $$
  SELECT canonical.id, canonical.theme, canonical.font_size, canonical.background_color,
    canonical.auto_hide_seconds, canonical.layout_style
  FROM public.widgets addressed
  LEFT JOIN public.widget_identities identity ON identity.user_id = addressed.user_id
  JOIN public.widgets canonical ON canonical.id = COALESCE(identity.widget_id, addressed.id)
  WHERE addressed.id = known_id;
$$;
REVOKE ALL ON FUNCTION public.lookup_public_widget(UUID) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION public.lookup_public_widget(UUID) TO anon, authenticated;
