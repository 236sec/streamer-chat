import { cookies } from "next/headers";
import { NextResponse } from "next/server";
import { createServerClient } from "@supabase/ssr";
import { env } from "@/env";
import crypto from "crypto";

function encryptToken(token: string, rawKey: string): string {
  const key = Buffer.from(rawKey.padEnd(32, "0").slice(0, 32), "utf8");
  const iv = crypto.randomBytes(12);
  const cipher = crypto.createCipheriv("aes-256-gcm", key, iv);

  let encrypted = cipher.update(token, "utf8", "base64");
  encrypted += cipher.final("base64");
  const authTag = cipher.getAuthTag().toString("base64");

  return `${iv.toString("base64")}:${authTag}:${encrypted}`;
}

export async function GET(request: Request) {
  const requestUrl = new URL(request.url);
  const code = requestUrl.searchParams.get("code");
  const next = requestUrl.searchParams.get("next") ?? "/dashboard";
  const explicitProvider = requestUrl.searchParams.get("provider");

  if (code) {
    const cookieStore = await cookies();
    const supabase = createServerClient(
      env.NEXT_PUBLIC_SUPABASE_URL,
      env.NEXT_PUBLIC_SUPABASE_ANON_KEY,
      {
        cookies: {
          getAll() {
            return cookieStore.getAll();
          },
          setAll(cookiesToSet) {
            try {
              cookiesToSet.forEach(({ name, value, options }) =>
                cookieStore.set(name, value, options)
              );
            } catch {
            }
          },
        },
      }
    );
    
    const { data: { session } } = await supabase.auth.exchangeCodeForSession(code);

    if (session && session.provider_token) {
      const { data: { user } } = await supabase.auth.getUser();
      if (user) {
        // Use the explicit provider passed in the redirect URL if available
        let platform = explicitProvider;
        
        if (!platform) {
          const currentIdentity = user.identities?.find(i => i.provider !== 'email');
          platform = currentIdentity?.provider || user.app_metadata?.provider || '';
          if (platform === 'google') platform = 'youtube';
        }
        
        if (platform) {
          const rawKey = process.env.MASTER_DECRYPTION_KEY || "0123456789abcdef0123456789abcdef";
          const encryptedToken = encryptToken(session.provider_token, rawKey);
          const encryptedRefreshToken = session.provider_refresh_token
            ? encryptToken(session.provider_refresh_token, rawKey)
            : undefined;

          const tokenData: {
            user_id: string;
            platform: string;
            encrypted_token: string;
            encrypted_refresh_token?: string;
          } = {
            user_id: user.id,
            platform,
            encrypted_token: encryptedToken,
          };

          if (encryptedRefreshToken) {
            tokenData.encrypted_refresh_token = encryptedRefreshToken;
          }

          // Save to platform_tokens table
          const { error } = await supabase
            .from("platform_tokens")
            .upsert(tokenData, { onConflict: "user_id,platform" });

          if (error) {
            console.error("Failed to save platform token:", error);
          }
        }
      }
    }
  }

  // URL to redirect to after sign in process completes
  return NextResponse.redirect(new URL(next, requestUrl.origin));
}
