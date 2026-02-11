use crate::config::Channel;

pub async fn send(client: &reqwest::Client, ch: &Channel, title: &str, summary: &str, raw: Option<&str>, extra: Option<&str>) -> Result<(), String> {
    let url = ch.webhook_url.as_deref().ok_or("missing webhook_url")?;

    let mut elements = vec![
        serde_json::json!({ "tag": "div", "text": { "content": format!("**AI 摘要**\n{summary}"), "tag": "lark_md" } }),
    ];
    if let Some(r) = raw {
        let truncated: String = r.chars().take(500).collect();
        elements.push(serde_json::json!({ "tag": "hr" }));
        elements.push(serde_json::json!({ "tag": "div", "text": { "content": "**原始输出**", "tag": "lark_md" } }));
        elements.push(serde_json::json!({
            "tag": "note",
            "elements": [{ "tag": "plain_text", "content": truncated }]
        }));
    }
    if let Some(e) = extra {
        elements.push(serde_json::json!({ "tag": "hr" }));
        elements.push(serde_json::json!({ "tag": "div", "text": { "content": e, "tag": "lark_md" } }));
    }
    elements.push(serde_json::json!({ "tag": "note", "elements": [{ "tag": "plain_text", "content": "codex-hook" }] }));

    let body = serde_json::json!({
        "msg_type": "interactive",
        "card": {
            "config": { "wide_screen_mode": true },
            "header": { "template": "green", "title": { "content": title, "tag": "plain_text" } },
            "elements": elements,
        }
    });

    let resp = client.post(url).json(&body).send().await.map_err(|e| e.to_string())?;
    let data = resp.json::<serde_json::Value>().await.map_err(|e| e.to_string())?;
    let code = data.get("code").and_then(|v| v.as_i64()).unwrap_or(-1);
    let status = data.get("StatusCode").and_then(|v| v.as_i64()).unwrap_or(-1);
    if code != 0 && status != 0 {
        return Err(format!("飞书 API: {data}"));
    }
    Ok(())
}
