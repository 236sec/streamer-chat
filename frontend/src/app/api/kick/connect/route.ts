import { NextResponse } from "next/server";
import { createServerClient } from "@supabase/ssr";
import { cookies } from "next/headers";
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

export async function POST(request: Request) {
  try {
    const { username } = await request.json();
    if (!username) {
      return NextResponse.json({ error: "Username is required" }, { status: 400 });
    }

    const cookieStore = await cookies();
    const supabase = createServerClient(
      env.NEXT_PUBLIC_SUPABASE_URL,
      env.NEXT_PUBLIC_SUPABASE_ANON_KEY,
      {
        cookies: {
          getAll() { return cookieStore.getAll(); },
          setAll(cookiesToSet) {
            try {
              cookiesToSet.forEach(({ name, value, options }) =>
                cookieStore.set(name, value, options)
              );
            } catch {}
          },
        },
      }
    );

    const { data: { user } } = await supabase.auth.getUser();
    if (!user) {
      return NextResponse.json({ error: "Unauthorized" }, { status: 401 });
    }

    const rawKey = process.env.MASTER_DECRYPTION_KEY || "0123456789abcdef0123456789abcdef";
    const encryptedToken = encryptToken(username, rawKey);

    const { error } = await supabase
      .from("platform_tokens")
      .upsert({
        user_id: user.id,
        platform: "kick",
        encrypted_token: encryptedToken,
      }, { onConflict: "user_id,platform" });

    if (error) {
      console.error("Supabase upsert error:", error);
      return NextResponse.json({ error: "Failed to save Kick account" }, { status: 500 });
    }

    return NextResponse.json({ success: true });
  } catch (error) {
    console.error("Kick connect error:", error);
    return NextResponse.json({ error: "Internal Server Error" }, { status: 500 });
  }
}
