import { z } from "zod";

const envSchema = z.object({
  NEXT_PUBLIC_SUPABASE_URL: z.string().url(),
  NEXT_PUBLIC_SUPABASE_ANON_KEY: z.string().min(1),
  NEXT_PUBLIC_WS_URL: z.string().url().default("ws://127.0.0.1:8080/ws"),
});

const isBuild = process.env.npm_lifecycle_event === "build" || process.env.NEXT_PHASE === "phase-production-build" || (!process.env.NEXT_PUBLIC_SUPABASE_URL && process.env.NODE_ENV === "production");

export const env = isBuild
  ? {
      NEXT_PUBLIC_SUPABASE_URL: "http://localhost:54321",
      NEXT_PUBLIC_SUPABASE_ANON_KEY: "dummy",
      NEXT_PUBLIC_WS_URL: "ws://127.0.0.1:8080/ws",
    }
  : envSchema.parse({
      NEXT_PUBLIC_SUPABASE_URL: process.env.NEXT_PUBLIC_SUPABASE_URL,
      NEXT_PUBLIC_SUPABASE_ANON_KEY: process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY,
      NEXT_PUBLIC_WS_URL: process.env.NEXT_PUBLIC_WS_URL,
    });
