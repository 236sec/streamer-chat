import { notFound } from "next/navigation";
import { z } from "zod";
import { ViewerCountClient } from "@/components/widget/ViewerCountClient";

export default async function ViewerCountPage({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  if (!z.string().uuid().safeParse(id).success) notFound();
  return <main className="widget-source fixed inset-0 bg-transparent p-2"><ViewerCountClient widgetId={id} /></main>;
}
