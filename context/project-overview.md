# Streamer Chat Display

## Overview

A unified chat display application for streamers that aggregates chat events from Twitch, YouTube, and Kick. It allows streamers to configure a custom widget via a web dashboard and display it directly in OBS as a browser source.

## Goals

1. Ingest chat messages in real-time from Twitch, YouTube, and Kick.
2. Provide streamers a unified web dashboard to manage their accounts and widget appearance.
3. Serve a reliable, customizable OBS browser source widget overlay.

## Core User Flow

1. Streamer lands on the homepage and signs in with Supabase (e.g. Twitch/YouTube).
2. Navigates to the dashboard and connects remaining accounts (Twitch, YouTube, Kick).
3. Configures the chat widget appearance (theme, font, size).
4. Copies the unique widget URL.
5. Pastes it into OBS as a browser source to see real-time unified chat on stream.

## Features

### Authentication & Integrations
- Supabase user authentication
- OAuth/API integrations for Twitch, YouTube, and Kick chat streams

### Real-Time Chat Engine
- WebSocket connections to supported platforms (Rust backend)
- Unified message broadcasting to connected widgets

### Customization
- Basic chat customization (font size, background color, theme) in Next.js dashboard

## Scope

### In Scope

- Unified read-only chat feed combining Twitch, YouTube, and Kick
- Basic chat widget customization
- User authentication via Supabase
- OBS browser source support via public unique widget URL
- Separate OBS viewer-count source for Twitch, YouTube, and Kick under the same widget identity

### Out of Scope

- In-app chat replies (read-only for now)
- Complex moderation tools (timeouts, bans)
- Custom emote uploads
- Monetization features
- Additional platforms like TikTok or Facebook Gaming

## Success Criteria

1. Streamer can log in via Supabase.
2. Streamer can connect external platform accounts successfully.
3. Streamer can configure widget appearance in the dashboard.
4. OBS widget displays real-time chat messages from connected platforms without manual refreshing.
