import { notFound } from "next/navigation";
import { createClient } from "@/lib/supabase/server";
import { WidgetClient } from "@/components/widget/WidgetClient";

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
    <WidgetClient
      widgetId={widget.id}
      theme={widget.theme}
      fontSize={widget.font_size}
      backgroundColor={widget.background_color}
      autoHideSeconds={autoHideSeconds}
      layoutStyle={layoutStyle}
      mock={mock}
    />
  );
}
