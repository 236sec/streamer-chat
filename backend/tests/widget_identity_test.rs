use sqlx::{PgPool, Row};
use uuid::Uuid;

async fn create_account(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO auth.users (id, instance_id, aud, role) VALUES ($1, $2, 'authenticated', 'authenticated')")
        .bind(id)
        .bind(Uuid::nil())
        .execute(pool)
        .await
        .unwrap();
    id
}

async fn create_widget(
    pool: &PgPool,
    owner: Uuid,
    theme: &str,
    pin: Option<serde_json::Value>,
    revision: i64,
) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO public.widgets (id, user_id, theme, pinned_message, pin_revision) VALUES ($1, $2, $3, $4, $5)")
        .bind(id)
        .bind(owner)
        .bind(theme)
        .bind(pin)
        .bind(revision)
        .execute(pool)
        .await
        .unwrap();
    id
}

async fn resolve(
    pool: &PgPool,
    owner: Uuid,
    hint: Option<Uuid>,
) -> (Uuid, Option<serde_json::Value>, i64) {
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SET LOCAL ROLE authenticated")
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query("SELECT set_config('request.jwt.claim.sub', $1, true)")
        .bind(owner.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    let row = sqlx::query(
        "SELECT id, pinned_message, pin_revision FROM public.resolve_account_widget($1)",
    )
    .bind(hint)
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    let result = (
        row.get("id"),
        row.get("pinned_message"),
        row.get("pin_revision"),
    );
    tx.commit().await.unwrap();
    result
}

async fn denied_as_anonymous(pool: &PgPool, statement: &'static str) {
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SET LOCAL ROLE anon")
        .execute(&mut *tx)
        .await
        .unwrap();
    assert!(sqlx::query(statement).fetch_all(&mut *tx).await.is_err());
    tx.rollback().await.unwrap();
}

fn pin_for(widget_id: Uuid, content: &str) -> serde_json::Value {
    serde_json::json!({
        "id": format!("message-{content}"), "type": "chat_message", "widget_id": widget_id,
        "platform": "twitch", "author": "Viewer", "content": content,
        "fragments": [{"type": "text", "text": content}]
    })
}

#[tokio::test]
#[ignore = "requires a disposable, migrated Supabase TEST_DATABASE_URL"]
async fn account_widget_resolution_and_public_access_contract() {
    let url =
        std::env::var("TEST_DATABASE_URL").expect("disposable migrated Supabase database required");
    let pool = PgPool::connect(&url).await.unwrap();
    let owner = create_account(&pool).await;
    let other = create_account(&pool).await;
    let empty = create_account(&pool).await;
    let first = create_widget(&pool, owner, "first", None, 2).await;
    let copied = create_widget(&pool, owner, "copied", None, 0).await;
    let foreign = create_widget(&pool, other, "foreign", None, 0).await;

    // A valid copied-chat hint wins once; later hints cannot rotate the mapping.
    assert_eq!(resolve(&pool, owner, Some(copied)).await.0, copied);
    assert_eq!(resolve(&pool, owner, Some(first)).await.0, copied);
    assert_eq!(resolve(&pool, other, Some(first)).await.0, foreign);
    assert_eq!(
        resolve(&pool, empty, None).await.0,
        resolve(&pool, empty, Some(first)).await.0
    );
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM public.widgets WHERE user_id = $1")
        .bind(owner)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 2);

    let public: Uuid = sqlx::query_scalar("SELECT id FROM public.lookup_public_widget($1)")
        .bind(first)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(public, copied);

    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SET LOCAL ROLE anon")
        .execute(&mut *tx)
        .await
        .unwrap();
    let known: Uuid = sqlx::query_scalar("SELECT id FROM public.lookup_public_widget($1)")
        .bind(first)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    assert_eq!(known, copied);
    let unknown_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM public.lookup_public_widget($1)")
            .bind(Uuid::new_v4())
            .fetch_one(&mut *tx)
            .await
            .unwrap();
    assert_eq!(unknown_count, 0);
    assert!(sqlx::query("SELECT id FROM public.widgets")
        .fetch_all(&mut *tx)
        .await
        .is_err());
    tx.rollback().await.unwrap();
    denied_as_anonymous(&pool, "SELECT widget_id FROM public.widget_identities").await;
    denied_as_anonymous(&pool, "SELECT id FROM public.resolve_account_widget()").await;
    denied_as_anonymous(&pool, "SELECT public.current_account_widget_id()").await;

    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SET LOCAL ROLE authenticated")
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query("SELECT set_config('request.jwt.claim.sub', $1, true)")
        .bind(owner.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    let owner_ids: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM public.widgets")
        .fetch_all(&mut *tx)
        .await
        .unwrap();
    assert_eq!(owner_ids, vec![copied]);
    let legacy_update = sqlx::query("UPDATE public.widgets SET theme = 'blocked' WHERE id = $1")
        .bind(first)
        .execute(&mut *tx)
        .await
        .unwrap();
    assert_eq!(legacy_update.rows_affected(), 0);
    let canonical_update = sqlx::query("UPDATE public.widgets SET theme = 'allowed' WHERE id = $1")
        .bind(copied)
        .execute(&mut *tx)
        .await
        .unwrap();
    assert_eq!(canonical_update.rows_affected(), 1);
    tx.rollback().await.unwrap();

    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SET LOCAL ROLE authenticated")
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query("SELECT set_config('request.jwt.claim.sub', $1, true)")
        .bind(owner.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    assert!(
        sqlx::query("UPDATE public.widgets SET pinned_message = NULL WHERE id = $1")
            .bind(copied)
            .execute(&mut *tx)
            .await
            .is_err()
    );
    tx.rollback().await.unwrap();

    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SET LOCAL ROLE authenticated")
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query("SELECT set_config('request.jwt.claim.sub', $1, true)")
        .bind(owner.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    assert!(
        sqlx::query("UPDATE public.widgets SET pin_revision = 99 WHERE id = $1")
            .bind(copied)
            .execute(&mut *tx)
            .await
            .is_err()
    );
    tx.rollback().await.unwrap();

    for id in [owner, other, empty] {
        sqlx::query("DELETE FROM auth.users WHERE id = $1")
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
    }
}

#[tokio::test]
#[ignore = "requires a disposable, migrated Supabase TEST_DATABASE_URL"]
async fn concurrent_resolution_and_pin_migration_contract() {
    let url =
        std::env::var("TEST_DATABASE_URL").expect("disposable migrated Supabase database required");
    let pool = PgPool::connect(&url).await.unwrap();
    let owner = create_account(&pool).await;
    let first = create_widget(&pool, owner, "first", None, 2).await;
    let second = create_widget(&pool, owner, "second", None, 8).await;
    let pin = serde_json::json!({
        "id": "message-1", "type": "chat_message", "widget_id": second,
        "platform": "twitch", "author": "Viewer", "content": "Pinned",
        "fragments": [{"type": "text", "text": "Pinned"}]
    });
    sqlx::query("UPDATE public.widgets SET pinned_message = $1 WHERE id = $2")
        .bind(pin)
        .bind(second)
        .execute(&pool)
        .await
        .unwrap();
    let (left, right) = tokio::join!(
        resolve(&pool, owner, Some(first)),
        resolve(&pool, owner, Some(second))
    );
    assert_eq!(left.0, right.0);
    let mapped: i64 =
        sqlx::query_scalar("SELECT count(*) FROM public.widget_identities WHERE user_id = $1")
            .bind(owner)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(mapped, 1);
    let canonical = resolve(&pool, owner, None).await;
    assert_eq!(canonical.1.unwrap()["widget_id"], canonical.0.to_string());
    if canonical.0 == first {
        assert!(canonical.2 > 8);
    }
    sqlx::query("DELETE FROM auth.users WHERE id = $1")
        .bind(owner)
        .execute(&pool)
        .await
        .unwrap();
}

#[tokio::test]
#[ignore = "requires a disposable, migrated Supabase TEST_DATABASE_URL"]
async fn deterministic_fallback_migrates_one_valid_legacy_pin() {
    let url =
        std::env::var("TEST_DATABASE_URL").expect("disposable migrated Supabase database required");
    let pool = PgPool::connect(&url).await.unwrap();
    let owner = create_account(&pool).await;
    let a = create_widget(&pool, owner, "keep-a", None, 3).await;
    let b = create_widget(&pool, owner, "keep-b", None, 7).await;
    let (canonical, legacy) = if a < b { (a, b) } else { (b, a) };
    let pin = serde_json::json!({
        "id": "message-2", "type": "chat_message", "widget_id": legacy,
        "platform": "kick", "author": "Viewer", "content": "Keep this",
        "fragments": [{"type": "text", "text": "Keep this"}]
    });
    sqlx::query("UPDATE public.widgets SET pinned_message = $1 WHERE id = $2")
        .bind(&pin)
        .bind(legacy)
        .execute(&pool)
        .await
        .unwrap();
    let resolved = resolve(&pool, owner, None).await;
    assert_eq!(resolved.0, canonical);
    assert_eq!(resolved.1.unwrap()["widget_id"], canonical.to_string());
    assert!(resolved.2 > 7);
    let original: serde_json::Value =
        sqlx::query_scalar("SELECT pinned_message FROM public.widgets WHERE id = $1")
            .bind(legacy)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(original, pin);
    let theme: String = sqlx::query_scalar("SELECT theme FROM public.widgets WHERE id = $1")
        .bind(canonical)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(theme, if canonical == a { "keep-a" } else { "keep-b" });
    sqlx::query("DELETE FROM auth.users WHERE id = $1")
        .bind(owner)
        .execute(&pool)
        .await
        .unwrap();
}

#[tokio::test]
#[ignore = "requires a disposable, migrated Supabase TEST_DATABASE_URL"]
async fn canonical_pin_wins_and_multiple_legacy_pins_choose_stably() {
    let url =
        std::env::var("TEST_DATABASE_URL").expect("disposable migrated Supabase database required");
    let pool = PgPool::connect(&url).await.unwrap();
    let pinned_owner = create_account(&pool).await;
    let canonical = create_widget(&pool, pinned_owner, "canonical", None, 5).await;
    let legacy = create_widget(&pool, pinned_owner, "legacy", None, 9).await;
    let canonical_pin = pin_for(canonical, "canonical stays");
    sqlx::query("UPDATE public.widgets SET pinned_message = $1 WHERE id = $2")
        .bind(&canonical_pin)
        .bind(canonical)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE public.widgets SET pinned_message = $1 WHERE id = $2")
        .bind(pin_for(legacy, "legacy ignored"))
        .bind(legacy)
        .execute(&pool)
        .await
        .unwrap();
    let result = resolve(&pool, pinned_owner, Some(canonical)).await;
    assert_eq!(result.1, Some(canonical_pin));
    assert_eq!(result.2, 5);

    let owner = create_account(&pool).await;
    let canonical = create_widget(&pool, owner, "canonical", None, 1).await;
    let a = create_widget(&pool, owner, "legacy-a", None, 8).await;
    let b = create_widget(&pool, owner, "legacy-b", None, 12).await;
    for (id, content) in [(a, "A"), (b, "B")] {
        sqlx::query("UPDATE public.widgets SET pinned_message = $1 WHERE id = $2")
            .bind(pin_for(id, content))
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
    }
    let result = resolve(&pool, owner, Some(canonical)).await;
    let expected = if a < b { "A" } else { "B" };
    assert_eq!(result.1.unwrap()["content"], expected);
    assert!(result.2 > 12);
    for id in [pinned_owner, owner] {
        sqlx::query("DELETE FROM auth.users WHERE id = $1")
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
    }
}
