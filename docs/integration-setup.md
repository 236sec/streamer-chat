# Integration Setup Guide

To get the OAuth connection buttons working on the dashboard, you need to configure the respective Auth Providers in your **Supabase Dashboard**. 

Here is exactly what you need to do for each platform.

## 1. Twitch Setup

1. Go to the [Twitch Developer Console](https://dev.twitch.tv/console/apps) and log in.
2. Click **Register Your Application**.
3. Fill in the details:
   - **Name**: `StreamSync` (or your app name)
   - **OAuth Redirect URLs**: Enter `https://<your-project-id>.supabase.co/auth/v1/callback` (you can find your exact Supabase URL in your Supabase Dashboard under Auth > Providers > Twitch).
   - **Category**: `Chat Bot` or `Application Integration`
4. Click **Create** and then click **Manage** on your new app to copy the **Client ID** and generate a **Client Secret**.
5. Go to your **Supabase Dashboard** -> **Authentication** -> **Providers** -> **Twitch**.
6. Turn it **ON**, paste the Client ID and Client Secret, and hit **Save**.

---

## 2. YouTube (Google) Setup

YouTube authentication uses Google's OAuth system. 

1. Go to the [Google Cloud Console](https://console.cloud.google.com/).
2. Create a new Project (e.g., "StreamSync").
3. Navigate to **APIs & Services** > **Library** and search for **YouTube Data API v3**. Click **Enable**.
4. Go to **APIs & Services** > **OAuth consent screen**:
   - Choose **External**.
   - Fill in your app name, support email, and developer contact.
   - On the **Scopes** page, add `https://www.googleapis.com/auth/youtube.readonly` so we can read the live chat.
5. Go to **APIs & Services** > **Credentials**:
   - Click **Create Credentials** > **OAuth client ID**.
   - **Application type**: `Web application`
   - **Authorized redirect URIs**: Enter `https://<your-project-id>.supabase.co/auth/v1/callback` (found in Supabase Auth > Providers > Google).
6. Copy the **Client ID** and **Client Secret**.
7. Go to your **Supabase Dashboard** -> **Authentication** -> **Providers** -> **Google**.
8. Turn it **ON**, paste the Client ID and Client Secret, and hit **Save**.

*(Note: While testing, you will need to add your own Google email to the "Test users" list in the OAuth consent screen if the app is unpublished).*

---

## 3. Kick Setup

**Important Note:** Currently, Supabase does not have a native "Kick" OAuth provider built-in, and Kick's public developer API is still quite locked down compared to Twitch and YouTube.

Because of this, clicking the **"Connect Kick"** button in the dashboard will likely throw an error right now. 

To fully support Kick in the future, we have two options:
1. **Custom Auth Flow**: We build a custom OAuth flow in our Next.js backend instead of relying on Supabase's native auth providers.
2. **Workaround**: We can change the Kick integration to ask the user to just type in their Kick username instead of doing a full OAuth login (since Kick chat can often be read anonymously via websockets).
