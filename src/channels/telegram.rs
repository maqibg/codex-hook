use crate::config::Channel;

/// HTML 转义
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// 清理 AI 返回中可能残留的 Markdown 标记
fn strip_md(s: &str) -> String {
    s.replace("**", "").replace("```", "").replace('`', "").replace("##", "").replace('#', "")
}

pub async fn send(client: &reqwest::Client, ch: &Channel, title: &str, summary: &str, raw: Option<&str>, extra: Option<&str>) -> Result<(), String> {
    let token = ch.token.as_deref().ok_or("missing token")?;
    let chat_id = ch.chat_id.as_deref().ok_or("missing chat_id")?;

    let clean_summary = strip_md(summary);
    let mut text = format!("<b>{}</b>\n\n<b>AI 摘要：</b>\n{}", escape_html(title), escape_html(&clean_summary));
    if let Some(r) = raw {
        let truncated: String = r.chars().take(500).collect();
        text.push_str(&format!("\n\n<b>原始输出：</b>\n<pre>{}</pre>", escape_html(&truncated)));
    }
    if let Some(e) = extra {
        text.push_str(&format!("\n\n{}", escape_html(e)));
    }

    let resp = client.post(format!("https://api.telegram.org/bot{token}/sendMessage"))
        .json(&serde_json::json!({ "chat_id": chat_id, "text": text, "parse_mode": "HTML" }))
        .send().await.map_err(|e| e.to_string())?;

    let data = resp.json::<serde_json::Value>().await.map_err(|e| e.to_string())?;
    if data.get("ok").and_then(|v| v.as_bool()) != Some(true) {
        return Err(format!("API error: {data}"));
    }
    Ok(())
}
