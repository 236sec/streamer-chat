use crate::{
    application::viewer_count::{
        PlatformViewerCounts, ViewerCountFuture, ViewerCountRepository, ViewerCredentials,
    },
    infrastructure::db,
};
use sqlx::PgPool;
use uuid::Uuid;

pub struct PostgresViewerCountRepository<'a> {
    pub pool: &'a PgPool,
    pub master_key: &'a str,
}

impl ViewerCountRepository for PostgresViewerCountRepository<'_> {
    fn canonical_widget_id(
        &self,
        addressed: Uuid,
    ) -> ViewerCountFuture<'_, Result<Option<Uuid>, String>> {
        Box::pin(async move {
            db::canonical_widget_id(self.pool, &addressed)
                .await
                .map_err(|_| "widget lookup failed".to_owned())
        })
    }

    fn credentials(
        &self,
        canonical: Uuid,
    ) -> ViewerCountFuture<'_, Result<ViewerCredentials, String>> {
        Box::pin(async move {
            let encrypted = db::fetch_tokens(self.pool, &canonical)
                .await
                .map_err(|_| "credential lookup failed".to_owned())?;
            let decrypt =
                |value: Option<String>| value.map(|v| db::decrypt_token(&v, self.master_key));
            Ok(ViewerCredentials {
                twitch: decrypt(encrypted.twitch),
                youtube: decrypt(encrypted.youtube),
                youtube_refresh: decrypt(encrypted.youtube_refresh),
                kick: decrypt(encrypted.kick),
            })
        })
    }
}

pub struct HttpPlatformViewerCounts {
    client: reqwest::Client,
    twitch_base: String,
    twitch_client_id: String,
    youtube_base: String,
    youtube_token_url: String,
    youtube_client_id: String,
    youtube_client_secret: String,
    kick_base: String,
}

