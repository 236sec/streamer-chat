"use client";

import { useEffect, useState, useRef, useCallback } from "react";
import { env } from "@/env";
import { cn } from "@/lib/utils";

export const LAYOUT_STYLES = {
  card: "bg-card text-card-foreground border border-border/50 rounded shadow-sm px-4 py-2",
  bubble: "bg-secondary text-secondary-foreground rounded-2xl shadow px-3 py-1.5",
  clean: "bg-transparent text-foreground drop-shadow-[0_2px_4px_rgba(0,0,0,0.8)] px-2 py-1",
} as const;

export type LayoutStyle = keyof typeof LAYOUT_STYLES;

type MessageFragment =
  | { type: "text"; text: string }
  | { type: "emote"; text: string; emote_id: string };

interface Message {
  id: string;
  type: string;
  author: string;
  content: string;
  color?: string;
  platform?: string;
  fragments?: MessageFragment[];
}

interface DisplayMessage extends Message {
  createdAt: number;
}

interface MessageItemProps {
  msg: DisplayMessage;
  layoutStyle: LayoutStyle;
  autoHideSeconds?: number;
  onRemove: (id: string) => void;
}

function MessageItem({ msg, layoutStyle, autoHideSeconds, onRemove }: MessageItemProps) {
  const [isFading, setIsFading] = useState(false);

  useEffect(() => {
    if (!autoHideSeconds || autoHideSeconds <= 0) {
      return;
    }

    const elapsed = Date.now() - msg.createdAt;
    const remainingTime = Math.max(0, autoHideSeconds * 1000 - elapsed);

    const fadeTimer = setTimeout(() => {
      setIsFading(true);
    }, remainingTime);

    const removeTimer = setTimeout(() => {
      onRemove(msg.id);
    }, remainingTime + 700);

    return () => {
      clearTimeout(fadeTimer);
      clearTimeout(removeTimer);
    };
  }, [autoHideSeconds, msg.createdAt, msg.id, onRemove]);

  return (
    <div
      className={cn(
        "break-words transition-all duration-700 ease-out transform-gpu",
        isFading
          ? "opacity-0 -translate-y-2 scale-95 pointer-events-none"
          : "opacity-100 translate-y-0 scale-100",
        LAYOUT_STYLES[layoutStyle]
      )}
    >
      {msg.platform && (
        <span
          className="inline-block px-1.5 py-0.5 mr-2 text-xs font-semibold uppercase rounded align-middle"
          style={{
            backgroundColor: `var(--${msg.platform})`,
            color: msg.platform === "kick" ? "black" : "white",
          }}
        >
          {msg.platform}
        </span>
      )}
      <span style={{ color: msg.color || "inherit" }} className="font-bold mr-2 align-middle">
        {msg.author}:
      </span>
      <span>
        {msg.fragments && msg.fragments.length > 0 ? (
          msg.fragments.map((frag, i) => {
            if (frag.type === "emote") {
              return (
                // eslint-disable-next-line @next/next/no-img-element
                <img
                  key={i}
                  src={`https://static-cdn.jtvnw.net/emoticons/v2/${frag.emote_id}/default/dark/1.0`}
                  alt={frag.text}
                  className="inline-block align-middle mx-1"
                />
              );
            }
            return <span key={i}>{frag.text}</span>;
          })
        ) : (
          msg.content
        )}
      </span>
    </div>
  );
}

export interface WidgetClientProps {
  widgetId: string;
  theme: string;
  fontSize: string;
  backgroundColor: string;
  autoHideSeconds?: number;
  layoutStyle?: string;
  mock?: boolean;
}

