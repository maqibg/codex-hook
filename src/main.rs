mod channels;
mod config;
mod config_types;
mod event;
mod http;
mod legacy_config;
mod notification;
mod summarizer;

use config::Config;
use event::{EventKind, NotificationRequest, source_label};
use std::time::{SystemTime, UNIX_EPOCH};

const HOOK_TIMEOUT_SECS: u64 = 10;

fn time_str_from_epoch(epoch_seconds: u64) -> String {
    let total = epoch_seconds + 8 * 60 * 60;
    let hour = (total % 86_400) / 3_600;
    let minute = (total % 3_600) / 60;
    format!("{hour:02}:{minute:02}")
}

fn now_time_str() -> String {
    let epoch_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    time_str_from_epoch(epoch_seconds)
}

#[derive(serde::Deserialize)]
struct CodexPayload {
    r#type: String,
    #[serde(
        default,
        rename = "last-assistant-message",
        alias = "last_assistant_message"
    )]
    last_assistant_message: Option<String>,
    #[serde(default)]
    cwd: Option<String>,
    #[serde(default, rename = "session-id", alias = "session_id")]
    session_id: Option<String>,
    #[serde(default, rename = "thread-id", alias = "thread_id")]
    thread_id: Option<String>,
    #[serde(default)]
    client: Option<String>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    reason: Option<String>,
    #[serde(default, rename = "input-messages", alias = "input_messages")]
    input_messages: Vec<serde_json::Value>,
    #[serde(default, rename = "is-subagent", alias = "is_subagent")]
    is_subagent: Option<serde_json::Value>,
    #[serde(default, rename = "parent-agent-id", alias = "parent_agent_id")]
    parent_agent_id: Option<String>,
    #[serde(default, rename = "parent-turn-id", alias = "parent_turn_id")]
    parent_turn_id: Option<String>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let result =
        tokio::time::timeout(std::time::Duration::from_secs(HOOK_TIMEOUT_SECS), run()).await;
    if result.is_err() {
        eprintln!("[codex-hook] 超时退出");
    }
}

async fn run() {
    let json = match std::env::args().nth(1) {
        Some(value) if !value.trim().is_empty() => value,
        _ => return,
    };
    let Ok(payload) = serde_json::from_str::<CodexPayload>(&json) else {
        return;
    };
    let config = match Config::load("Codex任务完成") {
        Ok(config) => config,
        Err(error) => {
            eprintln!("[codex-hook] {error}");
            return;
        }
    };
    if config.debug {
        eprintln!("[codex-hook] 事件: {}", payload.r#type);
    }
    if should_ignore_client(payload.client.as_deref()) || is_subagent_payload(&payload) {
        return;
    }

    let source = source_label(
        "Codex",
        payload.cwd.as_deref(),
        payload
            .session_id
            .as_deref()
            .or(payload.thread_id.as_deref()),
    );
    let request = match payload.r#type.as_str() {
        "agent-turn-complete" => {
            let content = payload
                .last_assistant_message
                .as_deref()
                .unwrap_or("")
                .trim();
            if content.is_empty() {
                return;
            }
            NotificationRequest {
                event: EventKind::Complete,
                title: format!("Codex 完成 ({})", now_time_str()),
                content: content.to_string(),
                extra: None,
            }
        }
        "approval-requested" => {
            let content = payload
                .reason
                .as_deref()
                .or(payload.message.as_deref())
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("Agent 正在等待权限确认。");
            NotificationRequest {
                event: EventKind::Confirm,
                title: format!("{source} · 需要确认"),
                content: content.to_string(),
                extra: None,
            }
        }
        _ => return,
    };

    let report = notification::dispatch(&config, request).await;
    if config.debug {
        eprintln!(
            "[codex-hook] remote sent={}, failed={}, skipped={}",
            report.sent, report.failed, report.skipped
        );
    }
}

fn should_ignore_client(client: Option<&str>) -> bool {
    let Some(client) = client.filter(|value| !value.trim().is_empty()) else {
        return false;
    };
    let normalized = client.trim().to_lowercase().replace(['_', ' '], "-");
    normalized != "codex" && !normalized.starts_with("codex-")
}

fn is_subagent_payload(payload: &CodexPayload) -> bool {
    let explicit = payload.is_subagent.as_ref().is_some_and(|value| {
        value.as_bool().unwrap_or_else(|| {
            value.as_str().is_some_and(|text| {
                matches!(
                    text.to_ascii_lowercase().as_str(),
                    "1" | "true" | "yes" | "subagent"
                )
            })
        })
    });
    explicit
        || payload
            .parent_agent_id
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty())
        || payload
            .parent_turn_id
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty())
        || (payload
            .client
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
            && payload.input_messages.len() > 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payload(json: &str) -> CodexPayload {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn payload_accepts_kebab_and_snake_case_fields() {
        let kebab = payload(
            r#"{"type":"agent-turn-complete","last-assistant-message":"done","session-id":"s1"}"#,
        );
        assert_eq!(kebab.last_assistant_message.as_deref(), Some("done"));
        assert_eq!(kebab.session_id.as_deref(), Some("s1"));
        let snake = payload(
            r#"{"type":"agent-turn-complete","last_assistant_message":"done","thread_id":"t1"}"#,
        );
        assert_eq!(snake.thread_id.as_deref(), Some("t1"));
    }

    #[test]
    fn filters_non_codex_clients_and_delegated_payloads() {
        assert!(!should_ignore_client(Some("codex_exec")));
        assert!(should_ignore_client(Some("other-client")));
        assert!(is_subagent_payload(&payload(
            r#"{"type":"agent-turn-complete","is_subagent":true}"#
        )));
        assert!(is_subagent_payload(&payload(
            r#"{"type":"agent-turn-complete","parent-agent-id":"parent"}"#
        )));
    }

    #[test]
    fn approval_payload_does_not_require_assistant_message() {
        let approval = payload(r#"{"type":"approval-requested","reason":"需要文件权限"}"#);
        assert_eq!(approval.reason.as_deref(), Some("需要文件权限"));
        assert!(!is_subagent_payload(&approval));
    }

    #[test]
    fn completion_title_time_uses_utc_plus_eight() {
        assert_eq!(time_str_from_epoch(0), "08:00");
        assert_eq!(time_str_from_epoch(17 * 60 * 60 + 28 * 60), "01:28");
    }
}
