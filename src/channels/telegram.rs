use crate::config::{TelegramConfig, TelegramInstance};
use crate::event::RenderedNotification;
use crate::http::{RequestError, response_json};

/// HTML 转义
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// 清理 AI 返回中可能残留的 Markdown 标记
fn strip_md(s: &str) -> String {
    s.replace("```", "").replace(['*', '_', '~', '`', '#'], "")
}

fn escape_within(value: &str, maximum: usize) -> String {
    let mut output = String::new();
    for character in value.chars() {
        let escaped = escape_html(&character.to_string());
        if output.chars().count() + escaped.chars().count() > maximum {
            break;
        }
        output.push_str(&escaped);
    }
    output
}

pub fn build_message(notification: &RenderedNotification) -> String {
    const MAXIMUM: usize = 4_000;
    let title = format!("<b>{}</b>", escape_within(&notification.title, 500));
    let summary_prefix = "\n\n<b>AI 摘要：</b>\n";
    let extra = notification
        .extra
        .as_ref()
        .map(|value| format!("\n\n{}", escape_html(value)))
        .unwrap_or_default();
    let reserved_raw = if notification.raw.is_empty() { 0 } else { 800 };
    let summary_budget = MAXIMUM.saturating_sub(
        title.chars().count()
            + summary_prefix.chars().count()
            + extra.chars().count()
            + reserved_raw,
    );
    let summary = escape_within(&strip_md(&notification.summary), summary_budget);
    let mut text = format!("{title}{summary_prefix}{summary}");
    if !notification.raw.is_empty() {
        let prefix = "\n\n<b>原始输出：</b>\n<pre>";
        let suffix = "</pre>";
        let budget = MAXIMUM.saturating_sub(
            text.chars().count()
                + prefix.chars().count()
                + suffix.chars().count()
                + extra.chars().count(),
        );
        if budget > 0 {
            text.push_str(&format!(
                "{prefix}{}{suffix}",
                escape_within(&notification.raw, budget)
            ));
        }
    }
    text.push_str(&extra);
    text
}

pub async fn send(
    client: &reqwest::Client,
    config: &TelegramConfig,
    instance: &TelegramInstance,
    notification: &RenderedNotification,
) -> Result<(), RequestError> {
    let base = config.api_base_url.trim_end_matches('/');
    let response = client
        .post(format!("{base}/bot{}/sendMessage", instance.bot_token))
        .json(&serde_json::json!({
            "chat_id": instance.chat_id,
            "text": build_message(notification),
            "parse_mode": "HTML",
            "disable_web_page_preview": true
        }))
        .send()
        .await
        .map_err(|_| RequestError::new("request-failed"))?;
    let data = response_json(response).await?;
    if data.get("ok").and_then(|v| v.as_bool()) != Some(true) {
        return Err(RequestError::new("telegram-api-rejected"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::EventKind;
    use crate::http::build_client;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn message_escapes_html_and_stays_bounded() {
        let message = build_message(&RenderedNotification {
            event: EventKind::Complete,
            title: "<Project & Agent>".to_string(),
            summary: "**done <script> & ok**".repeat(500),
            raw: "<raw>".to_string(),
            extra: None,
        });
        assert!(message.starts_with("<b>&lt;Project &amp; Agent&gt;</b>"));
        assert!(message.contains("<b>AI 摘要：</b>"));
        assert!(message.contains("&lt;raw&gt;"));
        assert!(message.chars().count() <= 4_000);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn send_posts_json_and_accepts_success_response() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = Vec::new();
            let mut buffer = [0_u8; 4096];
            loop {
                let count = stream.read(&mut buffer).unwrap();
                if count == 0 {
                    break;
                }
                request.extend_from_slice(&buffer[..count]);
                if request_complete(&request) {
                    break;
                }
            }
            let body = r#"{"ok":true}"#;
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            ).unwrap();
            String::from_utf8(request).unwrap()
        });

        let config = TelegramConfig {
            enabled: true,
            api_base_url: format!("http://{address}"),
            timeout_ms: 2_000,
            ..Default::default()
        };
        let instance = TelegramInstance {
            name: "test".to_string(),
            enabled: true,
            bot_token: "test-token".to_string(),
            chat_id: "test-chat".to_string(),
        };
        let notification = RenderedNotification {
            event: EventKind::Complete,
            title: "完成".to_string(),
            summary: "已验证".to_string(),
            raw: String::new(),
            extra: None,
        };
        let client = build_client("", config.timeout_ms).unwrap();
        send(&client, &config, &instance, &notification)
            .await
            .unwrap();
        let request = server.join().unwrap();
        assert!(request.starts_with("POST /bottest-token/sendMessage HTTP/1.1"));
        assert!(request.contains(r#""chat_id":"test-chat""#));
    }

    fn request_complete(request: &[u8]) -> bool {
        let Some(header_end) = request.windows(4).position(|window| window == b"\r\n\r\n") else {
            return false;
        };
        let headers = String::from_utf8_lossy(&request[..header_end]);
        let content_length = headers
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse::<usize>().ok())
                    .flatten()
            })
            .unwrap_or(0);
        request.len() >= header_end + 4 + content_length
    }
}