export function WidgetClient({
  widgetId,
  theme,
  fontSize,
  backgroundColor,
  autoHideSeconds = 0,
  layoutStyle = "card",
  mock,
}: WidgetClientProps) {
  const [messages, setMessages] = useState<DisplayMessage[]>([]);
  const [platformErrors, setPlatformErrors] = useState<Map<string, string>>(new Map());
  const wsRef = useRef<WebSocket | null>(null);

  const activeLayoutStyle: LayoutStyle =
    layoutStyle && layoutStyle in LAYOUT_STYLES
      ? (layoutStyle as LayoutStyle)
      : "card";

  const handleRemoveMessage = useCallback((id: string) => {
    setMessages((prev) => prev.filter((m) => m.id !== id));
  }, []);

  useEffect(() => {
    let ws: WebSocket;
    let reconnectTimer: NodeJS.Timeout;
    let mockInterval: NodeJS.Timeout;
    let reconnectAttempts = 0;
    let isMounted = true;
    const maxReconnectDelay = 30000;

    if (mock) {
      // Frontend mock logic
      let count = 0;
      mockInterval = setInterval(() => {
        count++;
        setMessages((prev) => {
          const newMessages: DisplayMessage[] = [
            ...prev,
            {
              id: `mock-${Date.now()}-${count}`,
              type: "chat_message",
              author: count % 2 === 0 ? "StreamFan" : "CoolGamer99",
              platform: count % 3 === 0 ? "youtube" : count % 2 === 0 ? "kick" : "twitch",
              content: `This is mock message #${count} to preview your widget styling!`,
              color: count % 2 === 0 ? "#8a2be2" : "#ff4500",
              fragments: [
                { type: "text", text: `This is mock message #${count} to preview your widget styling!` },
              ],
              createdAt: Date.now(),
            },
          ];
          if (newMessages.length > 50) {
            return newMessages.slice(newMessages.length - 50);
          }
          return newMessages;
        });
      }, 2000);
      return () => {
        clearInterval(mockInterval);
      };
    }

    const connect = () => {
      if (!isMounted) return;

      // Connect to the WebSocket URL
      const wsUrl = `${env.NEXT_PUBLIC_WS_URL}/widget/${widgetId}`;
      console.log(`[WebSocket] Attempting to connect to ${wsUrl} (Attempt ${reconnectAttempts + 1})`);
      ws = new WebSocket(wsUrl);
      wsRef.current = ws;

      ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);

          if (data.type === "platform_error") {
            setPlatformErrors((prev) => {
              const next = new Map(prev);
              next.set(data.platform, data.message);
              return next;
            });
            return;
          }

          setMessages((prev) => {
            const newMessages: DisplayMessage[] = [
              ...prev,
              {
                ...data,
                id: data.id || `msg-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
                createdAt: Date.now(),
              },
            ];
            if (newMessages.length > 50) {
              return newMessages.slice(newMessages.length - 50);
            }
            return newMessages;
          });
        } catch (error) {
          console.error("[WebSocket] Failed to parse message", error, "Raw data:", event.data);
        }
      };

      ws.onopen = () => {
        console.log(`[WebSocket] Connected successfully to ${wsUrl}`);
        reconnectAttempts = 0;
      };

      ws.onclose = (event) => {
        if (!isMounted) return;
        const delay = Math.min(1000 * Math.pow(2, reconnectAttempts), maxReconnectDelay);
        console.warn(
          `[WebSocket] Connection closed. Code: ${event.code}, Reason: ${event.reason || "None"}, Clean: ${event.wasClean}. Reconnecting in ${delay}ms...`
        );
        reconnectAttempts++;
        reconnectTimer = setTimeout(connect, delay);
      };

      ws.onerror = (error) => {
        if (!isMounted) return; // Ignore errors from Strict Mode unmount aborts
        console.error(`[WebSocket] Error occurred. ReadyState: ${ws.readyState}`);
        if (error instanceof ErrorEvent) {
          console.error("[WebSocket] Error message:", error.message);
        }
      };
    };

    connect();

    return () => {
      isMounted = false;
      clearTimeout(reconnectTimer);
      if (wsRef.current) {
        wsRef.current.close();
      }
    };
  }, [widgetId, mock]);

  return (
    <div
      className={cn("absolute inset-0 overflow-hidden flex flex-col justify-end p-4", theme === "dark" && "dark")}
      style={{ backgroundColor: backgroundColor || "transparent", fontSize: fontSize }}
    >
      {platformErrors.size > 0 && (
        <div className="mb-2 px-3 py-1.5 rounded bg-destructive/80 text-destructive-foreground text-xs">
          {Array.from(platformErrors.entries()).map(([platform]) => (
            <div key={platform} className="flex items-center gap-1">
              <span className="font-semibold uppercase">{platform}</span>
              <span>— Token expired. Reconnect in dashboard.</span>
            </div>
          ))}
        </div>
      )}
      <div className="flex flex-col gap-2">
        {messages.map((msg) => (
          <MessageItem
            key={msg.id}
            msg={msg}
            layoutStyle={activeLayoutStyle}
            autoHideSeconds={autoHideSeconds}
            onRemove={handleRemoveMessage}
          />
        ))}
      </div>
    </div>
  );
}
