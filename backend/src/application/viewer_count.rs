use serde::Serialize;
use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

pub type ViewerCountFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Clone, Debug, Default)]
pub struct ViewerCredentials {
    pub twitch: Option<String>,
    pub youtube: Option<String>,
    pub youtube_refresh: Option<String>,
    pub kick: Option<String>,
}

pub trait ViewerCountRepository: Send + Sync {
    fn canonical_widget_id(
        &self,
        addressed: Uuid,
    ) -> ViewerCountFuture<'_, Result<Option<Uuid>, String>>;
    fn credentials(
        &self,
        canonical: Uuid,
    ) -> ViewerCountFuture<'_, Result<ViewerCredentials, String>>;
}

pub trait PlatformViewerCounts: Send + Sync {
    fn twitch(&self, credentials: ViewerCredentials) -> ViewerCountFuture<'_, Result<u64, String>>;
    fn youtube(&self, credentials: ViewerCredentials)
        -> ViewerCountFuture<'_, Result<u64, String>>;
    fn kick(&self, credentials: ViewerCredentials) -> ViewerCountFuture<'_, Result<u64, String>>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewerCountError {
    Unavailable,
    NotFound,
    Timeout,
}

pub struct ViewerCountService {
    refresh_seconds: u64,
    platform_timeout: Duration,
    operation_timeout: Duration,
    cache: RwLock<HashMap<Uuid, (ViewerCountResponse, Instant)>>,
    locks: Mutex<HashMap<Uuid, Arc<Mutex<()>>>>,
}

impl ViewerCountService {
    pub fn new(refresh_seconds: u64, platform_timeout: Duration) -> Self {
        Self {
            refresh_seconds,
            platform_timeout,
            operation_timeout: Duration::from_millis(8500),
            cache: RwLock::new(HashMap::new()),
            locks: Mutex::new(HashMap::new()),
        }
    }

    pub async fn read<R: ViewerCountRepository, P: PlatformViewerCounts>(
        &self,
        repository: &R,
        platforms: &P,
        addressed: Uuid,
    ) -> Result<ViewerCountResponse, ViewerCountError> {
        tokio::time::timeout(
            self.operation_timeout,
            self.read_inner(repository, platforms, addressed),
        )
        .await
        .map_err(|_| ViewerCountError::Timeout)?
    }

    async fn read_inner<R: ViewerCountRepository, P: PlatformViewerCounts>(
        &self,
        repository: &R,
        platforms: &P,
        addressed: Uuid,
    ) -> Result<ViewerCountResponse, ViewerCountError> {
        let canonical = repository
            .canonical_widget_id(addressed)
            .await
            .map_err(|_| ViewerCountError::Unavailable)?
            .ok_or(ViewerCountError::NotFound)?;
        if let Some(response) = self.cached(canonical).await {
            return Ok(response);
        }
        let lock = {
            let mut locks = self.locks.lock().await;
            locks
                .entry(canonical)
                .or_insert_with(|| Arc::new(Mutex::new(())))
                .clone()
        };
        let _guard = tokio::time::timeout(self.operation_timeout, lock.lock())
            .await
            .map_err(|_| ViewerCountError::Timeout)?;
        if let Some(response) = self.cached(canonical).await {
            return Ok(response);
        }
        let credentials = repository
            .credentials(canonical)
            .await
            .map_err(|_| ViewerCountError::Unavailable)?;
        let (twitch, youtube, kick) = tokio::join!(
            bounded_count(self.platform_timeout, platforms.twitch(credentials.clone())),
            bounded_count(
                self.platform_timeout,
                platforms.youtube(credentials.clone())
            ),
            bounded_count(self.platform_timeout, platforms.kick(credentials)),
        );
        let response = ViewerCountResponse {
            counts: ViewerCounts {
                twitch,
                youtube,
                kick,
            },
            refresh_seconds: self.refresh_seconds,
        };
        self.cache.write().await.insert(
            canonical,
            (
                response.clone(),
                Instant::now() + Duration::from_secs(self.refresh_seconds),
            ),
        );
        Ok(response)
    }

    async fn cached(&self, id: Uuid) -> Option<ViewerCountResponse> {
        self.cache
            .read()
            .await
            .get(&id)
            .and_then(|(value, expires)| (*expires > Instant::now()).then(|| value.clone()))
    }
}

async fn bounded_count(
    future_timeout: Duration,
    future: ViewerCountFuture<'_, Result<u64, String>>,
) -> u64 {
    tokio::time::timeout(future_timeout, future)
        .await
        .ok()
        .and_then(Result::ok)
        .unwrap_or(0)
}

#[derive(Clone, Debug, Default, Serialize, PartialEq, Eq)]
pub struct ViewerCounts {
    pub twitch: u64,
    pub youtube: u64,
    pub kick: u64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ViewerCountResponse {
    pub counts: ViewerCounts,
    pub refresh_seconds: u64,
}

pub fn refresh_seconds_from(value: Option<&str>) -> u64 {
    value
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|v| (30..=300).contains(v))
        .unwrap_or(60)
}

