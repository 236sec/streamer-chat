"use client";

import { useState, useEffect } from "react";
import { WidgetClient } from "@/components/widget/WidgetClient";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { createClient } from "@/lib/supabase/client";
import { useToast } from "@/components/ui/toast";
import { ObsSetupGuideModal } from "@/components/obs/ObsSetupGuideModal";
import { useOrigin } from "@/lib/use-origin";
import { Copy, Check, HelpCircle, Save } from "lucide-react";
import { cn } from "@/lib/utils";

interface WidgetRecord {
  id: string;
  theme?: string;
  font_size?: string;
  background_color?: string;
  auto_hide_seconds?: number;
  layout_style?: string;
}

export default function WidgetSettingsPage() {
  const [widgetId, setWidgetId] = useState("loading...");
  const [theme, setTheme] = useState("dark");
  const [fontSize, setFontSize] = useState("16px");
  const [backgroundColor, setBackgroundColor] = useState("transparent");
  const [autoHideSeconds, setAutoHideSeconds] = useState<number>(0);
  const [layoutStyle, setLayoutStyle] = useState<string>("card");
  const [width, setWidth] = useState("400");
  const [height, setHeight] = useState("600");
  const origin = useOrigin();
  const [isSaving, setIsSaving] = useState(false);
  const [copied, setCopied] = useState(false);
  const [isGuideOpen, setIsGuideOpen] = useState(false);

  const { success, error: toastError } = useToast();

  useEffect(() => {
    const fetchWidget = async () => {
      const supabase = createClient();
      const {
        data: { user },
      } = await supabase.auth.getUser();
      if (!user) return;

      const { data, error } = await supabase
        .from("widgets")
        .select("*")
        .eq("user_id", user.id)
        .single();

      if (data) {
        setWidgetId(data.id);
        if (data.theme) setTheme(data.theme);
        if (data.font_size) setFontSize(data.font_size);
        if (data.background_color) setBackgroundColor(data.background_color);
        if (data.auto_hide_seconds !== undefined && data.auto_hide_seconds !== null) {
          setAutoHideSeconds(Number(data.auto_hide_seconds));
        }
        if (data.layout_style) {
          setLayoutStyle(data.layout_style);
        }
      } else if (error && error.code === "PGRST116") {
        // Record not found, create new widget
        let newWidgetData: WidgetRecord | null = null;
        const { data: newWidget, error: insertError } = await supabase
          .from("widgets")
          .insert({
            user_id: user.id,
            auto_hide_seconds: 0,
            layout_style: "card",
          })
          .select("*")
          .single();

        if (newWidget) {
          newWidgetData = newWidget;
        } else if (insertError) {
          const fallback = await supabase
            .from("widgets")
            .insert({ user_id: user.id })
            .select("*")
            .single();
          newWidgetData = fallback.data;
          if (fallback.error) {
            toastError(fallback.error.message || "Failed to initialize widget.", "Widget Setup Failed");
          }
        }

        if (newWidgetData) {
          setWidgetId(newWidgetData.id);
          if (newWidgetData.theme) setTheme(newWidgetData.theme);
          if (newWidgetData.font_size) setFontSize(newWidgetData.font_size);
          if (newWidgetData.background_color) setBackgroundColor(newWidgetData.background_color);
          if (newWidgetData.auto_hide_seconds !== undefined && newWidgetData.auto_hide_seconds !== null) {
            setAutoHideSeconds(Number(newWidgetData.auto_hide_seconds));
          }
          if (newWidgetData.layout_style) {
            setLayoutStyle(newWidgetData.layout_style);
          }
        }
      } else if (error) {
        toastError(error.message || "Failed to load widget settings.", "Loading Failed");
      }
    };
    fetchWidget();
  }, [toastError]);

  const queryParams = new URLSearchParams();
  if (autoHideSeconds > 0) {
    queryParams.set("auto_hide_seconds", String(autoHideSeconds));
  }
  if (layoutStyle && layoutStyle !== "card") {
    queryParams.set("layout_style", layoutStyle);
  }
  const queryString = queryParams.toString();
  const widgetUrl =
    origin && widgetId !== "loading..."
      ? `${origin}/widget/${widgetId}${queryString ? `?${queryString}` : ""}`
      : "";

  const handleSave = async () => {
    if (!widgetId || widgetId === "loading...") return;
    setIsSaving(true);
    try {
      const supabase = createClient();
      const updatePayload: Record<string, unknown> = {
        theme,
        font_size: fontSize,
        background_color: backgroundColor,
        auto_hide_seconds: autoHideSeconds,
        layout_style: layoutStyle,
      };

      let { error } = await supabase
        .from("widgets")
        .update(updatePayload)
        .eq("id", widgetId);

      let usedFallback = false;
      if (error && (error.message.includes("auto_hide_seconds") || error.message.includes("layout_style"))) {
        const fallbackRes = await supabase
          .from("widgets")
          .update({
            theme,
            font_size: fontSize,
            background_color: backgroundColor,
          })
          .eq("id", widgetId);
        error = fallbackRes.error;
        usedFallback = true;
      }

      if (error) {
        toastError(error.message || "Failed to save widget settings.", "Save Failed");
      } else if (usedFallback) {
        success(
          "Saved! Settings are encoded in your Widget URL. Run migration in Supabase SQL editor to store in DB.",
          "Settings Saved"
        );
      } else {
        success("Widget appearance saved successfully.", "Changes Saved");
      }
    } catch {
      toastError("An unexpected error occurred while saving.", "Save Failed");
    } finally {
      setIsSaving(false);
    }
  };

  const handleCopy = async () => {
    if (!widgetUrl) return;
    try {
      await navigator.clipboard.writeText(widgetUrl);
      setCopied(true);
      success("Widget URL copied to clipboard!", "Copied");
      setTimeout(() => setCopied(false), 2000);
    } catch {
      toastError("Failed to copy URL to clipboard.");
    }
  };

  return (
    <div className="flex flex-col gap-6 h-full">
      <div className="flex flex-wrap items-center justify-between gap-4">
        <div>
          <h1 className="text-3xl font-heading font-bold text-foreground">
            Widget Settings
          </h1>
          <p className="text-muted-foreground text-sm mt-1">
            Configure how your chat overlay looks and functions in OBS.
          </p>
        </div>
        <Button
          variant="secondary"
          onClick={() => setIsGuideOpen(true)}
          className="flex items-center gap-2"
        >
          <HelpCircle className="h-4 w-4" />
          OBS Setup Guide
        </Button>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 flex-1 min-h-0">
        {/* Settings Panel */}
        <div className="p-6 bg-card rounded-lg border border-border flex flex-col gap-6 lg:col-span-1 overflow-y-auto max-h-[calc(100vh-8rem)]">
          <div>
            <h2 className="text-xl font-semibold mb-2">Appearance</h2>
            <p className="text-muted-foreground text-sm mb-4">
              Customize font size, theme, layout style, and auto-hide timing.
            </p>
          </div>

          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="theme">Theme</Label>
              <select
                id="theme"
                value={theme}
                onChange={(e) => setTheme(e.target.value)}
                className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
              >
                <option value="dark">Dark</option>
                <option value="light">Light</option>
              </select>
            </div>

            <div className="space-y-2">
              <Label htmlFor="layoutStyle">Layout Preset</Label>
              <div className="grid grid-cols-3 gap-2">
                {[
                  { id: "card", label: "Card" },
                  { id: "bubble", label: "Bubble" },
                  { id: "clean", label: "Clean" },
                ].map((preset) => (
                  <button
                    key={preset.id}
                    type="button"
                    onClick={() => setLayoutStyle(preset.id)}
                    className={cn(
                      "px-3 py-2 text-sm font-medium rounded-md border transition-colors",
                      layoutStyle === preset.id
                        ? "border-primary bg-primary/10 text-primary"
                        : "border-border bg-background hover:bg-secondary text-muted-foreground hover:text-foreground"
                    )}
                  >
                    {preset.label}
                  </button>
                ))}
              </div>
            </div>

            <div className="space-y-2">
              <Label htmlFor="autoHide">Auto-Hide Duration</Label>
              <select
                id="autoHide"
                value={autoHideSeconds}
                onChange={(e) => setAutoHideSeconds(Number(e.target.value))}
                className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
              >
                <option value={0}>Disabled (Permanent)</option>
                <option value={5}>5 seconds</option>
                <option value={10}>10 seconds</option>
                <option value={15}>15 seconds</option>
                <option value={30}>30 seconds</option>
                <option value={60}>60 seconds</option>
              </select>
              <p className="text-xs text-muted-foreground">
                Messages smoothly fade out after this duration.
              </p>
            </div>

            <div className="space-y-2">
              <Label htmlFor="fontSize">Font Size</Label>
              <Input
                id="fontSize"
                value={fontSize}
                onChange={(e) => setFontSize(e.target.value)}
                placeholder="e.g. 16px, 1.2rem"
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="bgColor">Background Color</Label>
              <Input
                id="bgColor"
                value={backgroundColor}
                onChange={(e) => setBackgroundColor(e.target.value)}
                placeholder="transparent or rgba(0,0,0,0.5)"
              />
            </div>

            <Button
              onClick={handleSave}
              disabled={isSaving || widgetId === "loading..."}
              className="w-full flex items-center justify-center gap-2 mt-2"
            >
              <Save className="h-4 w-4" />
              {isSaving ? "Saving..." : "Save Changes"}
            </Button>
          </div>

          <div className="mt-4 pt-4 border-t border-border">
            <h2 className="text-lg font-semibold mb-2">OBS Source Size</h2>
            <p className="text-muted-foreground text-sm mb-4">
              Simulate your OBS browser source dimensions.
            </p>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="width">Width (px)</Label>
                <Input
                  id="width"
                  type="number"
                  value={width}
                  onChange={(e) => setWidth(e.target.value)}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="height">Height (px)</Label>
                <Input
                  id="height"
                  type="number"
                  value={height}
                  onChange={(e) => setHeight(e.target.value)}
                />
              </div>
            </div>
          </div>

          <div className="mt-4 pt-4 border-t border-border">
            <h2 className="text-xl font-semibold mb-2">Widget URL</h2>
            <p className="text-muted-foreground text-sm mb-4">
              Copy this URL and add it as a Browser Source in OBS.
            </p>
            <div className="flex gap-2">
              <Input
                readOnly
                value={widgetUrl || "Generating..."}
                className="font-mono text-xs"
              />
              <Button
                onClick={handleCopy}
                disabled={!widgetUrl}
                variant="secondary"
                className="shrink-0 flex items-center gap-1.5"
              >
                {copied ? (
                  <>
                    <Check className="h-4 w-4 text-primary" />
                    Copied!
                  </>
                ) : (
                  <>
                    <Copy className="h-4 w-4" />
                    Copy
                  </>
                )}
              </Button>
            </div>
          </div>
        </div>

        {/* Preview Panel */}
        <div className="p-6 bg-card rounded-lg border border-border lg:col-span-2 flex flex-col min-h-[500px]">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-xl font-semibold">Live Preview</h2>
            <span className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium bg-secondary text-secondary-foreground border border-border">
              <span className="h-2 w-2 rounded-full bg-primary animate-pulse" />
              Preview Mode (Simulated Chat)
            </span>
          </div>

          <div className="flex-1 rounded-md border border-dashed border-border overflow-auto relative bg-background/30 flex items-center justify-center p-8">
            {/* Checkerboard background for transparent preview */}
            <div
              className="absolute inset-0 z-0 opacity-10 pointer-events-none"
              style={{
                backgroundImage:
                  "linear-gradient(45deg, var(--muted) 25%, transparent 25%), linear-gradient(-45deg, var(--muted) 25%, transparent 25%), linear-gradient(45deg, transparent 75%, var(--muted) 75%), linear-gradient(-45deg, transparent 75%, var(--muted) 75%)",
                backgroundSize: "20px 20px",
                backgroundPosition: "0 0, 0 10px, 10px -10px, -10px 0px",
              }}
            />
            <div
              className="relative z-10 shadow-2xl border border-border/50 bg-background/50 flex-shrink-0"
              style={{ width: `${width}px`, height: `${height}px` }}
            >
              <WidgetClient
                widgetId={widgetId}
                theme={theme}
                fontSize={fontSize}
                backgroundColor={backgroundColor}
                autoHideSeconds={autoHideSeconds}
                layoutStyle={layoutStyle}
                mock={true}
              />
            </div>
          </div>
        </div>
      </div>

      <ObsSetupGuideModal
        isOpen={isGuideOpen}
        onClose={() => setIsGuideOpen(false)}
        widgetUrl={widgetUrl}
      />
    </div>
  );
}
