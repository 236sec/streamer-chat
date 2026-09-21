import { notFound } from "next/navigation";
import { createClient } from "@/lib/supabase/server";
import { WidgetClient } from "@/components/widget/WidgetClient";

interface PageProps {
  params: Promise<{
    id: string;
  }>;
}

export default async function WidgetPage(props: PageProps) {
  const params = await props.params;
  const { id } = params;
  
  const supabase = await createClient();

  const { data: widget, error } = await supabase
    .from("widgets")
    .select("*")
    .eq("id", id)
    .single();

  if (error || !widget) {
    console.error("Widget fetch error:", error);
    notFound();
  }

  return (
    <WidgetClient
      widgetId={widget.id}
      theme={widget.theme}
      fontSize={widget.font_size}
      backgroundColor={widget.background_color}
    />
  );
}
