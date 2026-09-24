import { z } from "zod";

const fragment = z.discriminatedUnion("type", [
  z.object({ type: z.literal("text"), text: z.string() }),
  z.object({ type: z.literal("emote"), text: z.string(), emote_id: z.string() }),
]);

export const chatMessage = z.object({
  id: z.string().min(1).max(256),
  type: z.literal("chat_message"),
  widget_id: z.string().uuid(),
  platform: z.enum(["twitch", "youtube", "kick"]),
  author: z.string().min(1).max(128),
  avatar_url: z.string().nullable().optional(),
  color: z.string().nullable().optional(),
  badges: z.array(z.string()).nullable().optional(),
  content: z.string().min(1).max(2000),
  fragments: z.array(fragment).max(100),
});

export const pinCommand = z.discriminatedUnion("action", [
  z.object({ action: z.literal("pin"), widgetId: z.string().uuid(), message: chatMessage }),
  z.object({ action: z.literal("unpin"), widgetId: z.string().uuid() }),
]);

export const pinEvent = z.object({
  type: z.enum(["pin_state", "pin_message", "unpin_message"]),
  revision: z.number().int().nonnegative(),
  message: chatMessage.nullable(),
});

export type ChatMessage = z.infer<typeof chatMessage>;
export type PinEvent = z.infer<typeof pinEvent>;
