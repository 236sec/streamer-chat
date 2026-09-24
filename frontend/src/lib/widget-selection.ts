import { z } from "zod";
import { createClient } from "@/lib/supabase/client";

const widgetRecord = z.object({
  id: z.string().uuid(),
  theme: z.string().nullable().optional(),
  font_size: z.string().nullable().optional(),
  background_color: z.string().nullable().optional(),
  auto_hide_seconds: z.number().nullable().optional(),
  layout_style: z.string().nullable().optional(),
  pinned_message: z.unknown().optional(),
});

export type WidgetRecord = z.infer<typeof widgetRecord>;

export function parseWidgetRecord(value: unknown): WidgetRecord {
  const parsed = widgetRecord.safeParse(value);
  if (!parsed.success) throw new Error("The widget record could not be read.");
  return parsed.data;
}

export async function resolveAccountWidget(): Promise<WidgetRecord> {
  const saved = typeof window === "undefined" ? null : window.localStorage.getItem("streamsync_obs_copied_widget");
  const copiedChatHint = z.string().uuid().safeParse(saved);
  const { data, error } = await createClient().rpc("resolve_account_widget", {
    copied_chat_hint: copiedChatHint.success ? copiedChatHint.data : null,
  });
  if (error) throw new Error(error.message || "Failed to load widgets.");
  const parsed = z.array(widgetRecord).safeParse(data);
  if (!parsed.success || parsed.data.length !== 1) throw new Error("The account widget could not be read.");
  return parsed.data[0];
}
