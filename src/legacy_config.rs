use crate::config_types::{
    AiSummaryConfig, Config, DEFAULT_SYSTEM_PROMPT, DEFAULT_USER_PROMPT, EventConfig, FeishuConfig,
    FeishuInstance, LocalConfig, MessageConfig, TelegramConfig, TelegramInstance, VoiceMessages,
};
use std::collections::HashSet;
use std::env;

pub fn from_legacy_env(complete_voice: &str) -> Config {
    let shared_proxy = env::var("HTTPS_PROXY")
        .ok()
        .filter(|value| !value.is_empty())
        .or_else(|| {
            env::var("HTTP_PROXY")
                .ok()
                .filter(|value| !value.is_empty())
        })
        .unwrap_or_default();
    let telegram_proxy = env_or_fallback("TELEGRAM_PROXY_URL", &shared_proxy);
    let feishu_proxy = env_or_fallback("FEISHU_PROXY_URL", &shared_proxy);
    let ai_proxy = env_or_fallback("AI_PROXY_URL", &shared_proxy);
    let (telegram_instances, feishu_instances) = parse_legacy_channels();
    let telegram_enabled = !telegram_instances.is_empty();
    let feishu_enabled = !feishu_instances.is_empty();
    Config {
        enabled: telegram_enabled || feishu_enabled,
        debug: env_bool("DEBUG", false),
        local: LocalConfig {
            desktop_enabled: env_bool("WIN_NOTIFY_ENABLE", true),
            voice_enabled: env_bool("VOICE_ENABLE", true),
            voice: VoiceMessages {
                complete: env_or("VOICE_STOP", complete_voice),
                confirm: env_or("VOICE_PERMISSION", "需要权限确认"),
                idle: env_or("VOICE_IDLE", "等待你的输入"),
                elicitation: env_or("VOICE_ELICITATION", "需要输入信息"),
                warning: env_or("VOICE_DEFAULT", "需要你的操作"),
            },
        },
        events: EventConfig {
            complete: env_bool("EVENT_COMPLETE", true),
            confirm: env_bool("EVENT_CONFIRM", true),
            warning: env_bool("EVENT_WARNING", true),
        },
        message: MessageConfig {
            include_raw: env_bool("MESSAGE_INCLUDE_RAW", false),
            raw_max_chars: env_usize("MESSAGE_RAW_MAX_CHARS", 500),
        },
        telegram: TelegramConfig {
            enabled: telegram_enabled,
            api_base_url: env_or("TELEGRAM_API_BASE_URL", "https://api.telegram.org"),
            proxy_url: telegram_proxy,
            timeout_ms: env_u64("TELEGRAM_TIMEOUT_MS", 5_000),
            instances: telegram_instances,
        },
        feishu: FeishuConfig {
            enabled: feishu_enabled,
            proxy_url: feishu_proxy,
            timeout_ms: env_u64("FEISHU_TIMEOUT_MS", 5_000),
            instances: feishu_instances,
        },
        ai_summary: AiSummaryConfig {
            enabled: env_bool("AI_ENABLE", true),
            base_url: env_or("AI_BASE_URL", "https://api.deepseek.com"),
            api_key: env_or("AI_API_KEY", ""),
            model: env_or("AI_MODEL", "deepseek-chat"),
            proxy_url: ai_proxy,
            timeout_ms: env_u64("AI_TIMEOUT_MS", 5_000),
            max_input_chars: env_usize("AI_MAX_INPUT_CHARS", 4_000),
            max_output_chars: env_or_alias("AI_MAX_OUTPUT_CHARS", "AI_MAX_WORDS", "500")
                .parse()
                .unwrap_or(500),
            system_prompt: env_or("AI_SYSTEM_PROMPT", DEFAULT_SYSTEM_PROMPT),
            user_prompt: env_or("AI_USER_PROMPT", DEFAULT_USER_PROMPT),
        },
    }
}

fn parse_legacy_channels() -> (Vec<TelegramInstance>, Vec<FeishuInstance>) {
    let mut prefix_set = HashSet::new();
    for key in env::vars().map(|(key, _)| key) {
        let (kind, rest) = if let Some(rest) = key.strip_prefix("TG_") {
            ("TG", rest)
        } else if let Some(rest) = key.strip_prefix("FS_") {
            ("FS", rest)
        } else {
            continue;
        };
        if let Some(index) = rest.split('_').next()
            && index.chars().all(|character| character.is_ascii_digit())
        {
            prefix_set.insert(format!("{kind}_{index}"));
        }
    }

    let mut prefixes: Vec<_> = prefix_set.into_iter().collect();
    prefixes.sort();
    let mut telegram = Vec::new();
    let mut feishu = Vec::new();
    for prefix in prefixes {
        if !env_bool(&format!("{prefix}_ENABLE"), false) {
            continue;
        }
        let index = prefix.split('_').nth(1).unwrap_or("0");
        let name = env_or(&format!("{prefix}_NAME"), "");
        if prefix.starts_with("TG_") {
            let instance = TelegramInstance {
                name: if name.is_empty() {
                    format!("telegram-{index}")
                } else {
                    name
                },
                enabled: true,
                bot_token: env_or(&format!("{prefix}_TOKEN"), ""),
                chat_id: env_or(&format!("{prefix}_CHAT_ID"), ""),
            };
            if !instance.bot_token.is_empty() && !instance.chat_id.is_empty() {
                telegram.push(instance);
            }
        } else {
            let instance = FeishuInstance {
                name: if name.is_empty() {
                    format!("feishu-{index}")
                } else {
                    name
                },
                enabled: true,
                webhook_url: env_or(&format!("{prefix}_WEBHOOK_URL"), ""),
            };
            if !instance.webhook_url.is_empty() {
                feishu.push(instance);
            }
        }
    }
    (telegram, feishu)
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn env_or_fallback(key: &str, fallback: &str) -> String {
    env::var(key).unwrap_or_else(|_| fallback.to_string())
}

fn env_or_alias(primary: &str, alias: &str, default: &str) -> String {
    env::var(primary)
        .or_else(|_| env::var(alias))
        .unwrap_or_else(|_| default.to_string())
}

fn env_u64(key: &str, default: u64) -> u64 {
    env_or(key, &default.to_string()).parse().unwrap_or(default)
}

fn env_usize(key: &str, default: usize) -> usize {
    env_or(key, &default.to_string()).parse().unwrap_or(default)
}

fn env_bool(key: &str, default: bool) -> bool {
    match env::var(key) {
        Ok(value) => value.eq_ignore_ascii_case("true"),
        Err(_) => default,
    }
}
