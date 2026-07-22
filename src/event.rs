use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
// 两个独立二进制共享事件契约，部分宿主不会主动产生所有事件。
#[allow(dead_code)]
pub enum EventKind {
    Complete,
    Confirm,
    Idle,
    Elicitation,
    Warning,
}

#[derive(Clone, Debug)]
pub struct NotificationRequest {
    pub event: EventKind,
    pub title: String,
    pub content: String,
    pub extra: Option<String>,
}

#[derive(Clone, Debug)]
pub struct RenderedNotification {
    pub event: EventKind,
    pub title: String,
    pub summary: String,
    pub raw: String,
    pub extra: Option<String>,
}

pub fn source_label(host: &str, cwd: Option<&str>, session_id: Option<&str>) -> String {
    let mut parts = vec![host.to_string()];
    if let Some(project) = cwd
        .and_then(|value| Path::new(value).file_name())
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
    {
        parts.push(project.to_string());
    }
    if let Some(session) = session_id.filter(|value| !value.is_empty()) {
        let token: String = session.chars().take(8).collect();
        parts.push(format!("会话 {token}"));
    }
    parts.join(" · ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_contains_host_project_and_short_session() {
        let label = source_label("Claude Code", Some("C:/work/demo"), Some("abcdef123456"));
        assert_eq!(label, "Claude Code · demo · 会话 abcdef12");
    }
}
