"use client";

import { useEffect, useState, useRef } from "react";
import { env } from "@/env";
import { cn } from "@/lib/utils";

interface Message {
  id: string;
  author: string;
  content: string;
  color?: string;
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
    document.body.style.backgroundColor = backgroundColor || "transparent";
    // Also remove the default Next.js background class if it exists
    document.body.classList.remove("bg-background");
    
    return () => {
      document.body.style.backgroundColor = "";
      document.body.classList.add("bg-background");
    };
  }, [backgroundColor]);

  useEffect(() => {
    let ws: WebSocket;
    let reconnectTimer: NodeJS.Timeout;

    const connect = () => {
      // Connect to the WebSocket URL, e.g. ws://127.0.0.1:3000/ws/widget/:id
      const wsUrl = `${env.NEXT_PUBLIC_WS_URL}/widget/${widgetId}${mock ? '?mock=true' : ''}`;
      ws = new WebSocket(wsUrl);
      wsRef.current = ws;

      ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          setMessages((prev) => {
            const newMessages = [...prev, data];
            // Keep only the last 50 messages to avoid DOM overload
            if (newMessages.length > 50) {
              return newMessages.slice(newMessages.length - 50);
            }
            return newMessages;
          });
        } catch (error) {
          console.error("Failed to parse message", error);
        }
      };

      ws.onclose = () => {
        // Automatically reconnect after 2 seconds
        reconnectTimer = setTimeout(connect, 2000);
      };

      ws.onerror = (error) => {
        console.error("WebSocket error", error);
      };
    };

    connect();

    return () => {
      clearTimeout(reconnectTimer);
      if (wsRef.current) {
        wsRef.current.close();
      }
    };
  }, [widgetId, mock]);

  return (
    <div
      className={cn("w-full h-screen overflow-hidden flex flex-col justify-end p-4", theme === "dark" && "dark")}
      style={{ backgroundColor: backgroundColor, fontSize: fontSize }}
    >
      <div className="flex flex-col gap-2">
        {messages.map((msg) => (
          <div key={msg.id} className={cn("px-4 py-2 rounded shadow-sm break-words bg-card text-card-foreground")}>
            <span style={{ color: msg.color || "inherit" }} className="font-bold mr-2">
              {msg.author}:
            </span>
            <span>{msg.content}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
