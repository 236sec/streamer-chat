"use client";

import { useEffect, useState, useRef } from "react";
import { env } from "@/env";
import { cn } from "@/lib/utils";

type MessageFragment =
  | { type: "text"; text: string }
  | { type: "emote"; text: string; emote_id: string };

interface Message {
  id: string;
  type: string;
  author: string;
  content: string;
  color?: string;
  fragments?: MessageFragment[];
}

interface WidgetClientProps {
  widgetId: string;
  theme: string;
  fontSize: string;
  backgroundColor: string;
  mock?: boolean;
}

export function WidgetClient({ widgetId, theme, fontSize, backgroundColor, mock }: WidgetClientProps) {
  const [messages, setMessages] = useState<Message[]>([]);
  const wsRef = useRef<WebSocket | null>(null);

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
          const newMessages = [...prev, {
            id: `mock-${Date.now()}-${count}`,
            type: "chat_message",
            author: count % 2 === 0 ? "StreamFan" : "CoolGamer99",
            content: `This is mock message #${count} to preview your widget styling!`,
            color: count % 2 === 0 ? "#8a2be2" : "#ff4500",
            fragments: [
              { type: "text", text: `This is mock message #${count} to preview your widget styling!` }
            ]
          } as Message];
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
          setMessages((prev) => {
            const newMessages = [...prev, data];
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
        console.warn(`[WebSocket] Connection closed. Code: ${event.code}, Reason: ${event.reason || 'None'}, Clean: ${event.wasClean}. Reconnecting in ${delay}ms...`);
        reconnectAttempts++;
        reconnectTimer = setTimeout(connect, delay);
      };

      ws.onerror = (error) => {
        if (!isMounted) return; // Ignore errors from Strict Mode unmount aborts
        console.error(`[WebSocket] Error occurred. ReadyState: ${ws.readyState}`);
        // WebSocket error events don't contain much info due to security reasons, 
        // but we log what we can.
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
      <div className="flex flex-col gap-2">
        {messages.map((msg) => (
          <div key={msg.id} className={cn("px-4 py-2 rounded shadow-sm break-words bg-card text-card-foreground")}>
            <span style={{ color: msg.color || "inherit" }} className="font-bold mr-2">
              {msg.author}:
            </span>
            <span>
              {msg.fragments && msg.fragments.length > 0 ? (
                msg.fragments.map((frag, i) => {
                  if (frag.type === "emote") {
                    return (
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
        ))}
      </div>
    </div>
  );
}
