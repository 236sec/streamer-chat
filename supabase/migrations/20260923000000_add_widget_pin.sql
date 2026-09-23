ALTER TABLE widgets
  ADD COLUMN pinned_message JSONB,
  ADD COLUMN pin_revision BIGINT NOT NULL DEFAULT 0;

ALTER TABLE widgets
  ADD CONSTRAINT pin_revision_nonnegative CHECK (pin_revision >= 0);
