"use client";

import { useEffect, useState, useSyncExternalStore } from "react";
import Link from "next/link";
import {
  CheckCircle2,
  Circle,
  ArrowRight,
  Copy,
  Check,
  HelpCircle,
  Radio,
  Sliders,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { createClient } from "@/lib/supabase/client";
import { useToast } from "@/components/ui/toast";
import { ObsSetupGuideModal } from "@/components/obs/ObsSetupGuideModal";
import { useOrigin } from "@/lib/use-origin";
import { resolveAccountWidget, type WidgetRecord } from "@/lib/widget-selection";

function subscribeStorage(callback: () => void) {
  window.addEventListener("storage", callback);
  return () => window.removeEventListener("storage", callback);
}

function getCopiedSnapshot() {
  return typeof window !== "undefined" ? localStorage.getItem("streamsync_obs_copied_widget") : null;
}

function getServerCopiedSnapshot() {
  return null;
}

export default function DashboardPage() {
  const [loading, setLoading] = useState(true);
  const [platforms, setPlatforms] = useState<string[]>([]);
  const [widget, setWidget] = useState<WidgetRecord | null>(null);
  const [widgetError, setWidgetError] = useState("");
  const origin = useOrigin();
  const [copiedId, setCopiedId] = useState<string | null>(null);
  const isCopiedInStorage = useSyncExternalStore(
    subscribeStorage,
    getCopiedSnapshot,
    getServerCopiedSnapshot
  );
  const hasCopied = Boolean(widget && (isCopiedInStorage === widget.id || copiedId === widget.id));
  const [isGuideOpen, setIsGuideOpen] = useState(false);

  const { success, error: toastError } = useToast();

  useEffect(() => {
    let mounted = true;
    let request = 0;
    const supabase = createClient();
    const authTimers = new Set<ReturnType<typeof setTimeout>>();

    async function revalidate() {
      const currentRequest = ++request;
      setWidget(null);
      setPlatforms([]);
      setCopiedId(null);
      setWidgetError("");
      setLoading(true);
      try {
        const { data: { user }, error: authError } = await supabase.auth.getUser();
        if (!mounted || currentRequest !== request) return;
        if (authError) throw new Error(authError.message);
        if (!user) return;
        const [tokensRes, canonical] = await Promise.all([
          supabase.from("platform_tokens").select("platform"),
          resolveAccountWidget(),
        ]);
        if (!mounted || currentRequest !== request) return;
        if (tokensRes.data) setPlatforms(tokensRes.data.map((p) => p.platform));
        setWidget(canonical);
      } catch (err) {
        if (mounted && currentRequest === request) {
          setWidgetError(err instanceof Error ? err.message : "Failed to load widgets.");
        }
      } finally {
        if (mounted && currentRequest === request) setLoading(false);
      }
    }

    const onFocus = () => { void revalidate(); };
    window.addEventListener("focus", onFocus);
    const { data: { subscription } } = supabase.auth.onAuthStateChange(() => {
      const timer = setTimeout(() => {
        authTimers.delete(timer);
        if (mounted) void revalidate();
      }, 0);
      authTimers.add(timer);
    });
    void revalidate();
    return () => {
      mounted = false;
      request += 1;
      window.removeEventListener("focus", onFocus);
      subscription.unsubscribe();
      authTimers.forEach(clearTimeout);
    };
  }, []);

  const widgetUrl =
    widget?.id && origin ? `${origin}/widget/${widget.id}` : "";

  const handleCopy = async () => {
    if (!widgetUrl || !widget) {
      toastError("Widget not yet generated. Visit widget settings to create one.");
      return;
    }
    const currentId = widget.id;
    try {
      await navigator.clipboard.writeText(widgetUrl);
      setCopiedId(currentId);
      if (typeof window !== "undefined") {
        localStorage.setItem("streamsync_obs_copied_widget", currentId);
      }
      success("Widget URL copied to clipboard!");
      setTimeout(() => setCopiedId((id) => id === currentId ? null : id), 2000);
    } catch {
      toastError("Failed to copy URL to clipboard.");
    }
  };

  const isStep1Done = platforms.length > 0;
  const isStep2Done = Boolean(widget?.id);
  const isStep3Done = hasCopied && Boolean(widget);

  const completedCount =
    (isStep1Done ? 1 : 0) + (isStep2Done ? 1 : 0) + (isStep3Done ? 1 : 0);
  const completionPercentage = Math.round((completedCount / 3) * 100);

  return (
    <div className="flex flex-col gap-8 max-w-5xl">
      <div>
        <h1 className="text-3xl font-heading font-bold text-foreground">
          Dashboard Overview
        </h1>
        <p className="text-muted-foreground text-sm mt-1">
          Welcome to StreamSync. Complete the setup steps below to get your unified chat into OBS.
        </p>
      </div>

      {widgetError && <p role="alert" className="rounded-md border border-destructive p-4 text-sm text-destructive">{widgetError}</p>}

      {/* Setup Progress Checklist Card */}
      <div className="p-6 bg-card rounded-lg border border-border flex flex-col gap-6">
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
          <div>
            <h2 className="text-xl font-semibold flex items-center gap-2">
              <span>Onboarding Progress</span>
              <span className="text-sm font-normal text-muted-foreground">
                ({completedCount} of 3 completed)
              </span>
            </h2>
            <p className="text-sm text-muted-foreground mt-0.5">
              Follow these three steps to bring live stream chats into your stream overlay.
            </p>
          </div>
          <div className="flex items-center gap-3">
            <span className="text-sm font-semibold text-primary">
              {completionPercentage}% Complete
            </span>
            <div className="w-28 h-2.5 bg-secondary rounded-full overflow-hidden">
              <div
                className="h-full bg-primary transition-all duration-500 rounded-full"
                style={{ width: `${completionPercentage}%` }}
              />
            </div>
          </div>
        </div>

        {/* 3 Steps */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {/* Step 1: Connect Accounts */}
          <div className="p-4 rounded-md border border-border/80 bg-background/50 flex flex-col justify-between gap-4">
            <div>
              <div className="flex items-center justify-between mb-2">
                <span className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
                  Step 1
                </span>
                {isStep1Done ? (
                  <CheckCircle2 className="h-5 w-5 text-primary" />
                ) : (
                  <Circle className="h-5 w-5 text-muted-foreground" />
                )}
              </div>
              <h3 className="font-semibold text-foreground mb-1">
                Connect Accounts
              </h3>
              <p className="text-xs text-muted-foreground mb-3 leading-relaxed">
                Connect your streaming accounts (Twitch, YouTube, Kick) to aggregate chat.
              </p>
              {loading ? (
                <div className="text-xs text-muted-foreground">Loading status...</div>
              ) : isStep1Done ? (
                <div className="flex flex-wrap gap-1.5">
                  {platforms.map((p) => (
                    <span
                      key={p}
                      className="px-2 py-0.5 text-[10px] font-semibold uppercase rounded bg-secondary text-secondary-foreground border border-border"
                    >
                      {p}
                    </span>
                  ))}
                </div>
              ) : (
                <span className="text-xs text-muted-foreground">
                  No accounts connected yet
                </span>
              )}
            </div>
            <Link href="/dashboard/accounts">
              <Button variant="outline" size="sm" className="w-full text-xs">
                Manage Accounts
                <ArrowRight className="h-3.5 w-3.5 ml-1" />
              </Button>
            </Link>
          </div>

          {/* Step 2: Customize Overlay */}
          <div className="p-4 rounded-md border border-border/80 bg-background/50 flex flex-col justify-between gap-4">
            <div>
              <div className="flex items-center justify-between mb-2">
                <span className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
                  Step 2
                </span>
                {isStep2Done ? (
                  <CheckCircle2 className="h-5 w-5 text-primary" />
                ) : (
                  <Circle className="h-5 w-5 text-muted-foreground" />
                )}
              </div>
              <h3 className="font-semibold text-foreground mb-1">
                Customize Overlay
              </h3>
              <p className="text-xs text-muted-foreground mb-3 leading-relaxed">
                Tailor theme, font sizes, and background transparency for your overlay.
              </p>
              {loading ? (
                <div className="text-xs text-muted-foreground">Loading status...</div>
              ) : widget ? (
                <div className="text-xs text-muted-foreground space-y-0.5">
                  <div>
                    Theme:{" "}
                    <span className="font-medium text-foreground capitalize">
                      {widget.theme || "Dark"}
                    </span>
                  </div>
                  <div>
                    Font Size:{" "}
                    <span className="font-medium text-foreground">
                      {widget.font_size || "16px"}
                    </span>
                  </div>
                </div>
              ) : (
                <span className="text-xs text-muted-foreground">
                  Default settings loaded
                </span>
              )}
            </div>
            <Link href="/dashboard/widget">
              <Button variant="outline" size="sm" className="w-full text-xs">
                Customize Widget
                <ArrowRight className="h-3.5 w-3.5 ml-1" />
              </Button>
            </Link>
          </div>

          {/* Step 3: Add to OBS */}
          <div className="p-4 rounded-md border border-border/80 bg-background/50 flex flex-col justify-between gap-4">
            <div>
              <div className="flex items-center justify-between mb-2">
                <span className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
                  Step 3
                </span>
                {isStep3Done ? (
                  <CheckCircle2 className="h-5 w-5 text-primary" />
                ) : (
                  <Circle className="h-5 w-5 text-muted-foreground" />
                )}
              </div>
              <h3 className="font-semibold text-foreground mb-1">Add to OBS</h3>
              <p className="text-xs text-muted-foreground mb-3 leading-relaxed">
                Copy your personal Browser Source URL and paste into OBS Studio.
              </p>
              <div className="flex flex-col gap-2">
                <Button
                  size="sm"
                  variant="secondary"
                  onClick={handleCopy}
                  disabled={!widgetUrl}
                  className="w-full text-xs flex items-center justify-center gap-1.5"
                >
                  {copiedId === widget?.id ? (
                    <>
                      <Check className="h-3.5 w-3.5 text-primary" />
                      Copied to Clipboard!
                    </>
                  ) : (
                    <>
                      <Copy className="h-3.5 w-3.5" />
                      Quick Copy URL
                    </>
                  )}
                </Button>
              </div>
            </div>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setIsGuideOpen(true)}
              className="w-full text-xs flex items-center justify-center gap-1.5"
            >
              <HelpCircle className="h-3.5 w-3.5" />
              View OBS Guide
            </Button>
          </div>
        </div>
      </div>

      {/* Navigation Quick Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <Link href="/dashboard/accounts" className="block group">
          <div className="p-6 bg-card rounded-lg border border-border h-full transition-colors group-hover:border-primary">
            <h2 className="text-xl font-semibold mb-2 flex items-center justify-between">
              <span className="flex items-center gap-2">
                <Radio className="h-5 w-5 text-primary" />
                Connected Accounts
              </span>
              <ArrowRight className="h-5 w-5 text-muted-foreground group-hover:text-primary transition-colors" />
            </h2>
            <p className="text-muted-foreground text-sm">
              Connect or reconnect your Twitch, YouTube, and Kick integrations.
            </p>
          </div>
        </Link>

        <Link href="/dashboard/widget" className="block group">
          <div className="p-6 bg-card rounded-lg border border-border h-full transition-colors group-hover:border-primary">
            <h2 className="text-xl font-semibold mb-2 flex items-center justify-between">
              <span className="flex items-center gap-2">
                <Sliders className="h-5 w-5 text-primary" />
                Widget Settings
              </span>
              <ArrowRight className="h-5 w-5 text-muted-foreground group-hover:text-primary transition-colors" />
            </h2>
            <p className="text-muted-foreground text-sm">
              Adjust your chat overlay styling, view live preview, and copy source URL.
            </p>
          </div>
        </Link>
      </div>

      <ObsSetupGuideModal
        isOpen={isGuideOpen}
        onClose={() => setIsGuideOpen(false)}
        widgetUrl={widgetUrl}
      />
    </div>
  );
}
