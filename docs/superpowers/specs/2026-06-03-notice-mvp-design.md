# Notice MVP Design Spec

## 1. Project Overview

**Notice** is a Developer Notification Center running in the macOS menu bar.
Its goal is to act as a unified hub for receiving state notifications from AI Agents (e.g., Codex, Trae), Dev Tools, and CI/CD systems, and intelligently forwarding them to channels like Feishu, WeCom, and Emails.

This document covers the **MVP (Minimum Viable Product)** phase, focusing entirely on a **local, secure environment** integrating specifically with **OpenAI Codex CLI** and forwarding to **Feishu**.

## 1.1 MVP Boundaries

The MVP explicitly does **not** include remote mobile approval, public webhook ingestion, multi-device sync, CI/CD integrations, or multi-user accounts. These features are deferred because they require stronger authentication, replay protection, audit logs, and operational controls.

The MVP success criteria are:
- Codex hooks can be installed, detected, and removed without overwriting existing user hook configuration.
- Codex completion, failure, and approval-related events are recorded locally and can be forwarded to Feishu.
- Feishu delivery failures are retried, recorded, and visible in the UI.
- Notification bodies, logs, and persisted event payloads pass through the same redaction pipeline.
- Quitting Notice does not leave unmanaged background processes or broken hooks behind.

## 1.2 External Compatibility Notes

Codex hook behavior must be verified against the installed Codex version during implementation. Context7 documentation for `/openai/codex` shows current hook event names including `PreToolUse`, `PermissionRequest`, `PostToolUse`, `SessionStart`, `UserPromptSubmit`, `SubagentStart`, `SubagentStop`, and `Stop`, and hook command output fields such as `continue`, `stopReason`, `suppressOutput`, and `systemMessage`.

Implementation must not assume that `hooks.json` or `codex_hooks = true` is the only supported configuration path. Use the current Codex configuration format as the source of truth, and keep the Hook Manager version-aware.

## 2. Core Architecture

The MVP relies on a pure local setup without exposing any public endpoints, prioritizing security and simplicity.

### 2.1 Components
- **Notice Backend (Rust/Tauri):** 
  - Runs a local HTTP server bound **strictly** to `127.0.0.1:3746`.
  - Responsible for receiving events, running the rule/aggregation engine, persisting data, and pushing to Feishu Webhooks.
- **Notice Frontend (Vue3/Naive UI):**
  - Communicates with Rust via Tauri IPC.
  - Acts as the dashboard for configuration, logs, and hook management.
- **Codex Integration (Native Hooks):**
  - Utilizes the official Codex CLI hook mechanism for events such as `SessionStart`, `PermissionRequest`, `PreToolUse`, `PostToolUse`, and `Stop`.
  - Notice injects a small managed wrapper command into Codex hooks. The wrapper reads hook stdin, attaches a local auth token, calls Notice's local server, handles timeouts, and returns Codex-compatible hook output.

### 2.2 Data Flow
```text
[Codex Agent] -> Triggers Native Hook (`Stop`, `PreToolUse`, etc.)
    ↓
[Notice hook wrapper] -> HTTP POST with `X-Notice-Token` -> `127.0.0.1:3746/api/webhook/codex`
    ↓
[Notice Backend] -> Parses to `NoticeEvent`
    ↓
[Rule Engine] -> Aggregation / Risk Detection
    ↓
[Channel Dispatcher] -> HTTP POST -> Feishu Webhook URL
    ↓
[Developer's Phone]
```

## 3. Core Modules Design

### 3.1 Hook Manager
Responsible for automating the Codex integration.
- **Install:** Detects the installed Codex hook configuration format, backs up the user's existing configuration, then appends Notice-managed hook entries for supported events.
- **Managed wrapper:** Stores wrapper scripts and metadata under `~/Library/Application Support/Notice/hooks/`. The wrapper should be tiny, deterministic, and easy to inspect.
- **Conflict handling:** Never overwrite existing hooks. All Notice-managed blocks must have stable markers so they can be detected and removed.
- **Uninstall:** Removes only Notice-managed configuration and wrapper files, preserving user-owned hook configuration.
- **Offline fallback:** If Notice is not running, low-risk events should fail open quickly or queue locally according to user settings. High-risk events should follow the configured fail-open/fail-closed policy.

### 3.2 Aggregation Engine (Anti-Spam)
To prevent notification bombing from frequent agent actions:
- **State Machine + Time Window:** `PostToolUse` events (e.g., executing a command) are recorded in SQLite and memory but **do not** trigger immediate notifications.
- **Trigger Conditions:** A Feishu card summarizing the run (e.g., "36 commands run, 31 success, 4 failed") is only sent when:
  1. A `Stop` event is received (Agent turn ended).
  2. A 10-minute timeout occurs without a `Stop` event (preventing silence on long runs).
  3. Continuous failures (e.g., 5 consecutive `Exit Code != 0` events) occur.

