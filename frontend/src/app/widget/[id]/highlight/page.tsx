import { notFound } from "next/navigation";
import { createClient } from "@/lib/supabase/server";
import { HighlightClient } from "@/components/widget/HighlightClient";
import { z } from "zod";

export default async function HighlightPage({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  if (!z.string().uuid().safeParse(id).success) notFound();
  const supabase = await createClient();
  const { data, error } = await supabase.rpc("lookup_public_widget", { known_id: id });
  const result = z.array(z.object({ id: z.string().uuid() })).safeParse(data);
  if (error || !result.success || result.data.length !== 1) notFound();
  return <HighlightClient widgetId={result.data[0].id} />;
}
