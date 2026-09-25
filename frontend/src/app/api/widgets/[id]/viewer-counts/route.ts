import { z } from "zod";

const uuidSchema = z.string().uuid();

export async function GET(_request: Request, context: RouteContext<"/api/widgets/[id]/viewer-counts">) {
  const { id } = await context.params;
  if (!uuidSchema.safeParse(id).success) return Response.json({ error: "Invalid widget ID" }, { status: 400 });
  const backend = process.env.BACKEND_INTERNAL_URL ?? "http://127.0.0.1:8080";
  try {
    const response = await fetch(`${backend}/public/widgets/${id}/viewer-counts`, { cache: "no-store", signal: AbortSignal.timeout(10_000) });
    if (!response.ok) return Response.json({ error: "Viewer count unavailable" }, { status: response.status === 404 ? 404 : 502 });
    return Response.json(await response.json(), { headers: { "Cache-Control": "no-store" } });
  } catch {
    return Response.json({ error: "Viewer count service unavailable" }, { status: 502 });
  }
}
