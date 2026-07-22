use crate::config::AiSummaryConfig;
use crate::http::{build_client, response_json};

pub fn truncate_chars(content: &str, maximum: usize) -> String {
    let chars: Vec<char> = content.chars().collect();
    if chars.len() <= maximum {
        return content.to_string();
    }
    if maximum <= 3 {
        return chars.into_iter().take(maximum).collect();
    }
    format!(
        "{}...",
        chars.into_iter().take(maximum - 3).collect::<String>()
    )
}

pub fn local_summary(content: &str, maximum: usize) -> String {
    let cleaned = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("。");
    if cleaned.chars().count() <= maximum {
        return cleaned;
    }
    let truncated = truncate_chars(&cleaned, maximum);
    let boundary = ["。", "；", "！", "？"]
        .iter()
        .filter_map(|mark| truncated.rfind(mark))
        .max();
    if let Some(index) = boundary
        && truncated[..index].chars().count() >= maximum / 2
    {
        return truncated[..index + 3].to_string();
    }
    truncated
}

pub fn resolve_chat_completions_url(base_url: &str) -> String {
    let base = base_url.trim_end_matches('/');
    if base.ends_with("/chat/completions") {
        return base.to_string();
    }
    let base = if base.ends_with("/v1") {
        base.to_string()
    } else {
        format!("{base}/v1")
    };
    format!("{base}/chat/completions")
}

pub async fn generate(cfg: &AiSummaryConfig, content: &str, debug: bool) -> String {
    let fallback = local_summary(content, cfg.max_output_chars);
    if !cfg.enabled || cfg.api_key.is_empty() || cfg.model.is_empty() || content.trim().is_empty() {
        return fallback;
    }
    let client = match build_client(&cfg.proxy_url, cfg.timeout_ms) {
        Ok(client) => client,
        Err(error) => {
            if debug {
                eprintln!("[codex-hook] AI client 失败: {}", error.code);
            }
            return fallback;
        }
    };
    let input = truncate_chars(content, cfg.max_input_chars);
    let url = resolve_chat_completions_url(&cfg.base_url);
    let body = serde_json::json!({
        "model": cfg.model,
        "messages": [
            { "role": "system", "content": cfg.system_prompt },
            { "role": "user", "content": cfg.user_prompt
                .replace("{max_output_chars}", &cfg.max_output_chars.to_string())
                .replace("{max_words}", &cfg.max_output_chars.to_string())
                .replace("{content}", &input) }
        ],
        "max_tokens": 800,
        "temperature": 0.3
    });
    if debug {
        eprintln!("[codex-hook] AI 请求已开始");
    }
    let result = tokio::time::timeout(std::time::Duration::from_millis(cfg.timeout_ms), async {
        let resp = client
            .post(&url)
            .bearer_auth(&cfg.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|_| "ai-request-failed")?;
        response_json(resp).await.map_err(|error| error.code)
    })
    .await;
    match result {
        Ok(Ok(data)) => {
            if let Some(text) = data
                .pointer("/choices/0/message/content")
                .and_then(|v| v.as_str())
            {
                let text = text.trim();
                if !text.is_empty() {
                    return truncate_chars(text, cfg.max_output_chars);
                }
            }
            if debug {
                eprintln!("[codex-hook] AI 响应无内容");
            }
        }
        Ok(Err(code)) => {
            if debug {
                eprintln!("[codex-hook] AI 请求失败: {code}");
            }
        }
        Err(_) => {
            if debug {
                eprintln!("[codex-hook] AI 请求失败: request-timeout");
            }
        }
    }
    fallback
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_summary_is_unicode_safe_and_bounded() {
        let summary = local_summary("第一项完成\n第二项完成并验证", 12);
        assert!(summary.chars().count() <= 12);
        assert!(!summary.contains('\u{fffd}'));
    }

    #[test]
    fn chat_completion_url_accepts_base_v1_and_full_endpoint() {
        assert_eq!(
            resolve_chat_completions_url("https://example.test"),
            "https://example.test/v1/chat/completions"
        );
        assert_eq!(
            resolve_chat_completions_url("https://example.test/v1"),
            "https://example.test/v1/chat/completions"
        );
        assert_eq!(
            resolve_chat_completions_url("https://example.test/v1/chat/completions"),
            "https://example.test/v1/chat/completions"
        );
    }
}
