'use client';

import { createClient } from "@/lib/supabase/client";
import { useEffect, useState } from "react";
import { CheckCircle2 } from "lucide-react";

export default function AccountsPage() {
  const supabase = createClient();
  const [connectedPlatforms, setConnectedPlatforms] = useState<string[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function fetchConnections() {
      const { data, error } = await supabase
        .from('platform_tokens')
        .select('platform');
      
      if (!error && data) {
        setConnectedPlatforms(data.map(t => t.platform));
      }
      setLoading(false);
    }
    fetchConnections();
  }, [supabase]);

  const handleConnect = async (platform: string) => {
    console.log(`Connecting to ${platform}...`);
    let providerName = platform;
    if (platform === 'youtube') providerName = 'google';
    if (platform === 'kick') providerName = 'kick';
    
    await supabase.auth.linkIdentity({
      provider: providerName as "twitch" | "google" | "github",
      options: {
        redirectTo: `${window.location.origin}/auth/callback?next=/dashboard/accounts&provider=${platform}`,
        scopes: platform === 'youtube' ? 'https://www.googleapis.com/auth/youtube.readonly' : undefined
      },
    });
  };

  const isConnected = (platform: string) => connectedPlatforms.includes(platform);

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
            </div>
            {isConnected('twitch') && <CheckCircle2 className="text-green-500 h-6 w-6" />}
          </div>
          <button 
            onClick={() => handleConnect('twitch')}
            disabled={loading || isConnected('twitch')}
            className="px-4 py-2 bg-twitch text-white rounded-md hover:opacity-90 mt-auto disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isConnected('twitch') ? 'Connected' : 'Connect Twitch'}
          </button>
        </div>

        <div className="p-6 bg-card rounded-lg border border-border flex flex-col items-start gap-4">
          <div className="w-full flex justify-between items-start">
            <div>
              <h2 className="text-xl font-semibold">YouTube</h2>
              <p className="text-sm text-muted-foreground">Connect your YouTube Live channel</p>
            </div>
            {isConnected('youtube') && <CheckCircle2 className="text-green-500 h-6 w-6" />}
          </div>
          <button 
            onClick={() => handleConnect('youtube')}
            disabled={loading || isConnected('youtube')}
            className="px-4 py-2 bg-youtube text-white rounded-md hover:opacity-90 mt-auto disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isConnected('youtube') ? 'Connected' : 'Connect YouTube'}
          </button>
        </div>

        <div className="p-6 bg-card rounded-lg border border-border flex flex-col items-start gap-4">
          <div className="w-full flex justify-between items-start">
            <div>
              <h2 className="text-xl font-semibold">Kick</h2>
              <p className="text-sm text-muted-foreground">Connect your Kick channel</p>
            </div>
            {isConnected('kick') && <CheckCircle2 className="text-green-500 h-6 w-6" />}
          </div>
          <button 
            onClick={() => handleConnect('kick')}
            disabled={loading || isConnected('kick')}
            className="px-4 py-2 bg-kick text-black rounded-md hover:opacity-90 mt-auto disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isConnected('kick') ? 'Connected' : 'Connect Kick'}
          </button>
        </div>
      </div>
    </div>
  );
}
