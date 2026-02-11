use std::collections::HashSet;
use std::env;

#[derive(Clone)]
pub struct Channel {
    pub ch_type: String, // "telegram" | "feishu"
    pub name: String,
    pub token: Option<String>,
    pub chat_id: Option<String>,
    pub webhook_url: Option<String>,
}

#[derive(Clone)]
pub struct Config {
    pub debug: bool,
    pub proxy: String,
    pub ai_enable: bool,
    pub ai_api_key: String,
    pub ai_base_url: String,
    pub ai_model: String,
    pub ai_max_words: usize,
    pub ai_system_prompt: String,
    pub ai_user_prompt: String,
    pub win_notify_enable: bool,
    pub voice_enable: bool,
    pub voice_stop: String,
    pub channels: Vec<Channel>,
}

fn env_path() -> std::path::PathBuf {
    let exe_dir = env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf()));
    if let Some(ref dir) = exe_dir {
        // exe 同目录
        let p = dir.join(".env");
        if p.exists() { return p; }
        // 项目根（target/release/../../.env）
        let p = dir.join("../../.env");
        if p.exists() { return p; }
    }
    std::path::PathBuf::from(".env")
}

fn parse_channels() -> Vec<Channel> {
    let mut prefixes = HashSet::new();
    for key in env::vars().map(|(k, _)| k) {
        if let Some(rest) = key.strip_prefix("TG_").or_else(|| key.strip_prefix("FS_")) {
            let prefix_type = if key.starts_with("TG_") { "TG" } else { "FS" };
            if let Some(idx) = rest.split('_').next() {
                if idx.chars().all(|c| c.is_ascii_digit()) {
                    prefixes.insert(format!("{}_{}", prefix_type, idx));
                }
            }
        }
    }

    let g = |prefix: &str, key: &str| -> String {
        env::var(format!("{prefix}_{key}")).unwrap_or_default()
    };

    let mut channels = Vec::new();
    for id in &prefixes {
        let enable = g(id, "ENABLE");
        if enable == "false" || enable.is_empty() { continue; }

        let ch_type = if id.starts_with("TG") { "telegram" } else { "feishu" };
        let idx = id.split('_').nth(1).unwrap_or("0");
        let ch = Channel {
            ch_type: ch_type.to_string(),
            name: { let n = g(id, "NAME"); if n.is_empty() { format!("{ch_type}-{idx}") } else { n } },
            token: non_empty(g(id, "TOKEN")),
            chat_id: non_empty(g(id, "CHAT_ID")),
            webhook_url: non_empty(g(id, "WEBHOOK_URL")),
        };

        // 校验
        let mut ok = true;
        if ch_type == "telegram" {
            if ch.token.is_none() { eprintln!("[codex-hook] {id}_TOKEN 未配置"); ok = false; }
            if ch.chat_id.is_none() { eprintln!("[codex-hook] {id}_CHAT_ID 未配置"); ok = false; }
        } else if ch.webhook_url.is_none() {
            eprintln!("[codex-hook] {id}_WEBHOOK_URL 未配置"); ok = false;
        }
        if ok { channels.push(ch); }
    }
    channels
}

fn non_empty(s: String) -> Option<String> {
    if s.is_empty() { None } else { Some(s) }
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

impl Config {
    pub fn load() -> Self {
        let _ = dotenvy::from_path(env_path());

        let cfg = Config {
            debug: env_or("DEBUG", "false") == "true",
            proxy: env::var("HTTPS_PROXY").or_else(|_| env::var("HTTP_PROXY")).unwrap_or_default(),
            ai_enable: env_or("AI_ENABLE", "true") != "false",
            ai_api_key: env_or("AI_API_KEY", ""),
            ai_base_url: env_or("AI_BASE_URL", "https://api.deepseek.com"),
            ai_model: env_or("AI_MODEL", "deepseek-chat"),
            ai_max_words: env_or("AI_MAX_WORDS", "500").parse().unwrap_or(500),
            ai_system_prompt: env_or("AI_SYSTEM_PROMPT", "你是摘要助手。要求：\n1. 输出简洁中文摘要，以浓缩易懂为首要目标，不必写满字数上限\n2. 使用纯文本，禁止使用 Markdown 格式（不要用 # ** `` 等标记）\n3. 如有多个要点用序号列出，每个序号独占一行\n4. 不加任何前缀（如\"摘要：\"）和后缀"),
            ai_user_prompt: env_or("AI_USER_PROMPT", "用中文总结以下内容，不超过{max_words}字，突出关键操作和结果：\n\n{content}"),
            win_notify_enable: env_or("WIN_NOTIFY_ENABLE", "true") != "false",
            voice_enable: env_or("VOICE_ENABLE", "true") != "false",
            voice_stop: env_or("VOICE_STOP", "Codex任务完成"),
            channels: parse_channels(),
        };
        if cfg.debug {
            eprintln!("[codex-hook] proxy={}, channels={}", if cfg.proxy.is_empty() { "(空)" } else { &cfg.proxy }, cfg.channels.len());
        }
        cfg
    }
}

pub fn build_http_client(proxy: &str) -> reqwest::Client {
    let mut builder = reqwest::Client::builder().timeout(std::time::Duration::from_secs(15));
    if !proxy.is_empty() {
        if let Ok(p) = reqwest::Proxy::all(proxy) {
            builder = builder.proxy(p);
        }
    }
    builder.build().unwrap_or_else(|_| reqwest::Client::new())
}