### 3.3 Risk Interception & Local Approval (PreToolUse)
Leveraging Codex's `PreToolUse` hook to prevent dangerous commands:
- **Detection:** Uses ranked rules instead of raw substring matching. Keyword checks are only the first MVP heuristic.
- **Interception:**
  1. If detected, Notice sends an urgent Feishu alert: "🚨 High-risk command detected. Please return to your computer to approve."
  2. Notice triggers a native macOS approval dialog or notification with **Approve** and **Reject** actions.
  3. The hook wrapper waits only up to a short configured timeout, then returns a Codex-compatible allow/deny response.
- **Resolution:**
  - If Approved: wrapper returns `continue: true` when supported by the current Codex hook protocol.
  - If Rejected: wrapper returns `continue: false` with a `stopReason`, or exits non-zero only when that is the verified abort path for the installed Codex version.
  - If Timed Out: follows the user's risk policy. The recommended default is fail-closed for `critical` rules and fail-open for low-risk telemetry events.

Rule levels:
- **critical:** Requires local approval by default, e.g. destructive filesystem commands, `sudo rm`, disk formatting, database `drop` / `truncate`.
- **high:** Notifies and can optionally require local approval, e.g. `chmod 777`, mass deletion, system directory writes.
- **medium:** Records or aggregates by default, e.g. ordinary `delete`, `git clean`, `docker prune`.

## 4. Unified Event Model

```typescript
export interface NoticeEvent {
  id: string
  version: 1
  provider: "codex" | "webhook"
  eventType: "TASK_START" | "TASK_FINISH" | "TASK_FAIL" | "USER_CONFIRM" | "WARNING" | "ERROR"
  sessionId?: string
  runId?: string
  dedupeKey?: string
  title: string
  content: string
  level: "info" | "success" | "warning" | "error"
  project?: string
  cwd?: string
  command?: string
  exitCode?: number
  durationMs?: number
  timestamp: string
  receivedAt: string
  rawPayload?: any
}
```

`rawPayload` is for local diagnostics only. It must be redacted before display, storage in long-lived logs, or delivery to Feishu.

## 5. Security & Storage
- **Network Security:** Local server listens *only* on `127.0.0.1`.
- **Local Auth:** Every local hook request must include `X-Notice-Token`. The token is generated on first launch and stored securely.
- **Credential Security:** Webhook URLs and tokens are masked in the UI and encrypted locally.
- **Request Limits:** Local ingestion rejects missing tokens, invalid tokens, oversized payloads, and unsupported event versions.
- **Redaction:** Commands, stderr, stdout snippets, environment variables, raw payloads, logs, and Feishu cards all pass through one redaction pipeline.
- **Storage:**
  - Config: `~/Library/Application Support/Notice/config.json`
  - DB (Events): `~/Library/Application Support/Notice/notice.db` (SQLite)
  - Hook wrappers and backups: `~/Library/Application Support/Notice/hooks/`
  - Logs: `~/Library/Application Support/Notice/logs/`
- **Retention:**
  - UI shows the most recent 100 events by default.
  - SQLite keeps 30 days of events by default.
  - Logs keep 7 days by default.

## 6. MVP Sprint Plan

- **Sprint 1:** Tauri setup, Feishu integration, config management, webhook masking, send-test flow, retry/error recording.
- **Sprint 2:** Local Axum HTTP server, `X-Notice-Token` auth, request size limits, event ingestion, SQLite persistence, event filtering.
- **Sprint 3:** Codex Hook Manager, version-aware hook config detection, install/detect/uninstall, backup/restore, wrapper script generation.
- **Sprint 4:** Menu bar UI, dashboard, event logs, channel status, provider status, hook health checks.
- **Sprint 5:** Aggregation engine, high-risk rule levels, local approval, rate limiting, redaction verification.

## 7. Test Plan

- **Hook safety:** Installing and uninstalling Notice preserves unrelated Codex configuration and hooks.
- **Server security:** Requests without `X-Notice-Token`, with invalid tokens, and with oversized bodies are rejected.
- **Feishu reliability:** Delivery timeout, non-2xx response, and rate-limit scenarios are retried and recorded.
- **Aggregation:** `Stop`, timeout, and consecutive failures all produce the expected summary event.
- **Risk rules:** Critical commands trigger local approval; ordinary commands do not block.
- **Redaction:** Known secret patterns never appear in Feishu messages, UI event details, or logs.