impl HttpPlatformViewerCounts {
    pub fn from_env() -> Result<Self, reqwest::Error> {
        Ok(Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(4))
                .build()?,
            twitch_base: std::env::var("TWITCH_HELIX_BASE_URL")
                .unwrap_or_else(|_| "https://api.twitch.tv".into()),
            twitch_client_id: std::env::var("TWITCH_CLIENT_ID").unwrap_or_default(),
            youtube_base: std::env::var("YOUTUBE_API_BASE_URL")
                .unwrap_or_else(|_| "https://www.googleapis.com/youtube/v3".into()),
            youtube_token_url: std::env::var("YOUTUBE_TOKEN_URL")
                .unwrap_or_else(|_| "https://oauth2.googleapis.com/token".into()),
            youtube_client_id: std::env::var("YOUTUBE_CLIENT_ID")
                .or_else(|_| std::env::var("GOOGLE_CLIENT_ID"))
                .unwrap_or_default(),
            youtube_client_secret: std::env::var("YOUTUBE_CLIENT_SECRET")
                .or_else(|_| std::env::var("GOOGLE_CLIENT_SECRET"))
                .unwrap_or_default(),
            kick_base: std::env::var("KICK_API_BASE_URL")
                .unwrap_or_else(|_| "https://kick.com".into()),
        })
    }

    #[cfg(test)]
    fn with_bases(twitch_base: String, youtube_base: String, kick_base: String) -> Self {
        let youtube_token_url = format!("{youtube_base}/token");
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(4))
                .build()
                .unwrap(),
            twitch_base,
            twitch_client_id: "test-client".into(),
            youtube_base,
            youtube_token_url,
            youtube_client_id: "test-client".into(),
            youtube_client_secret: "test-secret".into(),
            kick_base,
        }
    }

    async fn twitch_count(&self, token: Option<String>) -> Result<u64, String> {
        let Some(token) = token else { return Ok(0) };
        if self.twitch_client_id.is_empty() {
            return Ok(0);
        }
        let user: serde_json::Value = self
            .client
            .get(format!("{}/helix/users", self.twitch_base))
            .bearer_auth(&token)
            .header("Client-Id", &self.twitch_client_id)
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|_| "Twitch user request failed".to_owned())?
            .json()
            .await
            .map_err(|_| "Twitch user response was malformed".to_owned())?;
        let Some(user_id) = user.pointer("/data/0/id").and_then(|v| v.as_str()) else {
            return Ok(0);
        };
        let streams: serde_json::Value = self
            .client
            .get(format!("{}/helix/streams", self.twitch_base))
            .query(&[("user_id", user_id)])
            .bearer_auth(token)
            .header("Client-Id", &self.twitch_client_id)
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|_| "Twitch stream request failed".to_owned())?
            .json()
            .await
            .map_err(|_| "Twitch stream response was malformed".to_owned())?;
        Ok(count_from_twitch(&streams))
    }

    async fn youtube_count(
        &self,
        token: Option<String>,
        refresh: Option<String>,
    ) -> Result<u64, String> {
        let Some(mut token) = token else { return Ok(0) };
        let request_broadcasts = |token: &str| {
            self.client
                .get(format!("{}/liveBroadcasts", self.youtube_base))
                .query(&[("part", "id,status"), ("broadcastStatus", "active")])
                .bearer_auth(token)
        };
        let response = request_broadcasts(&token)
            .send()
            .await
            .map_err(|_| "YouTube broadcast request failed".to_owned())?;
        let broadcasts = if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            let Some(refresh) = refresh else { return Ok(0) };
            let refreshed: serde_json::Value = self
                .client
                .post(&self.youtube_token_url)
                .form(&[
                    ("grant_type", "refresh_token"),
                    ("refresh_token", refresh.as_str()),
                    ("client_id", self.youtube_client_id.as_str()),
                    ("client_secret", self.youtube_client_secret.as_str()),
                ])
                .send()
                .await
                .and_then(reqwest::Response::error_for_status)
                .map_err(|_| "YouTube token refresh failed".to_owned())?
                .json()
                .await
                .map_err(|_| "YouTube token refresh response was malformed".to_owned())?;
            let Some(new_token) = refreshed.get("access_token").and_then(|v| v.as_str()) else {
                return Ok(0);
            };
            token = new_token.to_owned();
            request_broadcasts(&token)
                .send()
                .await
                .and_then(reqwest::Response::error_for_status)
                .map_err(|_| "YouTube broadcast request failed".to_owned())?
                .json::<serde_json::Value>()
                .await
                .map_err(|_| "YouTube broadcast response was malformed".to_owned())?
        } else {
            response
                .error_for_status()
                .map_err(|_| "YouTube broadcast request failed".to_owned())?
                .json::<serde_json::Value>()
                .await
                .map_err(|_| "YouTube broadcast response was malformed".to_owned())?
        };
        let Some(broadcast_id) = broadcasts
            .pointer("/items/0/id")
            .and_then(|value| value.as_str())
        else {
            return Ok(0);
        };
        let videos: serde_json::Value = self
            .client
            .get(format!("{}/videos", self.youtube_base))
            .query(&[("part", "liveStreamingDetails"), ("id", broadcast_id)])
            .bearer_auth(token)
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|_| "YouTube video request failed".to_owned())?
            .json()
            .await
            .map_err(|_| "YouTube video response was malformed".to_owned())?;
        Ok(count_from_youtube(&videos))
    }

    async fn kick_count(&self, username: Option<String>) -> Result<u64, String> {
        let Some(username) = username else {
            return Ok(0);
        };
        let value: serde_json::Value = self
            .client
            .get(format!("{}/api/v1/channels/{username}", self.kick_base))
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|_| "Kick channel request failed".to_owned())?
            .json()
            .await
            .map_err(|_| "Kick channel response was malformed".to_owned())?;
        Ok(count_from_kick(&value))
    }
}

impl PlatformViewerCounts for HttpPlatformViewerCounts {
    fn twitch(&self, credentials: ViewerCredentials) -> ViewerCountFuture<'_, Result<u64, String>> {
        Box::pin(self.twitch_count(credentials.twitch))
    }
    fn youtube(
        &self,
        credentials: ViewerCredentials,
    ) -> ViewerCountFuture<'_, Result<u64, String>> {
        Box::pin(self.youtube_count(credentials.youtube, credentials.youtube_refresh))
    }
    fn kick(&self, credentials: ViewerCredentials) -> ViewerCountFuture<'_, Result<u64, String>> {
        Box::pin(self.kick_count(credentials.kick))
    }
}

