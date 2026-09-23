'use client';

import { createClient } from "@/lib/supabase/client";
import { useEffect, useState } from "react";
import { CheckCircle2 } from "lucide-react";

export default function AccountsPage() {
  const supabase = createClient();
  const [platformTokens, setPlatformTokens] = useState<{platform: string, encrypted_refresh_token: string | null}[]>([]);
  const [loading, setLoading] = useState(true);
  const [kickUsername, setKickUsername] = useState("");
  const [isKickPromptOpen, setIsKickPromptOpen] = useState(false);

  useEffect(() => {
    async function fetchConnections() {
      const { data, error } = await supabase
        .from('platform_tokens')
        .select('platform, encrypted_refresh_token');
      
      if (!error && data) {
        setPlatformTokens(data);
      }
      setLoading(false);
    }
    fetchConnections();
  }, [supabase]);

  const handleConnect = async (platform: string) => {
    if (platform === 'kick') {
      setIsKickPromptOpen(true);
      return;
    }

    let providerName = platform;
    if (platform === 'youtube') providerName = 'google';

    const oauthOptions = {
      redirectTo: `${window.location.origin}/auth/callback?next=/dashboard/accounts&provider=${platform}`,
      scopes: platform === 'twitch'
                ? 'user:read:chat user:read:email'
                : platform === 'youtube'
                  ? 'https://www.googleapis.com/auth/youtube.readonly'
                  : undefined,
      queryParams: platform === 'youtube'
        ? { access_type: 'offline', prompt: 'consent' }
        : undefined,
    };

    // Try linkIdentity first (for new connections)
    const { error } = await supabase.auth.linkIdentity({
      provider: providerName as "twitch" | "google" | "github",
      options: oauthOptions,
    });

    if (error) {
      // Identity already linked — use signInWithOAuth to re-authorize and refresh the token
      console.log(`linkIdentity failed (${error.message}), falling back to signInWithOAuth`);
      await supabase.auth.signInWithOAuth({
        provider: providerName as "twitch" | "google" | "github",
        options: oauthOptions,
      });
    }
  };

  const handleKickSubmit = async () => {
    if (!kickUsername.trim()) return;
    setLoading(true);
    try {
      const res = await fetch("/api/kick/connect", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ username: kickUsername.trim() }),
      });
      if (res.ok) {
        setPlatformTokens((prev) => [...prev, { platform: "kick", encrypted_refresh_token: null }]);
        setIsKickPromptOpen(false);
        setKickUsername("");
      } else {
        console.error("Failed to connect Kick");
      }
    } catch (e) {
      console.error(e);
    }
    setLoading(false);
  };

  const handleDisconnect = async (platform: string) => {
    const { error } = await supabase
      .from('platform_tokens')
      .delete()
      .eq('platform', platform);

    if (error) {
      console.error(`Failed to disconnect ${platform}:`, error);
      return;
    }

    setPlatformTokens(prev => prev.filter(p => p.platform !== platform));
  };

  const isConnected = (platform: string) => platformTokens.some(p => p.platform === platform);
  const hasRefreshToken = (platform: string) => platformTokens.some(p => p.platform === platform && p.encrypted_refresh_token !== null);

  return (
    <div className="flex flex-col gap-6">
      <h1 className="text-3xl font-heading font-bold text-foreground">Connected Accounts</h1>
      <p className="text-muted-foreground">Manage your Twitch, YouTube, and Kick integrations to stream your chat to OBS.</p>
      
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        <div className="p-6 bg-card rounded-lg border border-border flex flex-col items-start gap-4">
          <div className="w-full flex justify-between items-start">
            <div>
              <h2 className="text-xl font-semibold">Twitch</h2>
              <p className="text-sm text-muted-foreground">Connect your Twitch channel</p>
              {isConnected('twitch') && !hasRefreshToken('twitch') && (
                <p className="text-xs text-amber-500 mt-1">
                  ⚠ No refresh token. Reconnect to fix token expiry issues.
                </p>
              )}
            </div>
            {isConnected('twitch') && <CheckCircle2 className="text-green-500 h-6 w-6" />}
          </div>
          <div className="flex gap-2 mt-auto">
            <button 
              onClick={() => handleConnect('twitch')}
              disabled={loading}
              className="px-4 py-2 bg-twitch text-white rounded-md hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isConnected('twitch') ? 'Reconnect' : 'Connect Twitch'}
            </button>
            {isConnected('twitch') && (
              <button
                onClick={() => handleDisconnect('twitch')}
                disabled={loading}
                className="px-4 py-2 bg-destructive text-destructive-foreground rounded-md hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Disconnect
              </button>
            )}
          </div>
        </div>

        <div className="p-6 bg-card rounded-lg border border-border flex flex-col items-start gap-4">
          <div className="w-full flex justify-between items-start">
            <div>
              <h2 className="text-xl font-semibold">YouTube</h2>
              <p className="text-sm text-muted-foreground">Connect your YouTube Live channel</p>
              {isConnected('youtube') && !hasRefreshToken('youtube') && (
                <p className="text-xs text-amber-500 mt-1">
                  ⚠ No refresh token. Reconnect to fix token expiry issues.
                </p>
              )}
            </div>
            {isConnected('youtube') && <CheckCircle2 className="text-green-500 h-6 w-6" />}
          </div>
          <div className="flex gap-2 mt-auto">
            <button 
              onClick={() => handleConnect('youtube')}
              disabled={loading}
              className="px-4 py-2 bg-youtube text-white rounded-md hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isConnected('youtube') ? 'Reconnect' : 'Connect YouTube'}
            </button>
            {isConnected('youtube') && (
              <button
                onClick={() => handleDisconnect('youtube')}
                disabled={loading}
                className="px-4 py-2 bg-destructive text-destructive-foreground rounded-md hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Disconnect
              </button>
            )}
          </div>
        </div>

        <div className="p-6 bg-card rounded-lg border border-border flex flex-col items-start gap-4">
          <div className="w-full flex justify-between items-start">
            <div>
              <h2 className="text-xl font-semibold">Kick</h2>
              <p className="text-sm text-muted-foreground">Connect your Kick channel</p>
            </div>
            {isConnected('kick') && <CheckCircle2 className="text-green-500 h-6 w-6" />}
          </div>
          <div className="flex gap-2 mt-auto">
            <button 
              onClick={() => handleConnect('kick')}
              disabled={loading}
              className="px-4 py-2 bg-kick text-black rounded-md hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isConnected('kick') ? 'Reconnect' : 'Connect Kick'}
            </button>
            {isConnected('kick') && (
              <button
                onClick={() => handleDisconnect('kick')}
                disabled={loading}
                className="px-4 py-2 bg-destructive text-destructive-foreground rounded-md hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Disconnect
              </button>
            )}
          </div>
        </div>
      </div>

      {isKickPromptOpen && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div className="bg-card p-6 rounded-lg shadow-lg border border-border w-full max-w-md flex flex-col gap-4">
            <h2 className="text-xl font-bold">Connect Kick</h2>
            <p className="text-sm text-muted-foreground">Enter your Kick username to connect your chat.</p>
            <input
              type="text"
              placeholder="Kick Username"
              value={kickUsername}
              onChange={(e) => setKickUsername(e.target.value)}
              className="w-full px-3 py-2 bg-background border border-input rounded-md"
              disabled={loading}
            />
            <div className="flex justify-end gap-2 mt-4">
              <button
                onClick={() => setIsKickPromptOpen(false)}
                disabled={loading}
                className="px-4 py-2 bg-secondary text-secondary-foreground rounded-md hover:opacity-90"
              >
                Cancel
              </button>
              <button
                onClick={handleKickSubmit}
                disabled={loading || !kickUsername.trim()}
                className="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:opacity-90 disabled:opacity-50"
              >
                {loading ? "Connecting..." : "Connect"}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
