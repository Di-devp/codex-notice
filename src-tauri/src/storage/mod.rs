use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Context;
use chrono::{DateTime, Duration, Utc};
use serde_json::Value;
use sqlx::{sqlite::SqlitePoolOptions, Row, SqlitePool};

use crate::domain::{
    DashboardSummary, EventFilter, NoticeEvent, NoticeEventType, NoticeLevel, Pagination,
    PendingApproval, Provider, TrafficWidgetStatus,
};

const MIGRATION: &str = include_str!("../../migrations/0001_init.sql");
const ACTIVE_SESSION_TTL_SECONDS: i64 = 10 * 60;

pub async fn initialize(data_dir: &Path) -> anyhow::Result<SqlitePool> {
    tokio::fs::create_dir_all(data_dir).await?;
    let db_path = data_dir.join("notice.db");
    let url = format!("sqlite://{}?mode=rwc", db_path.display());
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .with_context(|| format!("connect sqlite database at {}", db_path.display()))?;

    for statement in MIGRATION.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() {
            sqlx::query(trimmed).execute(&pool).await?;
        }
    }

    Ok(pool)
}

pub fn app_data_dir() -> anyhow::Result<PathBuf> {
    let base = dirs::data_dir().context("cannot resolve user data directory")?;
    Ok(base.join("Notice"))
}

pub async fn insert_event(pool: &SqlitePool, event: &NoticeEvent) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        INSERT OR REPLACE INTO events (
          id, version, provider, event_type, session_id, run_id, dedupe_key,
          title, content, level, project, cwd, command, exit_code, duration_ms,
          timestamp, received_at, raw_payload
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&event.id)
    .bind(event.version)
    .bind(event.provider.as_str())
    .bind(event.event_type.as_str())
    .bind(&event.session_id)
    .bind(&event.run_id)
    .bind(&event.dedupe_key)
    .bind(&event.title)
    .bind(&event.content)
    .bind(event.level.as_str())
    .bind(&event.project)
    .bind(&event.cwd)
    .bind(&event.command)
    .bind(event.exit_code)
    .bind(event.duration_ms)
    .bind(event.timestamp.to_rfc3339())
    .bind(event.received_at.to_rfc3339())
    .bind(event.raw_payload.as_ref().map(Value::to_string))
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_events(
    pool: &SqlitePool,
    filter: EventFilter,
    pagination: Pagination,
) -> anyhow::Result<Vec<NoticeEvent>> {
    let rows = sqlx::query(
        r#"
        SELECT * FROM events
        WHERE (?1 IS NULL OR title LIKE ?1 OR content LIKE ?1 OR command LIKE ?1)
          AND (?2 IS NULL OR ?2 = '' OR level = ?2)
          AND (?3 IS NULL OR ?3 = '' OR project = ?3)
        ORDER BY received_at DESC
        LIMIT ?4 OFFSET ?5
        "#,
    )
    .bind(
        filter
            .search
            .filter(|s| !s.is_empty())
            .map(|s| format!("%{s}%")),
    )
    .bind(filter.level)
    .bind(filter.project)
    .bind(pagination.page_size.clamp(1, 500))
    .bind(((pagination.page.max(1) - 1) * pagination.page_size.max(1)).max(0))
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(row_to_event).collect()
}

pub async fn clear_events(pool: &SqlitePool) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM events").execute(pool).await?;
    Ok(())
}

pub async fn session_has_failures(
    pool: &SqlitePool,
    session_id: Option<&str>,
) -> anyhow::Result<bool> {
    let Some(session_id) = session_id else {
        return Ok(false);
    };
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE session_id = ? AND level = 'error'")
            .bind(session_id)
            .fetch_one(pool)
            .await?;
    Ok(count > 0)
}

