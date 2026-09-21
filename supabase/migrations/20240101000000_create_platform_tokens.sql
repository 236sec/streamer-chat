CREATE TABLE platform_tokens (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
  platform TEXT NOT NULL,
  encrypted_token TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE(user_id, platform)
);

ALTER TABLE platform_tokens ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Restricted select to authenticated owner"
  ON platform_tokens
  FOR SELECT
  USING (auth.uid() = user_id);

CREATE POLICY "Restricted insert to authenticated owner"
  ON platform_tokens
  FOR INSERT
  WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Restricted update to authenticated owner"
  ON platform_tokens
  FOR UPDATE
  USING (auth.uid() = user_id);

CREATE POLICY "Restricted delete to authenticated owner"
  ON platform_tokens
  FOR DELETE
  USING (auth.uid() = user_id);
