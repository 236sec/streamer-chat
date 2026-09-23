use std::time::Duration;
use tokio::sync::broadcast;

fn url_encode(input: &str) -> String {
    let mut encoded = String::new();
    for byte in input.bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            b' ' => encoded.push('+'),
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    encoded
}

async fn refresh_access_token(
    client: &reqwest::Client,
    refresh_token: &str,
) -> Result<String, String> {
    let token_url = std::env::var("YOUTUBE_TOKEN_URL")
        .unwrap_or_else(|_| "https://oauth2.googleapis.com/token".to_string());
    let client_id = std::env::var("YOUTUBE_CLIENT_ID")
        .or_else(|_| std::env::var("GOOGLE_CLIENT_ID"))
        .unwrap_or_default();
    let client_secret = std::env::var("YOUTUBE_CLIENT_SECRET")
        .or_else(|_| std::env::var("GOOGLE_CLIENT_SECRET"))
        .unwrap_or_default();

    let mut body = format!(
        "grant_type=refresh_token&refresh_token={}",
        url_encode(refresh_token)
    );
    if !client_id.is_empty() {
        body.push_str(&format!("&client_id={}", url_encode(&client_id)));
    }
    if !client_secret.is_empty() {
        body.push_str(&format!("&client_secret={}", url_encode(&client_secret)));
    }

    let res = client
        .post(&token_url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .map_err(|e| format!("Token refresh request failed: {}", e))?;

    let status = res.status();
    if !status.is_success() {
        let body = res
            .text()
            .await
            .unwrap_or_else(|_| "<unreadable>".to_string());
        return Err(format!("Token refresh returned {}: {}", status, body));
    }

    let json: serde_json::Value = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse token refresh response: {}", e))?;

    json.get("access_token")
        .and_then(|t| t.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "No access_token found in refresh response".to_string())
}

async fn sleep_or_cancel(duration: Duration, tx: &broadcast::Sender<String>) -> bool {
    let mut elapsed = Duration::ZERO;
    let step = Duration::from_millis(100);
    let mut zero_receivers_count = 0;
    while elapsed < duration {
        if tx.receiver_count() == 0 {
            zero_receivers_count += 1;
            // Allow up to 10 consecutive ticks (1 second) of 0 receivers
            // so brief reconnection / page reload does not immediately terminate workers
            if zero_receivers_count >= 10 {
                return false;
            }
        } else {
            zero_receivers_count = 0;
        }
        let current_step = step.min(duration - elapsed);
        tokio::time::sleep(current_step).await;
        elapsed += current_step;
    }
    tx.receiver_count() > 0 || zero_receivers_count < 10
}

pub async fn spawn_youtube_client(
    widget_id: String,
    token: String,
    refresh_token: Option<String>,
    tx: broadcast::Sender<String>,
) {
    let base_url = std::env::var("YOUTUBE_API_BASE_URL")
        .unwrap_or_else(|_| "https://www.googleapis.com/youtube/v3".to_string());
    let discovery_interval_secs: u64 = std::env::var("YOUTUBE_DISCOVERY_INTERVAL_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30);
    let discovery_interval = Duration::from_secs(discovery_interval_secs);

    let client = reqwest::Client::new();
    let mut current_token = token;

    println!("[YouTube] Starting YouTube client for widget {}", widget_id);

    'outer: loop {
        if tx.receiver_count() == 0 {
            break 'outer;
        }

        // 1. Broadcast discovery loop
        let broadcasts_url = format!(
            "{}/liveBroadcasts?part=snippet,status&broadcastStatus=active",
            base_url
        );

        let (broadcast_id, mut live_chat_id) = loop {
            if tx.receiver_count() == 0 {
                break 'outer;
            }

            let res = client
                .get(&broadcasts_url)
                .bearer_auth(&current_token)
                .send()
                .await;

            match res {
                Ok(response) => {
                    let status = response.status();
                    if status == reqwest::StatusCode::UNAUTHORIZED {
                        if let Some(ref ref_tok) = refresh_token {
                            println!("[YouTube] Access token expired (401), attempting refresh...");
                            match refresh_access_token(&client, ref_tok).await {
                                Ok(new_token) => {
                                    println!("[YouTube] Token refreshed successfully");
                                    current_token = new_token;
                                    continue;
                                }
                                Err(e) => {
                                    eprintln!("[YouTube] Failed to refresh token: {}", e);
                                    break 'outer;
                                }
                            }
                        } else {
                            eprintln!("[YouTube] 401 Unauthorized and no refresh token available");
                            break 'outer;
                        }
                    } else if status.is_success() {
                        if let Ok(json) = response.json::<serde_json::Value>().await {
                            if let Some(items) =
                                json.get("items").and_then(|items| items.as_array())
                            {
                                // Prefer broadcast that has lifeCycleStatus == "live", fallback to first
                                let active_item = items
                                    .iter()
                                    .find(|item| {
                                        item.get("status")
                                            .and_then(|s| s.get("lifeCycleStatus"))
                                            .and_then(|l| l.as_str())
                                            == Some("live")
                                    })
                                    .or_else(|| items.first());

                                if let Some(item) = active_item {
                                    let b_id = item
                                        .get("id")
                                        .and_then(|id| id.as_str())
                                        .map(|s| s.to_string());
                                    let mut chat_id = item
                                        .get("snippet")
                                        .and_then(|snippet| snippet.get("liveChatId"))
                                        .and_then(|id| id.as_str())
                                        .map(|s| s.to_string());

                                    // If liveChatId is missing from snippet, attempt videos.list fallback
                                    if chat_id.is_none() {
                                        if let Some(ref vid) = b_id {
                                            let vid_url = format!(
                                                "{}/videos?part=liveStreamingDetails&id={}",
                                                base_url, vid
                                            );
                                            if let Ok(vid_res) = client
                                                .get(&vid_url)
                                                .bearer_auth(&current_token)
                                                .send()
                                                .await
                                            {
                                                if let Ok(vid_json) =
                                                    vid_res.json::<serde_json::Value>().await
                                                {
                                                    chat_id = vid_json
                                                        .get("items")
                                                        .and_then(|arr| arr.as_array())
                                                        .and_then(|arr| arr.first())
                                                        .and_then(|v| v.get("liveStreamingDetails"))
                                                        .and_then(|lsd| lsd.get("activeLiveChatId"))
                                                        .and_then(|id| id.as_str())
                                                        .map(|s| s.to_string());
                                                }
                                            }
                                        }
                                    }

                                    if let Some(id) = chat_id {
                                        println!(
                                            "[YouTube] Found active broadcast with liveChatId: {}",
                                            id
                                        );
                                        break (b_id, id);
                                    }
                                }
                            }
                        }
                    } else {
                        eprintln!("[YouTube] Discovery request returned status: {}", status);
                    }
                }
                Err(e) => {
                    eprintln!("[YouTube] Discovery request error: {}", e);
                }
            }

            if !sleep_or_cancel(discovery_interval, &tx).await {
                break 'outer;
            }
        };

        // 2. Chat polling loop
        let mut page_token: Option<String> = None;
        let mut not_found_retries = 0;
        loop {
            if tx.receiver_count() == 0 {
                break 'outer;
            }

            let mut messages_url = format!(
                "{}/liveChat/messages?liveChatId={}&part=snippet,authorDetails",
                base_url, live_chat_id
            );
            if let Some(ref pt) = page_token {
                messages_url.push_str(&format!("&pageToken={}", pt));
            }

            let res = client
                .get(&messages_url)
                .bearer_auth(&current_token)
                .send()
                .await;

            let mut next_poll_millis: u64 = 5000;

            match res {
                Ok(response) => {
                    let status = response.status();
                    if status == reqwest::StatusCode::UNAUTHORIZED {
                        if let Some(ref ref_tok) = refresh_token {
                            println!("[YouTube] Access token expired (401) during chat poll, attempting refresh...");
                            match refresh_access_token(&client, ref_tok).await {
                                Ok(new_token) => {
                                    println!("[YouTube] Token refreshed successfully");
                                    current_token = new_token;
                                    continue;
                                }
                                Err(e) => {
                                    eprintln!("[YouTube] Failed to refresh token: {}", e);
                                    break 'outer;
                                }
                            }
                        } else {
                            eprintln!("[YouTube] 401 Unauthorized during chat poll and no refresh token available");
                            break 'outer;
                        }
                    } else if status == reqwest::StatusCode::NOT_FOUND {
                        let body = response.text().await.unwrap_or_default();
                        eprintln!(
                            "[YouTube] Live chat returned 404 for {}: {}",
                            live_chat_id, body
                        );

                        // Try activeLiveChatId fallback from video resource
                        if not_found_retries == 0 {
                            if let Some(ref vid) = broadcast_id {
                                let vid_url = format!(
                                    "{}/videos?part=liveStreamingDetails&id={}",
                                    base_url, vid
                                );
                                if let Ok(vid_res) = client
                                    .get(&vid_url)
                                    .bearer_auth(&current_token)
                                    .send()
                                    .await
                                {
                                    if let Ok(vid_json) = vid_res.json::<serde_json::Value>().await
                                    {
                                        if let Some(new_id) = vid_json
                                            .get("items")
                                            .and_then(|arr| arr.as_array())
                                            .and_then(|arr| arr.first())
                                            .and_then(|v| v.get("liveStreamingDetails"))
                                            .and_then(|lsd| lsd.get("activeLiveChatId"))
                                            .and_then(|id| id.as_str())
                                        {
                                            if new_id != live_chat_id {
                                                println!("[YouTube] Switched to activeLiveChatId from video details: {}", new_id);
                                                live_chat_id = new_id.to_string();
                                                page_token = None;
                                                continue;
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        not_found_retries += 1;
                        if not_found_retries <= 5 {
                            println!(
                                "[YouTube] Live chat initializing or transient 404, retrying in 2s (attempt {}/5)...",
                                not_found_retries
                            );
                            if !sleep_or_cancel(Duration::from_secs(2), &tx).await {
                                break 'outer;
                            }
                            continue;
                        }

                        println!(
                            "[YouTube] Broadcast or chat not found (ended), returning to discovery"
                        );
                        break;
                    } else if status == reqwest::StatusCode::BAD_REQUEST {
                        let body = response.text().await.unwrap_or_default();
                        eprintln!("[YouTube] Chat polling returned 400 Bad Request: {}", body);
                        if page_token.is_some() {
                            println!("[YouTube] Resetting pageToken after 400 Bad Request");
                            page_token = None;
                            continue;
                        }
                        break;
                    } else if status.is_success() {
                        not_found_retries = 0;
                        if let Ok(json) = response.json::<serde_json::Value>().await {
                            if let Some(interval) =
                                json.get("pollingIntervalMillis").and_then(|v| v.as_u64())
                            {
                                next_poll_millis = interval.max(100);
                            }

                            if let Some(next_pt) =
                                json.get("nextPageToken").and_then(|v| v.as_str())
                            {
                                page_token = Some(next_pt.to_string());
                            }

                            if let Some(items) = json.get("items").and_then(|v| v.as_array()) {
                                for item in items {
                                    if let Some(chat_msg) =
                                        crate::domain::normalize::normalize_message(
                                            "youtube", &widget_id, item,
                                        )
                                    {
                                        if let Ok(msg_json) = serde_json::to_string(&chat_msg) {
                                            if let Err(e) = tx.send(msg_json) {
                                                eprintln!(
                                                    "[YouTube] Failed to broadcast message: {}",
                                                    e
                                                );
                                                break 'outer;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        eprintln!("[YouTube] Chat polling returned status: {}", status);
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("[YouTube] Chat polling network error: {}", e);
                }
            }

            if !sleep_or_cancel(Duration::from_millis(next_poll_millis), &tx).await {
                break 'outer;
            }
        }

        if !sleep_or_cancel(discovery_interval, &tx).await {
            break 'outer;
        }
    }

    println!(
        "[YouTube] YouTube client terminated for widget {}",
        widget_id
    );
}