pub async fn latest_non_stop_event_type(
    pool: &SqlitePool,
    session_id: Option<&str>,
) -> anyhow::Result<Option<String>> {
    let Some(session_id) = session_id else {
        return Ok(None);
    };
    let event_type = sqlx::query_scalar(
        r#"
        SELECT event_type
        FROM events
        WHERE session_id = ? AND title != 'Codex task finished'
        ORDER BY received_at DESC
        LIMIT 1
        "#,
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await?;
    Ok(event_type)
}

pub async fn touch_session_activity(
    pool: &SqlitePool,
    fallback: &NoticeEvent,
) -> anyhow::Result<()> {
    let Some(session_id) = fallback.session_id.as_deref() else {
        insert_event(pool, fallback).await?;
        return Ok(());
    };

    let latest: Option<(String, String, String)> = sqlx::query_as(
        r#"
        SELECT id, event_type, title
        FROM events
        WHERE session_id = ? AND title != 'Codex task finished'
        ORDER BY received_at DESC
        LIMIT 1
        "#,
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await?;

    let Some((id, event_type, _title)) = latest else {
        insert_event(pool, fallback).await?;
        return Ok(());
    };

    if event_type == "TASK_FAIL" || event_type == "USER_CONFIRM" {
        insert_event(pool, fallback).await?;
        return Ok(());
    }

    let now = Utc::now().to_rfc3339();
    sqlx::query("UPDATE events SET timestamp = ?, received_at = ? WHERE id = ?")
        .bind(&now)
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn dashboard_summary(pool: &SqlitePool) -> anyhow::Result<DashboardSummary> {
    let since = Utc::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE received_at >= ?")
        .bind(since.to_rfc3339())
        .fetch_one(pool)
        .await?;
    let success: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM events WHERE received_at >= ? AND level = 'success'",
    )
    .bind(since.to_rfc3339())
    .fetch_one(pool)
    .await?;
    let failure: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM events WHERE received_at >= ? AND level = 'error'",
    )
    .bind(since.to_rfc3339())
    .fetch_one(pool)
    .await?;
    let confirmations: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM events WHERE received_at >= ? AND event_type = 'USER_CONFIRM'",
    )
    .bind(since.to_rfc3339())
    .fetch_one(pool)
    .await?;
    let recent_summary: Option<String> =
        sqlx::query_scalar("SELECT content FROM events ORDER BY received_at DESC LIMIT 1")
            .fetch_optional(pool)
            .await?;

    Ok(DashboardSummary {
        service_status: "running".to_string(),
        today_total: total,
        today_success: success,
        today_failure: failure,
        today_confirmations: confirmations,
        recent_summary,
    })
}

