use crate::config::FeishuInstance;
use crate::event::{EventKind, RenderedNotification};
use crate::http::{RequestError, response_json};
use crate::summarizer::truncate_chars;

pub fn build_card(notification: &RenderedNotification) -> serde_json::Value {
    let mut elements = vec![serde_json::json!({
        "tag": "div",
        "text": {
            "content": format!("**AI 摘要**\n{}", truncate_chars(&notification.summary, 3980)),
            "tag": "lark_md"
        }
    })];
    if !notification.raw.is_empty() {
        elements.push(serde_json::json!({ "tag": "hr" }));
        elements.push(serde_json::json!({
            "tag": "div",
            "text": { "content": "**原始输出**", "tag": "lark_md" }
        }));
        elements.push(serde_json::json!({
            "tag": "note",
            "elements": [{ "tag": "plain_text", "content": truncate_chars(&notification.raw, 4000) }]
        }));
    }
    if let Some(extra) = &notification.extra {
        elements.push(serde_json::json!({ "tag": "hr" }));
        elements.push(
            serde_json::json!({ "tag": "div", "text": { "content": extra, "tag": "lark_md" } }),
        );
    }
    elements.push(serde_json::json!({ "tag": "note", "elements": [{ "tag": "plain_text", "content": "codex-hook" }] }));
    let template = match notification.event {
        EventKind::Complete => "green",
        EventKind::Warning => "orange",
        EventKind::Confirm | EventKind::Idle | EventKind::Elicitation => "yellow",
    };
    serde_json::json!({
        "msg_type": "interactive",
        "card": {
            "config": { "wide_screen_mode": true },
            "header": { "template": template, "title": { "content": truncate_chars(&notification.title, 200), "tag": "plain_text" } },
            "elements": elements,
        }
    })
}

pub async fn send(
    client: &reqwest::Client,
    instance: &FeishuInstance,
    notification: &RenderedNotification,
) -> Result<(), RequestError> {
    let response = client
        .post(&instance.webhook_url)
        .json(&build_card(notification))
        .send()
        .await
        .map_err(|_| RequestError::new("request-failed"))?;
    let data = response_json(response).await?;
    let code = data.get("code").and_then(|v| v.as_i64()).unwrap_or(-1);
    let status = data
        .get("StatusCode")
        .and_then(|v| v.as_i64())
        .unwrap_or(-1);
    if code != 0 && status != 0 {
        return Err(RequestError::new("feishu-api-rejected"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warning_card_uses_orange_and_legacy_sections() {
        let card = build_card(&RenderedNotification {
            event: EventKind::Warning,
            title: "警告".to_string(),
            summary: "<unsafe>".to_string(),
            raw: "原始内容".to_string(),
            extra: None,
        });
        assert_eq!(
            card.pointer("/card/header/template")
                .and_then(|value| value.as_str()),
            Some("orange")
        );
        assert_eq!(
            card.pointer("/card/elements/0/text/tag")
                .and_then(|value| value.as_str()),
            Some("lark_md")
        );
        assert_eq!(
            card.pointer("/card/elements/0/text/content")
                .and_then(|value| value.as_str()),
            Some("**AI 摘要**\n<unsafe>")
        );
        assert_eq!(
            card.pointer("/card/elements/2/text/content")
                .and_then(|value| value.as_str()),
            Some("**原始输出**")
        );
    }
}
