import { z } from "zod";
import { chatMessage } from "@/lib/pin";
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

export async function listOwnedWidgets(userId: string): Promise<WidgetRecord[]> {
  const { data, error } = await createClient()
    .from("widgets")
    .select("*")
    .eq("user_id", userId);
  if (error) throw new Error(error.message || "Failed to load widgets.");
  const parsed = z.array(widgetRecord).safeParse(data);
  if (!parsed.success) throw new Error("The widget list could not be read.");
  return parsed.data;
}

export function selectedWidgetKey(userId: string) {
  return `streamsync_selected_widget:${userId}`;
}

export function resolveSelectedWidget(widgets: WidgetRecord[], userId: string): WidgetRecord | null {
  if (widgets.length === 1) return widgets[0];
  if (widgets.length === 0) return null;
  const savedId = window.localStorage.getItem(selectedWidgetKey(userId));
  return widgets.find((widget) => widget.id === savedId) ?? null;
}

export function pinPreview(widget: WidgetRecord): string | null {
  const parsed = chatMessage.safeParse(widget.pinned_message);
  if (!parsed.success || parsed.data.widget_id !== widget.id) return null;
  const preview = `${parsed.data.author}: ${parsed.data.content}`;
  return preview.length > 160 ? `${preview.slice(0, 160)}…` : preview;
}
