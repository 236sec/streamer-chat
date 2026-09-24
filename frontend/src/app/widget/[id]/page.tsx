import { notFound } from "next/navigation";
import { createClient } from "@/lib/supabase/server";
import { WidgetClient } from "@/components/widget/WidgetClient";
import { parseWidgetRecord } from "@/lib/widget-selection";
import { z } from "zod";

interface PageProps {
  params: Promise<{
    id: string;
  }>;
  searchParams: Promise<{
    [key: string]: string | string[] | undefined;
  }>;
}

export default async function WidgetPage(props: PageProps) {
  const params = await props.params;
  const searchParams = await props.searchParams;
  const { id } = params;
  if (!z.string().uuid().safeParse(id).success) notFound();

  const supabase = await createClient();

  const { data, error } = await supabase.rpc("lookup_public_widget", { known_id: id });

  if (error || !Array.isArray(data) || data.length !== 1) notFound();
  const widget = parseWidgetRecord(data[0]);

  const mock = searchParams.mock === "true";
  const autoHideSeconds =
    typeof searchParams.auto_hide_seconds === "string"
      ? Number(searchParams.auto_hide_seconds)
      : widget.auto_hide_seconds !== undefined && widget.auto_hide_seconds !== null
      ? Number(widget.auto_hide_seconds)
      : 0;

  const layoutStyle =
    (typeof searchParams.layout_style === "string"
      ? searchParams.layout_style
      : undefined) ??
    widget.layout_style ??
    "card";

  return (
    <main className="widget-source fixed inset-0">
      <WidgetClient
        widgetId={widget.id}
        theme={widget.theme ?? "dark"}
        fontSize={widget.font_size ?? "16px"}
        backgroundColor={widget.background_color ?? "transparent"}
        autoHideSeconds={autoHideSeconds}
        layoutStyle={layoutStyle}
        mock={mock}
      />
    </main>
  );
}
