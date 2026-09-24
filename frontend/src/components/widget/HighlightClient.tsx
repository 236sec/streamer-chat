"use client";

import { useEffect, useRef, useState } from "react";
import { env } from "@/env";
import { pinEvent, type ChatMessage } from "@/lib/pin";
import { cn } from "@/lib/utils";

export function HighlightClient({ widgetId }: { widgetId: string }) {
  const [visible, setVisible] = useState<ChatMessage | null>(null);
  const [leaving, setLeaving] = useState(false);
  const [animationKey, setAnimationKey] = useState(0);
  const revision = useRef(-1);
  const transition = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  useEffect(() => {
    let socket: WebSocket | null = null;
    let reconnect: ReturnType<typeof setTimeout> | undefined;
    let stopped = false;
    let attempts = 0;
    const connect = () => {
      if (stopped) return;
      socket = new WebSocket(`${env.NEXT_PUBLIC_WS_URL}/widget/${widgetId}`);
      socket.onopen = () => { attempts = 0; };
      socket.onmessage = (event) => {
        try {
          const parsed = pinEvent.safeParse(JSON.parse(event.data));
          if (!parsed.success || parsed.data.revision <= revision.current) return;
          revision.current = parsed.data.revision;
          const next = parsed.data.message;
          clearTimeout(transition.current);
          setLeaving(true);
          transition.current = setTimeout(() => {
            setVisible(next);
            setAnimationKey(parsed.data.revision);
            setLeaving(false);
          }, window.matchMedia("(prefers-reduced-motion: reduce)").matches ? 0 : 220);
        } catch { /* Ignore other widget frames. */ }
      };
      socket.onclose = () => {
        if (stopped) return;
        reconnect = setTimeout(connect, Math.min(1000 * 2 ** attempts++, 30000));
      };
    };
    connect();
    return () => { stopped = true; clearTimeout(reconnect); clearTimeout(transition.current); socket?.close(); };
  }, [widgetId]);

  return <main className="highlight-source fixed inset-0 flex items-center justify-center overflow-hidden bg-transparent p-3">
    {visible && <article key={`${animationKey}-${visible.id}`} className={cn("highlight-card w-full max-w-[800px] rounded-lg border border-border bg-card p-5 text-card-foreground shadow-2xl", leaving && "highlight-card-exit")}>
      <div className="flex items-start gap-4">
        {visible.avatar_url && (
          // eslint-disable-next-line @next/next/no-img-element
          <img src={visible.avatar_url} alt="" className="h-12 w-12 shrink-0 rounded-full object-cover" />
        )}
        <div className="min-w-0">
          <div className="mb-2 flex flex-wrap items-center gap-2"><span className="rounded-sm bg-primary px-2 py-0.5 text-xs font-semibold uppercase text-primary-foreground">{visible.platform}</span><strong className="text-lg">{visible.author}</strong></div>
          <p className="break-words text-xl leading-snug">{visible.content}</p>
        </div>
      </div>
    </article>}
  </main>;
}
