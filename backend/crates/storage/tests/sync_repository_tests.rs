use serde_json::json;
use sqlx::{PgPool, Row};
use std::time::Duration;
use uuid::Uuid;

use iqrah_backend_domain::{
    MemoryStateChange, SessionChange, SessionItemChange, SettingChange, SyncChanges,
};
use iqrah_backend_storage::SyncRepository;

#[sqlx::test(migrations = "../../migrations")]
async fn sync_push_then_pull_returns_changes(pool: PgPool) -> Result<(), sqlx::Error> {
    let user_id = Uuid::new_v4();
    let device_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let item_id = Uuid::new_v4();

    sqlx::query("INSERT INTO users (id, oauth_sub) VALUES ($1, $2)")
        .bind(user_id)
        .bind(format!("sub-{}", user_id))
        .execute(&pool)
        .await?;

    let repo = SyncRepository::new(pool.clone());
    repo.touch_device(user_id, device_id, None, None, None)
        .await
        .map_err(|e| sqlx::Error::Protocol(format!("touch_device failed: {e}")))?;

    let changes = SyncChanges {
        settings: vec![SettingChange {
            key: "theme".to_string(),
            value: json!({ "mode": "dark" }),
            client_updated_at: 1_700_000_000_000,
        }],
        memory_states: vec![MemoryStateChange {
            node_id: 42,
            energy: 0.7,
            fsrs_stability: Some(2.5),
            fsrs_difficulty: Some(4.2),
            last_reviewed_at: Some(1_700_000_000_000),
            next_review_at: Some(1_700_000_100_000),
            client_updated_at: 1_700_000_000_000,
        }],
        sessions: vec![SessionChange {
            id: session_id,
            goal_id: Some("goal-1".to_string()),
            started_at: 1_700_000_000_000,
            completed_at: Some(1_700_000_120_000),
            items_completed: 1,
            client_updated_at: 1_700_000_000_000,
        }],
        session_items: vec![SessionItemChange {
            id: item_id,
            session_id,
            node_id: 42,
            exercise_type: "translate".to_string(),
            grade: Some(3),
            duration_ms: Some(1200),
            client_updated_at: 1_700_000_000_000,
        }],
    };

    repo.apply_changes(user_id, device_id, &changes)
        .await
        .map_err(|e| sqlx::Error::Protocol(format!("apply_changes failed: {e}")))?;

    let (pulled, has_more, next_cursor) = repo
        .get_changes_since(user_id, 0, 1000, None)
        .await
        .map_err(|e| sqlx::Error::Protocol(format!("get_changes_since failed: {e}")))?;

    assert_eq!(pulled.settings.len(), 1);
    assert_eq!(pulled.memory_states.len(), 1);
    assert_eq!(pulled.sessions.len(), 1);
    assert_eq!(pulled.session_items.len(), 1);
    assert!(!has_more);
    assert!(next_cursor.is_none());

    let setting = &pulled.settings[0];
    assert_eq!(setting.key, "theme");
    assert_eq!(setting.value, json!({ "mode": "dark" }));

    let state = &pulled.memory_states[0];
    assert_eq!(state.node_id, 42);
    assert!((state.energy - 0.7).abs() < f32::EPSILON);

    let session = &pulled.sessions[0];
    assert_eq!(session.id, session_id);
    assert_eq!(session.goal_id.as_deref(), Some("goal-1"));
    assert_eq!(session.items_completed, 1);

    let item = &pulled.session_items[0];
    assert_eq!(item.id, item_id);
    assert_eq!(item.session_id, session_id);
    assert_eq!(item.exercise_type, "translate");

    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn lww_prefers_newer_updates(pool: PgPool) -> Result<(), sqlx::Error> {
    let user_id = Uuid::new_v4();
    let device_a = Uuid::new_v4();
    let device_b = Uuid::new_v4();

    sqlx::query("INSERT INTO users (id, oauth_sub) VALUES ($1, $2)")
        .bind(user_id)
        .bind(format!("sub-{}", user_id))
        .execute(&pool)
        .await?;

    let repo = SyncRepository::new(pool.clone());
    repo.touch_device(user_id, device_a, None, None, None)
        .await
        .map_err(|e| sqlx::Error::Protocol(format!("touch_device failed: {e}")))?;
    repo.touch_device(user_id, device_b, None, None, None)
        .await
        .map_err(|e| sqlx::Error::Protocol(format!("touch_device failed: {e}")))?;

    let first = SyncChanges {
        settings: vec![SettingChange {
            key: "mode".to_string(),
            value: json!("early"),
            client_updated_at: 1_700_000_000_000,
        }],
        ..SyncChanges::default()
    };
    repo.apply_changes(user_id, device_a, &first)
        .await
        .map_err(|e| sqlx::Error::Protocol(format!("apply_changes failed: {e}")))?;

    tokio::time::sleep(Duration::from_millis(2)).await;

    let second = SyncChanges {
        settings: vec![SettingChange {
            key: "mode".to_string(),
            value: json!("later"),
            client_updated_at: 1_700_000_000_100,
        }],
        ..SyncChanges::default()
    };
    repo.apply_changes(user_id, device_b, &second)
        .await
        .map_err(|e| sqlx::Error::Protocol(format!("apply_changes failed: {e}")))?;

    let row = sqlx::query(
        r#"
        SELECT value
        FROM user_settings
        WHERE user_id = $1 AND key = $2
        "#,
    )
    .bind(user_id)
    .bind("mode")
    .fetch_one(&pool)
    .await?;

    let value: serde_json::Value = row.try_get("value")?;
    assert_eq!(value, json!("later"));

    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn applied_changes_create_sync_events(pool: PgPool) -> Result<(), sqlx::Error> {
    let user_id = Uuid::new_v4();
    let device_id = Uuid::new_v4();

    sqlx::query("INSERT INTO users (id, oauth_sub) VALUES ($1, $2)")
        .bind(user_id)
        .bind(format!("sub-{}", user_id))
        .execute(&pool)
        .await?;

    let repo = SyncRepository::new(pool.clone());
    repo.touch_device(user_id, device_id, None, None, None)
        .await
        .map_err(|e| sqlx::Error::Protocol(format!("touch_device failed: {e}")))?;

    let changes = SyncChanges {
        settings: vec![SettingChange {
            key: "language".to_string(),
            value: json!("ar"),
            client_updated_at: 1_700_000_000_500,
        }],
        ..SyncChanges::default()
    };

    let (applied, skipped) = repo
        .apply_changes(user_id, device_id, &changes)
        .await
        .map_err(|e| sqlx::Error::Protocol(format!("apply_changes failed: {e}")))?;

    assert_eq!(applied, 1);
    assert_eq!(skipped, 0);

    let event_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM sync_events WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await?;
    assert_eq!(event_count, 1);

    let conflict_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM conflict_logs WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await?;
    assert_eq!(conflict_count, 0);

    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn skipped_changes_create_conflict_logs(pool: PgPool) -> Result<(), sqlx::Error> {
    let user_id = Uuid::new_v4();
    let device_a = Uuid::new_v4();
    let device_b = Uuid::new_v4();

    sqlx::query("INSERT INTO users (id, oauth_sub) VALUES ($1, $2)")
        .bind(user_id)
        .bind(format!("sub-{}", user_id))
        .execute(&pool)
        .await?;

    let repo = SyncRepository::new(pool.clone());
    repo.touch_device(user_id, device_a, None, None, None)
        .await
        .map_err(|e| sqlx::Error::Protocol(format!("touch_device failed: {e}")))?;
    repo.touch_device(user_id, device_b, None, None, None)
        .await
        .map_err(|e| sqlx::Error::Protocol(format!("touch_device failed: {e}")))?;

    let newer = SyncChanges {
        settings: vec![SettingChange {
            key: "mode".to_string(),
            value: json!("new"),
            client_updated_at: 1_700_000_100_000,
        }],
        ..SyncChanges::default()
    };

    let older = SyncChanges {
        settings: vec![SettingChange {
            key: "mode".to_string(),
            value: json!("old"),
            client_updated_at: 1_700_000_000_000,
        }],
        ..SyncChanges::default()
    };

    let (applied_first, skipped_first) = repo
        .apply_changes(user_id, device_a, &newer)
        .await
        .map_err(|e| sqlx::Error::Protocol(format!("apply_changes failed: {e}")))?;
    assert_eq!((applied_first, skipped_first), (1, 0));

    let (applied_second, skipped_second) = repo
        .apply_changes(user_id, device_b, &older)
        .await
        .map_err(|e| sqlx::Error::Protocol(format!("apply_changes failed: {e}")))?;
    assert_eq!((applied_second, skipped_second), (0, 1));

    let event_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM sync_events WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await?;
    assert_eq!(event_count, 1);

    let conflict_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM conflict_logs WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await?;
    assert_eq!(conflict_count, 1);

    Ok(())
}
