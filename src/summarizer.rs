use crate::config::Config;

/// 本地降级摘要
fn local_summary(content: &str, max: usize) -> String {
    let cleaned = content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .collect::<Vec<_>>()
        .join("。");

    if cleaned.chars().count() <= max {
        return cleaned;
    }

    let truncated: String = cleaned.chars().take(max).collect();
    // 在标点处截断
    if let Some(pos) = truncated.rfind(|c| "。；，、".contains(c)) {
        return truncated[..pos + 3].to_string(); // UTF-8 中文标点 3 bytes
    }
    format!("{truncated}...")
}

/// 调用 OpenAI 兼容 API 生成摘要，失败降级本地算法
pub async fn generate(cfg: &Config, client: &reqwest::Client, content: &str) -> String {
    if !cfg.ai_enable || cfg.ai_api_key.is_empty() {
        return local_summary(content, cfg.ai_max_words);
    }

    // 按字符截断，避免切断 UTF-8
    let truncated: String = content.chars().take(4000).collect();
    let base = cfg.ai_base_url.trim_end_matches('/');
    let base = if base.ends_with("/v1") { base.to_string() } else { format!("{base}/v1") };
    let url = format!("{base}/chat/completions");
    let body = serde_json::json!({
        "model": cfg.ai_model,
        "messages": [
            { "role": "system", "content": "你是摘要助手。要求：\n1. 输出简洁中文摘要，以浓缩易懂为首要目标，不必写满字数上限\n2. 使用纯文本，禁止使用 Markdown 格式（不要用 # ** `` 等标记）\n3. 如有多个要点用序号列出，每个序号独占一行\n4. 不加任何前缀（如\"摘要：\"）和后缀" },
            { "role": "user", "content": format!(
                "用中文总结以下内容，不超过{}字，突出关键操作和结果：\n\n{}",
                cfg.ai_max_words, truncated
            )}
        ],
        "max_tokens": 800,
        "temperature": 0.3
    });

    if cfg.debug { eprintln!("[codex-hook] AI 请求: {url}"); }

    let result = tokio::time::timeout(
        std::time::Duration::from_secs(8),
        async {
            let resp = client.post(&url)
                .header("Authorization", format!("Bearer {}", cfg.ai_api_key))
                .json(&body)
                .send().await.map_err(|e| e.to_string())?;
            let text = resp.text().await.map_err(|e| e.to_string())?;
            serde_json::from_str::<serde_json::Value>(&text).map_err(|e| e.to_string())
        }
    ).await;

    match result {
        Ok(Ok(data)) => {
            if let Some(text) = data.pointer("/choices/0/message/content").and_then(|v| v.as_str()) {
                let t = text.trim();
                if !t.is_empty() { return t.to_string(); }
            }
            if cfg.debug { eprintln!("[codex-hook] AI 响应无内容: {data}"); }
        }
        Ok(Err(e)) => { if cfg.debug { eprintln!("[codex-hook] AI 请求失败: {e}"); } }
        Err(_) => { if cfg.debug { eprintln!("[codex-hook] AI 请求超时"); } }
    }

    local_summary(content, cfg.ai_max_words)
}
