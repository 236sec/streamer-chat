import { notFound } from "next/navigation";
import { createClient } from "@/lib/supabase/server";
import { HighlightClient } from "@/components/widget/HighlightClient";

export default async function HighlightPage({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  const supabase = await createClient();
  const { data: widget } = await supabase.from("widgets").select("id").eq("id", id).maybeSingle();
  if (!widget) notFound();
  return <HighlightClient widgetId={widget.id} />;
}
