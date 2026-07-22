pub use crate::config_types::{
    AiSummaryConfig, Config, FeishuInstance, TelegramConfig, TelegramInstance,
};
use crate::event::EventKind;
use crate::legacy_config::from_legacy_env;
use std::env;
use std::path::{Path, PathBuf};

impl Config {
    pub fn load(legacy_complete_voice: &str) -> Result<Self, String> {
        if let Some(path) = find_near_executable("notifications.json") {
            let content = std::fs::read_to_string(&path)
                .map_err(|_| format!("无法读取通知配置：{}", path.display()))?;
            let config: Config = serde_json::from_str(&content)
                .map_err(|error| format!("通知配置 JSON 无效：{error}"))?;
            config.validate()?;
            return Ok(config);
        }

        let _ = dotenvy::from_path(
            find_near_executable(".env").unwrap_or_else(|| PathBuf::from(".env")),
        );
        Ok(from_legacy_env(legacy_complete_voice))
    }

    pub fn remote_enabled_for(&self, event: EventKind) -> bool {
        self.enabled && self.events.enabled_for(event)
    }

    pub fn ready_telegram_instances(&self) -> Vec<TelegramInstance> {
        if !self.telegram.enabled {
            return Vec::new();
        }
        self.telegram
            .instances
            .iter()
            .filter(|item| item.enabled && !item.bot_token.is_empty() && !item.chat_id.is_empty())
            .cloned()
            .collect()
    }

    pub fn ready_feishu_instances(&self) -> Vec<FeishuInstance> {
        if !self.feishu.enabled {
            return Vec::new();
        }
        self.feishu
            .instances
            .iter()
            .filter(|item| item.enabled && !item.webhook_url.is_empty())
            .cloned()
            .collect()
    }

    fn validate(&self) -> Result<(), String> {
        if self.version != 1 {
            return Err("通知配置 version 目前只支持 1".to_string());
        }
        validate_timeout("telegram.timeout_ms", self.telegram.timeout_ms)?;
        validate_timeout("feishu.timeout_ms", self.feishu.timeout_ms)?;
        validate_timeout("ai_summary.timeout_ms", self.ai_summary.timeout_ms)?;
        if self.message.raw_max_chars > 4_000 {
            return Err("message.raw_max_chars 必须在 0 到 4000 之间".to_string());
        }
        if !(100..=100_000).contains(&self.ai_summary.max_input_chars) {
            return Err("ai_summary.max_input_chars 必须在 100 到 100000 之间".to_string());
        }
        if !(20..=4_000).contains(&self.ai_summary.max_output_chars) {
            return Err("ai_summary.max_output_chars 必须在 20 到 4000 之间".to_string());
        }
        validate_http_url("telegram.api_base_url", &self.telegram.api_base_url, false)?;
        validate_proxy_url("telegram.proxy_url", &self.telegram.proxy_url)?;
        validate_proxy_url("feishu.proxy_url", &self.feishu.proxy_url)?;
        validate_http_url("ai_summary.base_url", &self.ai_summary.base_url, false)?;
        validate_proxy_url("ai_summary.proxy_url", &self.ai_summary.proxy_url)?;
        for item in self
            .feishu
            .instances
            .iter()
            .filter(|item| !item.webhook_url.is_empty())
        {
            validate_http_url("feishu.instances[].webhook_url", &item.webhook_url, false)?;
        }
        Ok(())
    }
}

fn find_near_executable(file_name: &str) -> Option<PathBuf> {
    if let Ok(executable) = env::current_exe()
        && let Some(directory) = executable.parent()
    {
        for path in [
            directory.join(file_name),
            directory.join("../../").join(file_name),
        ] {
            if path.is_file() {
                return Some(path);
            }
        }
    }
    let current = Path::new(file_name);
    current.is_file().then(|| current.to_path_buf())
}

fn validate_timeout(path: &str, value: u64) -> Result<(), String> {
    if (100..=60_000).contains(&value) {
        Ok(())
    } else {
        Err(format!("{path} 必须在 100 到 60000 之间"))
    }
}

fn validate_http_url(path: &str, value: &str, allow_empty: bool) -> Result<(), String> {
    if allow_empty && value.is_empty() {
        return Ok(());
    }
    let url = reqwest::Url::parse(value).map_err(|_| format!("{path} 不是有效 URL"))?;
    if matches!(url.scheme(), "http" | "https") {
        Ok(())
    } else {
        Err(format!("{path} 只支持 http 或 https"))
    }
}

fn validate_proxy_url(path: &str, value: &str) -> Result<(), String> {
    if value.is_empty() {
        return Ok(());
    }
    let url = reqwest::Url::parse(value).map_err(|_| format!("{path} 不是有效代理 URL"))?;
    if matches!(url.scheme(), "http" | "https" | "socks5" | "socks5h") {
        Ok(())
    } else {
        Err(format!("{path} 使用了不支持的代理协议"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config_types::EventConfig;

    #[test]
    fn partial_json_uses_safe_defaults() {
        let config: Config = serde_json::from_str(r#"{"version":1,"enabled":true}"#).unwrap();
        assert!(config.local.desktop_enabled);
        assert!(!config.telegram.enabled);
        assert!(!config.message.include_raw);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn event_switch_maps_waiting_events_to_confirm() {
        let events = EventConfig {
            complete: true,
            confirm: false,
            warning: true,
        };
        assert!(!events.enabled_for(EventKind::Idle));
        assert!(!events.enabled_for(EventKind::Elicitation));
        assert!(events.enabled_for(EventKind::Warning));
    }

    #[test]
    fn invalid_limits_and_urls_are_rejected_without_echoing_secrets() {
        let mut config = Config::default();
        config.message.raw_max_chars = 4_001;
        assert!(config.validate().unwrap_err().contains("raw_max_chars"));
        config.message.raw_max_chars = 500;
        config.telegram.api_base_url = "file:///secret".to_string();
        let error = config.validate().unwrap_err();
        assert!(error.contains("telegram.api_base_url"));
        assert!(!error.contains("secret"));
    }
}
