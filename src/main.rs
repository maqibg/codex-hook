mod config;
mod summarizer;
mod channels;

use config::Config;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(serde::Deserialize)]
struct CodexPayload {
    r#type: String,
    #[serde(rename = "last-assistant-message")]
    last_assistant_message: Option<String>,
}

fn now_time_str() -> String {
    let total = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() + 8 * 3600; // UTC+8
    let h = (total % 86400) / 3600;
    let m = (total % 3600) / 60;
    format!("{:02}:{:02}", h, m)
}

async fn dispatch(cfg: &Config, client: &reqwest::Client, title: &str, summary: &str, raw: Option<&str>) {
    let mut tasks: Vec<tokio::task::JoinHandle<()>> = Vec::new();

    if cfg.win_notify_enable {
        let t = title.to_string();
        let s = summary.to_string();
        let voice_cfg = cfg.clone();
        tasks.push(tokio::spawn(async move {
            channels::windows::notify(&t, &s);
            channels::windows::speak(&voice_cfg);
        }));
    }

    for ch in &cfg.channels {
        let client = client.clone();
        let t = title.to_string();
        let s = summary.to_string();
        let r = raw.map(|s| s.to_string());
        let ch = ch.clone();
        tasks.push(tokio::spawn(async move {
            let result = match ch.ch_type.as_str() {
                "telegram" => channels::telegram::send(&client, &ch, &t, &s, r.as_deref(), None).await,
                "feishu" => channels::feishu::send(&client, &ch, &t, &s, r.as_deref(), None).await,
                _ => Ok(()),
            };
            if let Err(e) = result { eprintln!("[codex-hook] {} {} 失败: {}", ch.ch_type, ch.name, e); }
        }));
    }

    for t in tasks { let _ = t.await; }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let result = tokio::time::timeout(std::time::Duration::from_secs(10), run()).await;
    if result.is_err() { eprintln!("[codex-hook] 超时退出"); }
}

async fn run() {
    let json = match std::env::args().nth(1) {
        Some(s) if !s.trim().is_empty() => s,
        _ => return,
    };

    let Ok(payload) = serde_json::from_str::<CodexPayload>(&json) else { return };

    let cfg = Config::load();
    if cfg.debug { eprintln!("[codex-hook] 事件: {}", payload.r#type); }

    if payload.r#type != "agent-turn-complete" { return; }

    let content = match payload.last_assistant_message {
        Some(ref s) if !s.trim().is_empty() => s.as_str(),
        _ => return,
    };

    let proxy_client = config::build_http_client(&cfg.proxy);
    let ai_client = config::build_http_client("");
    let summary = summarizer::generate(&cfg, &ai_client, content).await;
    let title = format!("Codex 完成 ({})", now_time_str());
    let raw: String = content.trim().chars().take(500).collect();
    dispatch(&cfg, &proxy_client, &title, &summary, Some(&raw)).await;
}
