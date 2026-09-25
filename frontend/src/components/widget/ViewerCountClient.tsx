"use client";

import { useEffect, useState } from "react";
import { z } from "zod";

const responseSchema = z.object({
  counts: z.object({ twitch: z.number().int().nonnegative(), youtube: z.number().int().nonnegative(), kick: z.number().int().nonnegative() }),
  refresh_seconds: z.number().int().min(30).max(300),
});
type Counts = z.infer<typeof responseSchema>["counts"];
const zeroCounts: Counts = { twitch: 0, youtube: 0, kick: 0 };

export function ViewerCountClient({ widgetId }: { widgetId: string }) {
  const [counts, setCounts] = useState(zeroCounts);
  useEffect(() => {
    let active = true;
    let timer: ReturnType<typeof setTimeout> | undefined;
    let request: AbortController | undefined;
    let interval = 60;
    const schedule = () => { timer = setTimeout(() => { void load(); }, interval * 1000); };
    const load = async () => {
      if (!active) return;
      request?.abort();
      const controller = new AbortController();
      request = controller;
      try {
        const response = await fetch(`/api/widgets/${widgetId}/viewer-counts`, { cache: "no-store", signal: controller.signal });
        if (!response.ok) throw new Error("Viewer count request failed");
        const parsed = responseSchema.safeParse(await response.json());
        if (!parsed.success) throw new Error("Invalid viewer count response");
        if (!active) return;
        interval = parsed.data.refresh_seconds;
        setCounts(parsed.data.counts);
      } catch {
        if (!active || controller.signal.aborted) return;
        setCounts(zeroCounts);
      } finally {
        if (active && !controller.signal.aborted) schedule();
      }
    };
    const reconnect = () => { if (timer) clearTimeout(timer); void load(); };
    window.addEventListener("online", reconnect);
    void load();
    return () => {
      active = false;
      if (timer) clearTimeout(timer);
      request?.abort();
      window.removeEventListener("online", reconnect);
    };
  }, [widgetId]);

  return <div className="flex flex-col gap-2 bg-transparent text-foreground" aria-label="Live viewers by platform">
    {([ ["Twitch", counts.twitch], ["YouTube", counts.youtube], ["Kick", counts.kick] ] as const).map(([platform, count]) =>
      <div key={platform} className="flex min-w-40 items-center justify-between gap-8 rounded-md border border-border bg-card/80 px-4 py-3">
        <span className="text-sm font-medium">{platform}</span><span className="font-mono text-lg tabular-nums">{count}</span>
      </div>
    )}
  </div>;
}
