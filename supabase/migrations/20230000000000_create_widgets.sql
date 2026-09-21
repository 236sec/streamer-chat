CREATE TABLE widgets (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
  theme TEXT NOT NULL DEFAULT 'default',
  font_size TEXT NOT NULL DEFAULT '16px',
  background_color TEXT NOT NULL DEFAULT 'transparent'
);

ALTER TABLE widgets ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Public read access for widgets using id"
  ON widgets
  FOR SELECT
  USING (true);

CREATE POLICY "Restricted updates to authenticated owner"
  ON widgets
  FOR UPDATE
  USING (auth.uid() = user_id);

CREATE POLICY "Restricted inserts to authenticated owner"
  ON widgets
  FOR INSERT
  WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Restricted deletes to authenticated owner"
  ON widgets
  FOR DELETE
  USING (auth.uid() = user_id);
