import { createClient } from "@/lib/supabase/server";
import { pinCommand } from "@/lib/pin";

export async function POST(request: Request) {
  const body: unknown = await request.json().catch(() => null);
  const parsed = pinCommand.safeParse(body);
  if (!parsed.success || (parsed.data.action === "pin" && parsed.data.message.widget_id !== parsed.data.widgetId)) {
    return Response.json({ error: "Invalid pin command" }, { status: 400 });
  }

  const supabase = await createClient();
  const { data: { user }, error: authError } = await supabase.auth.getUser();
  if (authError || !user) return Response.json({ error: "Unauthorized" }, { status: 401 });

  const { data: widget, error: ownershipError } = await supabase
    .from("widgets").select("id").eq("id", parsed.data.widgetId).eq("user_id", user.id).maybeSingle();
  if (ownershipError) return Response.json({ error: "Unable to verify widget ownership" }, { status: 502 });
  if (!widget) return Response.json({ error: "Widget not found" }, { status: 404 });

  const secret = process.env.PIN_COMMAND_SECRET;
  if (!secret) return Response.json({ error: "Pin service is not configured" }, { status: 503 });
  const backend = process.env.BACKEND_INTERNAL_URL ?? "http://127.0.0.1:8080";
  try {
    const response = await fetch(`${backend}/internal/widgets/${widget.id}/pin`, {
      method: "POST",
      headers: { "content-type": "application/json", "x-pin-secret": secret },
      body: JSON.stringify(parsed.data.action === "pin"
        ? { action: "pin", message: parsed.data.message }
        : { action: "unpin" }),
      cache: "no-store",
    });
    if (!response.ok) return Response.json({ error: "Pin change failed" }, { status: 502 });
    return Response.json(await response.json());
  } catch {
    return Response.json({ error: "Pin service unavailable" }, { status: 502 });
  }
}
