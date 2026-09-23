"use client";

import { createClient } from "@/lib/supabase/client";
import { useEffect, useState, Suspense } from "react";
import { useSearchParams } from "next/navigation";
import {
  CheckCircle2,
  AlertCircle,
  AlertTriangle,
  X,
  Radio,
  Trash2,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useToast } from "@/components/ui/toast";

interface PlatformToken {
  platform: string;
  encrypted_refresh_token: string | null;
}

function AccountsContent() {
  const supabase = createClient();
  const searchParams = useSearchParams();
  const { success, error: toastError } = useToast();

  const [platformTokens, setPlatformTokens] = useState<PlatformToken[]>([]);
  const [loading, setLoading] = useState(true);

  // Kick modal state
  const [isKickPromptOpen, setIsKickPromptOpen] = useState(false);
  const [kickUsername, setKickUsername] = useState("");
  const [kickInputError, setKickInputError] = useState<string | null>(null);
  const [isKickSubmitting, setIsKickSubmitting] = useState(false);

  // Disconnect modal state
  const [disconnectTarget, setDisconnectTarget] = useState<string | null>(null);
  const [isDisconnecting, setIsDisconnecting] = useState(false);

  // OAuth error banner state
  const oauthError = searchParams.get("error");
  const oauthErrorDescription = searchParams.get("error_description");
  const [isOAuthErrorDismissed, setIsOAuthErrorDismissed] = useState(false);

  useEffect(() => {
    async function fetchConnections() {
      const { data, error } = await supabase
        .from("platform_tokens")
        .select("platform, encrypted_refresh_token");

      if (!error && data) {
        setPlatformTokens(data);
      }
      setLoading(false);
    }
    fetchConnections();
  }, [supabase]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        if (isKickPromptOpen) setIsKickPromptOpen(false);
        if (disconnectTarget) setDisconnectTarget(null);
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isKickPromptOpen, disconnectTarget]);

  const handleConnect = async (platform: string) => {
    if (platform === "kick") {
      setKickInputError(null);
      setKickUsername("");
      setIsKickPromptOpen(true);
      return;
    }

    let providerName = platform;
    if (platform === "youtube") providerName = "google";

    const oauthOptions = {
      redirectTo: `${window.location.origin}/auth/callback?next=/dashboard/accounts&provider=${platform}`,
      scopes:
        platform === "twitch"
          ? "user:read:chat user:read:email"
          : platform === "youtube"
          ? "https://www.googleapis.com/auth/youtube.readonly"
          : undefined,
      queryParams:
        platform === "youtube"
          ? { access_type: "offline", prompt: "consent" }
          : undefined,
    };

    try {
      const { error } = await supabase.auth.linkIdentity({
        provider: providerName as "twitch" | "google" | "github",
        options: oauthOptions,
      });

      if (error) {
        await supabase.auth.signInWithOAuth({
          provider: providerName as "twitch" | "google" | "github",
          options: oauthOptions,
        });
      }
    } catch {
      toastError(`Failed to initiate ${platform} authorization.`);
    }
  };

  const handleKickSubmit = async () => {
    setKickInputError(null);
    const raw = kickUsername.trim();
    const cleanUsername = raw.replace(/^@+/, "");

    if (!cleanUsername) {
      setKickInputError("Kick channel username cannot be empty.");
      return;
    }

    if (/\s/.test(cleanUsername)) {
      setKickInputError("Kick username cannot contain spaces.");
      return;
    }

    setIsKickSubmitting(true);
    try {
      const res = await fetch("/api/kick/connect", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ username: cleanUsername }),
      });

      const data = await res.json().catch(() => null);

      if (res.ok) {
        setPlatformTokens((prev) => [
          ...prev.filter((p) => p.platform !== "kick"),
          { platform: "kick", encrypted_refresh_token: null },
        ]);
        setIsKickPromptOpen(false);
        setKickUsername("");
        success(`Kick channel "${cleanUsername}" connected successfully!`);
      } else {
        const errorMsg =
          data?.error || "Failed to connect Kick account. Please check the username.";
        setKickInputError(errorMsg);
        toastError(errorMsg, "Kick Connection Failed");
      }
    } catch {
      const errorMsg = "Network error connecting to Kick. Please try again.";
      setKickInputError(errorMsg);
      toastError(errorMsg, "Connection Error");
    } finally {
      setIsKickSubmitting(false);
    }
  };

  const handleConfirmDisconnect = async () => {
    if (!disconnectTarget) return;
    setIsDisconnecting(true);
    try {
      const { error } = await supabase
        .from("platform_tokens")
        .delete()
        .eq("platform", disconnectTarget);

      if (error) {
        toastError(
          error.message || `Failed to disconnect ${disconnectTarget}.`,
          "Disconnection Failed"
        );
      } else {
        setPlatformTokens((prev) =>
          prev.filter((p) => p.platform !== disconnectTarget)
        );
        success(
          `${
            disconnectTarget.charAt(0).toUpperCase() + disconnectTarget.slice(1)
          } account disconnected successfully.`
        );
        setDisconnectTarget(null);
      }
    } catch {
      toastError(
        `Failed to disconnect ${disconnectTarget}. Please try again.`,
        "Disconnection Error"
      );
    } finally {
      setIsDisconnecting(false);
    }
  };

  const isConnected = (platform: string) =>
    platformTokens.some((p) => p.platform === platform);
  const hasRefreshToken = (platform: string) =>
    platformTokens.some(
      (p) => p.platform === platform && p.encrypted_refresh_token !== null
    );

  const platformsList = [
    {
      id: "twitch",
      name: "Twitch",
      description: "Connect your Twitch channel chat",
      requiresOfflineToken: true,
    },
    {
      id: "youtube",
      name: "YouTube",
      description: "Connect your YouTube Live channel chat",
      requiresOfflineToken: true,
    },
    {
      id: "kick",
      name: "Kick",
      description: "Connect your Kick public channel chat",
      requiresOfflineToken: false,
    },
  ];

  return (
    <div className="flex flex-col gap-6 max-w-5xl">
      <div>
        <h1 className="text-3xl font-heading font-bold text-foreground">
          Connected Accounts
        </h1>
        <p className="text-muted-foreground text-sm mt-1">
          Manage your Twitch, YouTube, and Kick integrations to stream your chat
          into OBS.
        </p>
      </div>

      {/* OAuth Error Alert Banner */}
      {(oauthError || oauthErrorDescription) && !isOAuthErrorDismissed && (
        <div
          role="alert"
          className="p-4 rounded-lg bg-card border border-destructive/60 text-card-foreground shadow-sm flex items-start justify-between gap-4"
        >
          <div className="flex items-start gap-3">
            <AlertCircle className="h-5 w-5 text-destructive shrink-0 mt-0.5" />
            <div className="space-y-1">
              <h3 className="font-semibold text-sm text-destructive">
                Authorization Error ({oauthError || "Unknown"})
              </h3>
              <p className="text-xs text-muted-foreground leading-relaxed">
                {oauthErrorDescription ||
                  "The third-party platform authorization could not be completed. Please try reconnecting."}
              </p>
            </div>
          </div>
          <button
            type="button"
            onClick={() => setIsOAuthErrorDismissed(true)}
            aria-label="Dismiss error banner"
            className="p-1 rounded text-muted-foreground hover:text-foreground transition-colors"
          >
            <X className="h-4 w-4" />
          </button>
        </div>
      )}

      {/* Platforms Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {platformsList.map((item) => {
          const connected = isConnected(item.id);
          const needsRefresh =
            connected && item.requiresOfflineToken && !hasRefreshToken(item.id);

          return (
            <div
              key={item.id}
              className="p-6 bg-card rounded-lg border border-border flex flex-col justify-between gap-5"
            >
              <div className="space-y-3">
                <div className="flex items-start justify-between">
                  <div>
                    <h2 className="text-xl font-semibold text-foreground flex items-center gap-2">
                      <Radio className="h-4 w-4 text-primary" />
                      {item.name}
                    </h2>
                    <p className="text-xs text-muted-foreground mt-0.5">
                      {item.description}
                    </p>
                  </div>
                  {connected && (
                    <span className="inline-flex items-center gap-1 text-xs font-medium text-primary">
                      <CheckCircle2 className="h-4 w-4" />
                      Active
                    </span>
                  )}
                </div>

                {needsRefresh && (
                  <div className="p-3 rounded-md bg-secondary/50 border border-border text-xs flex items-start gap-2.5">
                    <AlertTriangle className="h-4 w-4 text-primary shrink-0 mt-0.5" />
                    <div className="space-y-1">
                      <div className="font-semibold text-foreground">
                        Missing Refresh Token
                      </div>
                      <p className="text-muted-foreground text-[11px] leading-relaxed">
                        Offline refresh tokens allow continuous streaming
                        without hourly re-authentication. Reconnect to grant
                        offline permissions and prevent chat dropouts during long
                        streams.
                      </p>
                    </div>
                  </div>
                )}
              </div>

              <div className="flex items-center gap-2 pt-2 border-t border-border">
                <Button
                  onClick={() => handleConnect(item.id)}
                  disabled={loading}
                  variant={connected ? "secondary" : "default"}
                  className="flex-1 text-xs"
                >
                  {connected ? "Reconnect" : `Connect ${item.name}`}
                </Button>
                {connected && (
                  <Button
                    onClick={() => setDisconnectTarget(item.id)}
                    disabled={loading}
                    variant="destructive"
                    className="text-xs px-3"
                    aria-label={`Disconnect ${item.name}`}
                  >
                    <Trash2 className="h-3.5 w-3.5 mr-1" />
                    Disconnect
                  </Button>
                )}
              </div>
            </div>
          );
        })}
      </div>

      {/* Kick Modal */}
      {isKickPromptOpen && (
        <div
          role="dialog"
          aria-modal="true"
          aria-labelledby="kick-modal-title"
          onClick={(e) => {
            if (e.target === e.currentTarget) setIsKickPromptOpen(false);
          }}
          className="fixed inset-0 bg-black/60 backdrop-blur-xs flex items-center justify-center z-50 p-4"
        >
          <div className="bg-card text-card-foreground p-6 rounded-lg shadow-xl border border-border w-full max-w-md flex flex-col gap-4 animate-in fade-in-0 zoom-in-95 duration-150">
            <div className="flex items-center justify-between">
              <h2 id="kick-modal-title" className="text-xl font-bold">
                Connect Kick Channel
              </h2>
              <button
                type="button"
                onClick={() => setIsKickPromptOpen(false)}
                className="text-muted-foreground hover:text-foreground"
                aria-label="Close modal"
              >
                <X className="h-5 w-5" />
              </button>
            </div>
            <p className="text-sm text-muted-foreground">
              Enter your Kick channel username or slug. Kick chats are public and
              do not require OAuth tokens.
            </p>

            <div className="space-y-2">
              <Label htmlFor="kick-username">Kick Username</Label>
              <Input
                id="kick-username"
                type="text"
                placeholder="e.g. xqc or trainwreckstv"
                value={kickUsername}
                onChange={(e) => {
                  setKickUsername(e.target.value);
                  if (kickInputError) setKickInputError(null);
                }}
                disabled={isKickSubmitting}
                className={kickInputError ? "border-destructive focus-visible:ring-destructive" : ""}
              />
              {kickInputError && (
                <p className="text-xs text-destructive flex items-center gap-1 mt-1">
                  <AlertCircle className="h-3.5 w-3.5" />
                  {kickInputError}
                </p>
              )}
            </div>

            <div className="flex justify-end gap-2 mt-2">
              <Button
                variant="secondary"
                onClick={() => setIsKickPromptOpen(false)}
                disabled={isKickSubmitting}
              >
                Cancel
              </Button>
              <Button
                onClick={handleKickSubmit}
                disabled={isKickSubmitting || !kickUsername.trim()}
              >
                {isKickSubmitting ? "Connecting..." : "Connect Kick"}
              </Button>
            </div>
          </div>
        </div>
      )}

      {/* Disconnect Confirmation Modal */}
      {disconnectTarget && (
        <div
          role="dialog"
          aria-modal="true"
          aria-labelledby="disconnect-modal-title"
          onClick={(e) => {
            if (e.target === e.currentTarget) setDisconnectTarget(null);
          }}
          className="fixed inset-0 bg-black/60 backdrop-blur-xs flex items-center justify-center z-50 p-4"
        >
          <div className="bg-card text-card-foreground p-6 rounded-lg shadow-xl border border-border w-full max-w-md flex flex-col gap-4 animate-in fade-in-0 zoom-in-95 duration-150">
            <div className="flex items-center justify-between">
              <h2 id="disconnect-modal-title" className="text-xl font-bold">
                Disconnect Account
              </h2>
              <button
                type="button"
                onClick={() => setDisconnectTarget(null)}
                className="text-muted-foreground hover:text-foreground"
                aria-label="Close modal"
              >
                <X className="h-5 w-5" />
              </button>
            </div>
            <p className="text-sm text-muted-foreground leading-relaxed">
              Are you sure you want to disconnect your{" "}
              <span className="font-semibold text-foreground capitalize">
                {disconnectTarget}
              </span>{" "}
              account? Incoming chat messages from this platform will immediately
              stop streaming to your OBS overlay.
            </p>

            <div className="flex justify-end gap-2 mt-3">
              <Button
                variant="secondary"
                onClick={() => setDisconnectTarget(null)}
                disabled={isDisconnecting}
              >
                Cancel
              </Button>
              <Button
                variant="destructive"
                onClick={handleConfirmDisconnect}
                disabled={isDisconnecting}
              >
                {isDisconnecting ? "Disconnecting..." : "Confirm Disconnect"}
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default function AccountsPage() {
  return (
    <Suspense
      fallback={
        <div className="text-muted-foreground p-6 text-sm">
          Loading connected accounts...
        </div>
      }
    >
      <AccountsContent />
    </Suspense>
  );
}