#[cfg(test)]
mod tests {
    use super::{
        refresh_seconds_from, PlatformViewerCounts, ViewerCountFuture, ViewerCountRepository,
        ViewerCountService, ViewerCredentials,
    };
    use std::{
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        },
        time::Duration,
    };
    use uuid::Uuid;

    struct FakeRepository {
        canonical: Option<Uuid>,
    }
    impl ViewerCountRepository for FakeRepository {
        fn canonical_widget_id(
            &self,
            _: Uuid,
        ) -> ViewerCountFuture<'_, Result<Option<Uuid>, String>> {
            let id = self.canonical;
            Box::pin(async move { Ok(id) })
        }
        fn credentials(&self, _: Uuid) -> ViewerCountFuture<'_, Result<ViewerCredentials, String>> {
            Box::pin(async { Ok(ViewerCredentials::default()) })
        }
    }

    struct FakePlatforms {
        twitch_fetches: AtomicUsize,
        kick_fetches: AtomicUsize,
    }
    impl PlatformViewerCounts for FakePlatforms {
        fn twitch(&self, _: ViewerCredentials) -> ViewerCountFuture<'_, Result<u64, String>> {
            self.twitch_fetches.fetch_add(1, Ordering::SeqCst);
            Box::pin(async {
                tokio::time::sleep(Duration::from_millis(20)).await;
                Ok(9)
            })
        }
        fn youtube(&self, _: ViewerCredentials) -> ViewerCountFuture<'_, Result<u64, String>> {
            Box::pin(async { Err("offline".into()) })
        }
        fn kick(&self, _: ViewerCredentials) -> ViewerCountFuture<'_, Result<u64, String>> {
            self.kick_fetches.fetch_add(1, Ordering::SeqCst);
            Box::pin(async { Ok(4) })
        }
    }

    #[test]
    fn refresh_interval_uses_configured_range_and_safe_default() {
        assert_eq!(refresh_seconds_from(None), 60);
        assert_eq!(refresh_seconds_from(Some("30")), 30);
        assert_eq!(refresh_seconds_from(Some("300")), 300);
        assert_eq!(refresh_seconds_from(Some("29")), 60);
        assert_eq!(refresh_seconds_from(Some("301")), 60);
        assert_eq!(refresh_seconds_from(Some("bad")), 60);
    }

    #[tokio::test]
    async fn platform_failure_defaults_only_its_count_and_concurrent_reads_coalesce() {
        let id = Uuid::new_v4();
        let alias = Uuid::new_v4();
        let repository = FakeRepository {
            canonical: Some(id),
        };
        let platforms = Arc::new(FakePlatforms {
            twitch_fetches: AtomicUsize::new(0),
            kick_fetches: AtomicUsize::new(0),
        });
        let service = Arc::new(ViewerCountService::new(60, Duration::from_secs(1)));
        let (first, second) = tokio::join!(
            service.read(&repository, platforms.as_ref(), alias),
            service.read(&repository, platforms.as_ref(), id)
        );
        assert_eq!(
            first.unwrap().counts,
            super::ViewerCounts {
                twitch: 9,
                youtube: 0,
                kick: 4
            }
        );
        assert_eq!(
            second.unwrap().counts,
            super::ViewerCounts {
                twitch: 9,
                youtube: 0,
                kick: 4
            }
        );
        assert_eq!(platforms.twitch_fetches.load(Ordering::SeqCst), 1);
        assert_eq!(platforms.kick_fetches.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn a_slow_platform_does_not_discard_other_successes() {
        let id = Uuid::new_v4();
        let repository = FakeRepository {
            canonical: Some(id),
        };
        let platforms = FakePlatforms {
            twitch_fetches: AtomicUsize::new(0),
            kick_fetches: AtomicUsize::new(0),
        };
        let service = ViewerCountService::new(60, Duration::from_millis(5));
        let result = service.read(&repository, &platforms, id).await.unwrap();
        assert_eq!(
            result.counts,
            super::ViewerCounts {
                twitch: 0,
                youtube: 0,
                kick: 4
            }
        );
    }

    #[tokio::test]
    async fn cached_response_retains_interval_and_expiry_fetches_again() {
        let id = Uuid::new_v4();
        let repository = FakeRepository {
            canonical: Some(id),
        };
        let platforms = FakePlatforms {
            twitch_fetches: AtomicUsize::new(0),
            kick_fetches: AtomicUsize::new(0),
        };
        let service = ViewerCountService::new(1, Duration::from_secs(1));
        assert_eq!(
            service
                .read(&repository, &platforms, id)
                .await
                .unwrap()
                .refresh_seconds,
            1
        );
        assert_eq!(
            service
                .read(&repository, &platforms, id)
                .await
                .unwrap()
                .refresh_seconds,
            1
        );
        assert_eq!(platforms.twitch_fetches.load(Ordering::SeqCst), 1);
        tokio::time::sleep(Duration::from_millis(1050)).await;
        service.read(&repository, &platforms, id).await.unwrap();
        assert_eq!(platforms.twitch_fetches.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn unknown_widget_is_rejected_without_loading_credentials() {
        let repository = FakeRepository { canonical: None };
        let platforms = FakePlatforms {
            twitch_fetches: AtomicUsize::new(0),
            kick_fetches: AtomicUsize::new(0),
        };
        let service = ViewerCountService::new(60, Duration::from_secs(1));
        assert_eq!(
            service.read(&repository, &platforms, Uuid::new_v4()).await,
            Err(super::ViewerCountError::NotFound)
        );
        assert_eq!(platforms.twitch_fetches.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn public_response_serializes_only_counts_and_refresh_metadata() {
        let response = super::ViewerCountResponse {
            counts: super::ViewerCounts {
                twitch: 1,
                youtube: 2,
                kick: 3,
            },
            refresh_seconds: 45,
        };
        assert_eq!(
            serde_json::to_value(response).unwrap(),
            serde_json::json!({
                "counts": {"twitch": 1, "youtube": 2, "kick": 3},
                "refresh_seconds": 45
            })
        );
    }
}
