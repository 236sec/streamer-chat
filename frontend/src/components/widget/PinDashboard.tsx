"use client";

import { useEffect, useRef, useState } from "react";
import { env } from "@/env";
import { chatMessage, pinEvent, type ChatMessage, type PinEvent } from "@/lib/pin";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";
import { useOrigin } from "@/lib/use-origin";

interface PinDashboardProps {
  widgetId: string;
  onPinChange?: (widgetId: string, message: ChatMessage | null) => void;
}

export function PinDashboard({ widgetId, onPinChange }: PinDashboardProps) {
  const origin = useOrigin();
  const [feed, setFeed] = useState<ChatMessage[]>([]);
  const [active, setActive] = useState<ChatMessage | null>(null);
  const [status, setStatus] = useState("Connecting to live chat…");
  const [error, setError] = useState("");
  const [platformErrors, setPlatformErrors] = useState<Record<string, string>>({});
  const [pending, setPending] = useState(false);
  const [copied, setCopied] = useState(false);
  const revision = useRef(-1);
  const highlightUrl = origin ? `${origin}/widget/${widgetId}/highlight` : "";

  useEffect(() => {
    if (!widgetId || widgetId === "loading...") return;
    let socket: WebSocket | null = null;
    let timer: ReturnType<typeof setTimeout> | undefined;
    let stopped = false;
    let attempt = 0;
    const connect = () => {
      if (stopped) return;
      socket = new WebSocket(`${env.NEXT_PUBLIC_WS_URL}/widget/${widgetId}`);
      socket.onopen = () => { attempt = 0; setStatus("Live chat connected"); };
      socket.onmessage = (event) => {
        try {
          const value: unknown = JSON.parse(event.data);
          if (typeof value === "object" && value !== null && "type" in value && value.type === "platform_error" &&
              "platform" in value && typeof value.platform === "string" &&
              "message" in value && typeof value.message === "string") {
            setPlatformErrors((previous) => ({ ...previous, [value.platform as string]: value.message as string }));
            return;
          }
          const message = chatMessage.safeParse(value);
          if (message.success && message.data.widget_id === widgetId) {
            setFeed((previous) => [...previous, message.data].slice(-50));
            return;
          }
          const pin = pinEvent.safeParse(value);
          if (pin.success && pin.data.revision > revision.current) {
            revision.current = pin.data.revision;
            setActive(pin.data.message);
            onPinChange?.(widgetId, pin.data.message);
          }
        } catch { /* Ignore malformed public frames. */ }
      };
      socket.onclose = () => {
        if (stopped) return;
        setStatus("Disconnected. Reconnecting…");
        timer = setTimeout(connect, Math.min(1000 * 2 ** attempt++, 30000));
      };
      socket.onerror = () => setStatus("Connection error. Reconnecting…");
    };
    connect();
    return () => { stopped = true; clearTimeout(timer); socket?.close(); };
  }, [widgetId, onPinChange]);

  const mutate = async (message: ChatMessage | null) => {
    setError("");
    setPending(true);
    try {
      const response = await fetch("/api/widgets/pin", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(message ? { action: "pin", widgetId, message } : { action: "unpin", widgetId }),
      });
      const value: unknown = await response.json();
      if (!response.ok) {
        const detail = typeof value === "object" && value !== null && "error" in value && typeof value.error === "string" ? value.error : "Pin change failed";
        throw new Error(detail);
      }
      const event = pinEvent.parse(value) as PinEvent;
      if (event.revision > revision.current) {
        revision.current = event.revision;
        setActive(event.message);
        onPinChange?.(widgetId, event.message);
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Pin change failed");
    } finally { setPending(false); }
  };

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(highlightUrl);
      setCopied(true);
      setError("");
    } catch { setError("Could not copy the highlight URL."); }
  };

  return (
    <section className="rounded-lg border border-border bg-card p-6 space-y-5">
      <div>
        <h2 className="text-xl font-semibold text-foreground">Message Highlight</h2>
        <p className="text-sm text-muted-foreground">Select a live message to feature in a separate OBS browser source.</p>
      </div>
      <div className="rounded-md border border-border bg-background p-4">
        <p className="text-xs font-semibold uppercase text-muted-foreground">Current pin</p>
        {active ? <p className="mt-2 break-words"><span className="font-semibold">{active.author}:</span> {active.content}</p>
          : <p className="mt-2 text-sm text-muted-foreground">No message pinned</p>}
        {active && <Button className="mt-3" variant="secondary" disabled={pending} onClick={() => mutate(null)}>Unpin message</Button>}
      </div>
      <div>
        <p className="text-sm font-medium">Live feed</p>
        <p className="text-xs text-muted-foreground" role="status">{status}</p>
        {feed.length === 0 ? <p className="mt-3 text-sm text-muted-foreground">Waiting for chat messages…</p> :
          <div className="mt-3 max-h-80 space-y-2 overflow-y-auto" aria-label="Recent chat messages">
            {feed.map((message, index) => <div key={`${message.id}-${index}`} className={cn("flex items-start justify-between gap-3 rounded-md border p-3", active?.id === message.id ? "border-primary bg-primary/10" : "border-border bg-background")}>
              <div className="min-w-0 break-words text-sm"><span className="mr-2 text-xs uppercase text-muted-foreground">{message.platform}</span><strong>{message.author}</strong>: {message.content}</div>
              <Button size="sm" variant={active?.id === message.id ? "secondary" : "default"} disabled={pending || active?.id === message.id} onClick={() => mutate(message)}>{active?.id === message.id ? "Pinned" : "Pin"}</Button>
            </div>)}
          </div>}
      </div>
      <div className="border-t border-border pt-4 space-y-2">
        <h3 className="font-semibold">Highlight browser source</h3>
        <p className="text-sm text-muted-foreground">Add this URL as a separate Browser Source in OBS. Start at 800 × 250 px and adjust to fit your scene.</p>
        <div className="flex gap-2"><Input readOnly aria-label="Highlight URL" value={highlightUrl} className="font-mono text-xs" /><Button variant="secondary" disabled={!highlightUrl} onClick={copy}>{copied ? "Copied" : "Copy"}</Button></div>
      </div>
      {Object.entries(platformErrors).map(([platform, message]) => <p key={platform} role="alert" className="text-sm text-destructive">{platform}: {message}</p>)}
      {error && <p role="alert" className="text-sm text-destructive">{error}</p>}
    </section>
  );
}