pub async fn put_setting(pool: &SqlitePool, key: &str, value: &str) -> anyhow::Result<()> {
    sqlx::query("INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?, ?, ?)")
        .bind(key)
        .bind(value)
        .bind(Utc::now().to_rfc3339())
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_setting(pool: &SqlitePool, key: &str) -> anyhow::Result<Option<String>> {
    let value = sqlx::query_scalar("SELECT value FROM settings WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await?;
    Ok(value)
}

pub async fn bool_setting(
    pool: &SqlitePool,
    key: &str,
    default_value: bool,
) -> anyhow::Result<bool> {
    Ok(get_setting(pool, key)
        .await?
        .map(|value| value == "true")
        .unwrap_or(default_value))
}

pub async fn traffic_widget_status(pool: &SqlitePool) -> anyhow::Result<TrafficWidgetStatus> {
    let enabled = bool_setting(pool, "traffic_widget_enabled", true).await?;
    let always_on_top = bool_setting(pool, "traffic_widget_always_on_top", true).await?;
    let pending_approvals: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM pending_approvals WHERE status = 'pending'")
            .fetch_one(pool)
            .await?;
    let latest_event: Option<(String, String)> =
        sqlx::query_as("SELECT title, received_at FROM events ORDER BY received_at DESC LIMIT 1")
            .fetch_optional(pool)
            .await?;
    let latest_event_title = latest_event
        .as_ref()
        .filter(|(title, _received_at)| title == "Codex task finished")
        .map(|(title, _received_at)| title.clone());
    let session_counts = session_state_counts(pool).await?;
    let waiting_sessions = pending_approvals + session_counts.waiting;
    let failed_sessions = session_counts.failed;
    let active_sessions = session_counts.active;

    let (color, label, detail) = if failed_sessions > 0 {
        (
            "red",
            "Needs attention",
            format!("{failed_sessions} Codex session(s) failed"),
        )
    } else if waiting_sessions > 0 {
        (
            "yellow",
            "Waiting",
            format!("{waiting_sessions} approval(s) pending"),
        )
    } else if active_sessions > 0 {
        (
            "running",
            "Running",
            format!("{active_sessions} Codex session(s) running"),
        )
    } else if latest_event_title.is_some() {
        (
            "green",
            "Complete",
            "All tracked Codex tasks are complete".to_string(),
        )
    } else {
        (
            "green",
            "Ready",
            "Notice is watching for Codex activity".to_string(),
        )
    };

    Ok(TrafficWidgetStatus {
        enabled,
        always_on_top,
        color: color.to_string(),
        label: label.to_string(),
        detail,
        active_sessions: active_sessions as i64,
        pending_approvals: waiting_sessions,
        today_failures: failed_sessions,
        latest_event_title,
    })
}

#[derive(Default)]
struct SessionStateCounts {
    failed: i64,
    waiting: i64,
    active: usize,
}

#[derive(Default)]
struct SessionSnapshot {
    latest_title: String,
    latest_non_stop_event_type: String,
    latest_non_stop_level: String,
    latest_received_at: Option<DateTime<Utc>>,
}

async fn session_state_counts(pool: &SqlitePool) -> anyhow::Result<SessionStateCounts> {
    let since = (Utc::now() - Duration::minutes(30)).to_rfc3339();
    let rows = sqlx::query(
        r#"
        SELECT COALESCE(session_id, id) AS session_key, title, event_type, level, received_at
        FROM events
        WHERE received_at >= ?
        ORDER BY received_at ASC
        "#,
    )
    .bind(since)
    .fetch_all(pool)
    .await?;
    let mut latest_by_session = HashMap::<String, SessionSnapshot>::new();
    for row in rows {
        let title = row.get::<String, _>("title");
        let event_type = row.get::<String, _>("event_type");
        let level = row.get::<String, _>("level");
        let received_at = parse_time(row.get::<String, _>("received_at"))?;
        let snapshot = latest_by_session.entry(row.get("session_key")).or_default();
        snapshot.latest_title = title;
        snapshot.latest_received_at = Some(received_at);
        if snapshot.latest_title != "Codex task finished" {
            snapshot.latest_non_stop_event_type = event_type;
            snapshot.latest_non_stop_level = level;
        }
    }

    let mut counts = SessionStateCounts::default();
    let active_cutoff = Utc::now() - Duration::seconds(ACTIVE_SESSION_TTL_SECONDS);
    for snapshot in latest_by_session.values() {
        if snapshot.latest_non_stop_event_type == "TASK_FAIL"
            || snapshot.latest_non_stop_level == "error"
        {
            counts.failed += 1;
        } else if snapshot.latest_non_stop_event_type == "USER_CONFIRM" {
            counts.waiting += 1;
        } else if snapshot.latest_title != "Codex task finished"
            && snapshot
                .latest_received_at
                .map(|received_at| received_at >= active_cutoff)
                .unwrap_or(false)
        {
            counts.active += 1;
        }
    }
    Ok(counts)
}

pub async fn insert_delivery_attempt(
    pool: &SqlitePool,
    event_id: &str,
    channel: &str,
    dedupe_key: Option<&str>,
    status: &str,
    attempts: i64,
    last_error: Option<&str>,
) -> anyhow::Result<()> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        r#"
        INSERT INTO delivery_attempts
        (id, event_id, channel, dedupe_key, status, attempts, last_error, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(event_id)
    .bind(channel)
    .bind(dedupe_key)
    .bind(status)
    .bind(attempts)
    .bind(last_error)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn recent_delivery_exists(pool: &SqlitePool, dedupe_key: &str) -> anyhow::Result<bool> {
    let since = Utc::now() - Duration::minutes(5);
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM delivery_attempts WHERE dedupe_key = ? AND status = 'sent' AND created_at >= ?",
    )
    .bind(dedupe_key)
    .bind(since.to_rfc3339())
    .fetch_one(pool)
    .await?;
    Ok(count > 0)
}

pub async fn create_pending_approval(
    pool: &SqlitePool,
    command: &str,
    project: Option<&str>,
    risk_level: &str,
    rule: &str,
    timeout_secs: i64,
) -> anyhow::Result<PendingApproval> {
    let now = Utc::now();
    let approval = PendingApproval {
        id: uuid::Uuid::new_v4().to_string(),
        command: command.to_string(),
        project: project.map(ToOwned::to_owned),
        risk_level: risk_level.to_string(),
        rule: rule.to_string(),
        status: "pending".to_string(),
        created_at: now,
        expires_at: now + Duration::seconds(timeout_secs),
    };
    sqlx::query(
        r#"
        INSERT INTO pending_approvals
        (id, command, project, risk_level, rule, status, created_at, expires_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&approval.id)
    .bind(&approval.command)
    .bind(&approval.project)
    .bind(&approval.risk_level)
    .bind(&approval.rule)
    .bind(&approval.status)
    .bind(approval.created_at.to_rfc3339())
    .bind(approval.expires_at.to_rfc3339())
    .execute(pool)
    .await?;
    Ok(approval)
}

pub async fn list_pending_approvals(pool: &SqlitePool) -> anyhow::Result<Vec<PendingApproval>> {
    let rows = sqlx::query(
        "SELECT * FROM pending_approvals WHERE status = 'pending' ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;
    rows.into_iter().map(row_to_approval).collect()
}

pub async fn resolve_approval(pool: &SqlitePool, id: &str, status: &str) -> anyhow::Result<()> {
    sqlx::query("UPDATE pending_approvals SET status = ? WHERE id = ?")
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn approval_status(pool: &SqlitePool, id: &str) -> anyhow::Result<Option<String>> {
    let status = sqlx::query_scalar("SELECT status FROM pending_approvals WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(status)
}

fn row_to_event(row: sqlx::sqlite::SqliteRow) -> anyhow::Result<NoticeEvent> {
    let provider = match row.get::<String, _>("provider").as_str() {
        "webhook" => Provider::Webhook,
        _ => Provider::Codex,
    };
    let event_type = match row.get::<String, _>("event_type").as_str() {
        "TASK_START" => NoticeEventType::TaskStart,
        "TASK_FINISH" => NoticeEventType::TaskFinish,
        "TASK_FAIL" => NoticeEventType::TaskFail,
        "USER_CONFIRM" => NoticeEventType::UserConfirm,
        "WARNING" => NoticeEventType::Warning,
        _ => NoticeEventType::Error,
    };
    let level = match row.get::<String, _>("level").as_str() {
        "success" => NoticeLevel::Success,
        "warning" => NoticeLevel::Warning,
        "error" => NoticeLevel::Error,
        _ => NoticeLevel::Info,
    };
    let raw_payload: Option<String> = row.get("raw_payload");
    Ok(NoticeEvent {
        id: row.get("id"),
        version: row.get("version"),
        provider,
        event_type,
        session_id: row.get("session_id"),
        run_id: row.get("run_id"),
        dedupe_key: row.get("dedupe_key"),
        title: row.get("title"),
        content: row.get("content"),
        level,
        project: row.get("project"),
        cwd: row.get("cwd"),
        command: row.get("command"),
        exit_code: row.get("exit_code"),
        duration_ms: row.get("duration_ms"),
        timestamp: parse_time(row.get::<String, _>("timestamp"))?,
        received_at: parse_time(row.get::<String, _>("received_at"))?,
        raw_payload: raw_payload.and_then(|raw| serde_json::from_str(&raw).ok()),
    })
}

fn row_to_approval(row: sqlx::sqlite::SqliteRow) -> anyhow::Result<PendingApproval> {
    Ok(PendingApproval {
        id: row.get("id"),
        command: row.get("command"),
        project: row.get("project"),
        risk_level: row.get("risk_level"),
        rule: row.get("rule"),
        status: row.get("status"),
        created_at: parse_time(row.get::<String, _>("created_at"))?,
        expires_at: parse_time(row.get::<String, _>("expires_at"))?,
    })
}

fn parse_time(value: String) -> anyhow::Result<DateTime<Utc>> {
    Ok(DateTime::parse_from_rfc3339(&value)?.with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use super::{
        create_pending_approval, initialize, insert_event, touch_session_activity,
        traffic_widget_status, ACTIVE_SESSION_TTL_SECONDS,
    };
    use crate::domain::{NoticeEvent, NoticeEventType, NoticeLevel, Provider};
    use chrono::{Duration, Utc};
    use tempfile::tempdir;
    use uuid::Uuid;

    #[tokio::test]
    async fn traffic_status_defaults_to_green() {
        let dir = tempdir().unwrap();
        let pool = initialize(dir.path()).await.unwrap();
        let status = traffic_widget_status(&pool).await.unwrap();
        assert_eq!(status.color, "green");
        assert_eq!(status.label, "Ready");
    }

    #[tokio::test]
    async fn traffic_status_turns_yellow_for_pending_approval() {
        let dir = tempdir().unwrap();
        let pool = initialize(dir.path()).await.unwrap();
        create_pending_approval(
            &pool,
            "rm -rf /tmp/demo",
            Some("Notice"),
            "critical",
            "rm",
            30,
        )
        .await
        .unwrap();
        let status = traffic_widget_status(&pool).await.unwrap();
        assert_eq!(status.color, "yellow");
        assert_eq!(status.pending_approvals, 1);
    }

    #[tokio::test]
    async fn traffic_status_turns_red_for_failures() {
        let dir = tempdir().unwrap();
        let pool = initialize(dir.path()).await.unwrap();
        insert_event(&pool, &event(NoticeLevel::Error))
            .await
            .unwrap();
        let status = traffic_widget_status(&pool).await.unwrap();
        assert_eq!(status.color, "red");
        assert_eq!(status.today_failures, 1);
    }

    #[tokio::test]
    async fn traffic_status_returns_to_running_after_failure_gets_new_work() {
        let dir = tempdir().unwrap();
        let pool = initialize(dir.path()).await.unwrap();
        insert_event(&pool, &event(NoticeLevel::Error))
            .await
            .unwrap();
        insert_event(
            &pool,
            &event_with(
                NoticeEventType::TaskStart,
                NoticeLevel::Info,
                "Codex task started",
            ),
        )
        .await
        .unwrap();

        let status = traffic_widget_status(&pool).await.unwrap();

        assert_eq!(status.color, "running");
        assert_eq!(status.label, "Running");
    }

    #[tokio::test]
    async fn traffic_status_keeps_failure_after_stop_until_new_work() {
        let dir = tempdir().unwrap();
        let pool = initialize(dir.path()).await.unwrap();
        insert_event(&pool, &event(NoticeLevel::Error))
            .await
            .unwrap();
        insert_event(
            &pool,
            &event_with(
                NoticeEventType::TaskFinish,
                NoticeLevel::Success,
                "Codex task finished",
            ),
        )
        .await
        .unwrap();

        let status = traffic_widget_status(&pool).await.unwrap();

        assert_eq!(status.color, "red");
        assert_eq!(status.label, "Needs attention");
    }

    #[tokio::test]
    async fn traffic_status_turns_green_for_successful_stop() {
        let dir = tempdir().unwrap();
        let pool = initialize(dir.path()).await.unwrap();
        insert_event(
            &pool,
            &event_with(
                NoticeEventType::TaskFinish,
                NoticeLevel::Success,
                "Codex task finished",
            ),
        )
        .await
        .unwrap();

        let status = traffic_widget_status(&pool).await.unwrap();

        assert_eq!(status.color, "green");
        assert_eq!(status.label, "Complete");
    }

    #[tokio::test]
    async fn traffic_status_runs_marquee_for_active_session() {
        let dir = tempdir().unwrap();
        let pool = initialize(dir.path()).await.unwrap();
        insert_event(
            &pool,
            &event_with(
                NoticeEventType::TaskStart,
                NoticeLevel::Info,
                "Codex session started",
            ),
        )
        .await
        .unwrap();

        let status = traffic_widget_status(&pool).await.unwrap();

        assert_eq!(status.color, "running");
        assert_eq!(status.label, "Running");
    }

    #[tokio::test]
    async fn traffic_status_expires_stale_active_session_without_stop() {
        let dir = tempdir().unwrap();
        let pool = initialize(dir.path()).await.unwrap();
        let mut event = event_with(
            NoticeEventType::TaskStart,
            NoticeLevel::Info,
            "Codex task started",
        );
        event.timestamp = Utc::now() - Duration::seconds(ACTIVE_SESSION_TTL_SECONDS + 5);
        event.received_at = event.timestamp;
        insert_event(&pool, &event).await.unwrap();

        let status = traffic_widget_status(&pool).await.unwrap();

        assert_eq!(status.color, "green");
        assert_eq!(status.active_sessions, 0);
    }

    #[tokio::test]
    async fn traffic_status_stays_running_when_tool_activity_refreshes_session() {
        let dir = tempdir().unwrap();
        let pool = initialize(dir.path()).await.unwrap();
        let mut event = event_with(
            NoticeEventType::TaskStart,
            NoticeLevel::Info,
            "Codex task started",
        );
        event.timestamp = Utc::now() - Duration::seconds(ACTIVE_SESSION_TTL_SECONDS + 5);
        event.received_at = event.timestamp;
        insert_event(&pool, &event).await.unwrap();

        let activity = event_with(
            NoticeEventType::TaskStart,
            NoticeLevel::Info,
            "Codex task activity",
        );
        touch_session_activity(&pool, &activity).await.unwrap();

        let status = traffic_widget_status(&pool).await.unwrap();

        assert_eq!(status.color, "running");
        assert_eq!(status.active_sessions, 1);
    }

    #[tokio::test]
    async fn traffic_status_returns_to_running_when_tool_activity_follows_failure() {
        let dir = tempdir().unwrap();
        let pool = initialize(dir.path()).await.unwrap();
        insert_event(
            &pool,
            &event_with(
                NoticeEventType::TaskFail,
                NoticeLevel::Error,
                "Codex tool failed",
            ),
        )
        .await
        .unwrap();

        let activity = event_with(
            NoticeEventType::TaskStart,
            NoticeLevel::Info,
            "Codex task activity",
        );
        touch_session_activity(&pool, &activity).await.unwrap();

        let status = traffic_widget_status(&pool).await.unwrap();

        assert_eq!(status.color, "running");
        assert_eq!(status.active_sessions, 1);
    }

    #[tokio::test]
    async fn traffic_status_returns_to_running_after_approval_resumes() {
        let dir = tempdir().unwrap();
        let pool = initialize(dir.path()).await.unwrap();
        insert_event(
            &pool,
            &event_with(
                NoticeEventType::UserConfirm,
                NoticeLevel::Warning,
                "Codex needs confirmation",
            ),
        )
        .await
        .unwrap();
        insert_event(
            &pool,
            &event_with(
                NoticeEventType::TaskStart,
                NoticeLevel::Info,
                "Codex task resumed",
            ),
        )
        .await
        .unwrap();

        let status = traffic_widget_status(&pool).await.unwrap();

        assert_eq!(status.color, "running");
        assert_eq!(status.label, "Running");
    }

    fn event(level: NoticeLevel) -> NoticeEvent {
        event_with(NoticeEventType::TaskFail, level, "Codex tool failed")
    }

    fn event_with(event_type: NoticeEventType, level: NoticeLevel, title: &str) -> NoticeEvent {
        NoticeEvent {
            id: Uuid::new_v4().to_string(),
            version: 1,
            provider: Provider::Codex,
            event_type,
            session_id: Some("test".to_string()),
            run_id: None,
            dedupe_key: None,
            title: title.to_string(),
            content: "failure".to_string(),
            level,
            project: Some("Notice".to_string()),
            cwd: None,
            command: None,
            exit_code: Some(1),
            duration_ms: None,
            timestamp: Utc::now(),
            received_at: Utc::now(),
            raw_payload: None,
        }
    }
}