fn count_from_twitch(value: &serde_json::Value) -> u64 {
    value
        .pointer("/data/0/viewer_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
}

fn count_from_youtube(value: &serde_json::Value) -> u64 {
    value
        .pointer("/items/0/liveStreamingDetails/concurrentViewers")
        .and_then(|v| v.as_str())
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}

fn count_from_kick(value: &serde_json::Value) -> u64 {
    value
        .pointer("/livestream/viewer_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::HttpPlatformViewerCounts;
    use crate::application::viewer_count::{PlatformViewerCounts, ViewerCredentials};
    use axum::{
        extract::State,
        http::{HeaderMap, StatusCode},
        response::IntoResponse,
        routing::{get, post},
        Json, Router,
    };
    use serde_json::{json, Value};
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    use tokio::net::TcpListener;

    #[derive(Clone, Default)]
    struct FixtureState {
        refreshes: Arc<AtomicUsize>,
    }

    async fn twitch_users() -> Json<Value> {
        Json(json!({"data":[{"id":"twitch-id"}]}))
    }
    async fn twitch_streams() -> Json<Value> {
        Json(json!({"data":[{"viewer_count":12}]}))
    }
    async fn broadcasts(headers: HeaderMap) -> impl IntoResponse {
        if headers.get("authorization").and_then(|v| v.to_str().ok()) == Some("Bearer expired") {
            (StatusCode::UNAUTHORIZED, Json(json!({"error":"expired"})))
        } else {
            (
                StatusCode::OK,
                Json(json!({"items":[{"id":"broadcast-id"}]})),
            )
        }
    }
    async fn refresh(State(state): State<FixtureState>) -> Json<Value> {
        state.refreshes.fetch_add(1, Ordering::SeqCst);
        Json(json!({"access_token":"fresh"}))
    }
    async fn videos(headers: HeaderMap) -> Json<Value> {
        assert_eq!(
            headers.get("authorization").and_then(|v| v.to_str().ok()),
            Some("Bearer fresh")
        );
        Json(json!({"items":[{"liveStreamingDetails":{"concurrentViewers":"24"}}]}))
    }
    async fn kick_live() -> Json<Value> {
        Json(json!({"livestream":{"is_live":true,"viewer_count":7,"viewers":7}}))
    }
    async fn kick_offline() -> Json<Value> {
        Json(json!({"current_livestream":null,"recent_livestream":{"viewer_count":null}}))
    }

    #[tokio::test]
    async fn platform_adapters_parse_fixtures_and_refresh_youtube_tokens() {
        let fixture = FixtureState::default();
        let app = Router::new()
            .route("/twitch/helix/users", get(twitch_users))
            .route("/twitch/helix/streams", get(twitch_streams))
            .route("/youtube/liveBroadcasts", get(broadcasts))
            .route("/youtube/token", post(refresh))
            .route("/youtube/videos", get(videos))
            .route("/kick/api/v1/channels/live", get(kick_live))
            .route("/kick/api/v1/channels/offline", get(kick_offline))
            .with_state(fixture.clone());
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let base = format!("http://{address}");
        let adapter = HttpPlatformViewerCounts::with_bases(
            format!("{base}/twitch"),
            format!("{base}/youtube"),
            format!("{base}/kick"),
        );
        let live_credentials = ViewerCredentials {
            twitch: Some("twitch-token".into()),
            youtube: Some("expired".into()),
            youtube_refresh: Some("refresh".into()),
            kick: Some("live".into()),
        };
        let (twitch, youtube, kick_live_count) = tokio::join!(
            adapter.twitch(live_credentials.clone()),
            adapter.youtube(live_credentials.clone()),
            adapter.kick(live_credentials),
        );
        assert_eq!(twitch.unwrap(), 12);
        assert_eq!(youtube.unwrap(), 24);
        assert_eq!(kick_live_count.unwrap(), 7);
        assert_eq!(fixture.refreshes.load(Ordering::SeqCst), 1);
        assert_eq!(
            adapter
                .kick(ViewerCredentials {
                    kick: Some("offline".into()),
                    ..Default::default()
                })
                .await
                .unwrap(),
            0
        );
    }
}
