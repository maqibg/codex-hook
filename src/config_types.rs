use crate::event::EventKind;
use serde::Deserialize;

pub const DEFAULT_SYSTEM_PROMPT: &str = "你是摘要助手。请输出简洁中文纯文本摘要，突出关键操作和结果；不要使用 Markdown，不添加摘要前缀或后缀。";
pub const DEFAULT_USER_PROMPT: &str =
    "用中文总结以下内容，不超过 {max_output_chars} 字：\n\n{content}";

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    pub version: u32,
    pub enabled: bool,
    pub debug: bool,
    pub min_duration_seconds: u64,
    pub local: LocalConfig,
    pub events: EventConfig,
    pub message: MessageConfig,
    pub telegram: TelegramConfig,
    pub feishu: FeishuConfig,
    pub ai_summary: AiSummaryConfig,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct LocalConfig {
    pub desktop_enabled: bool,
    pub voice_enabled: bool,
    pub voice: VoiceMessages,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct VoiceMessages {
    pub complete: String,
    pub confirm: String,
    pub idle: String,
    pub elicitation: String,
    pub warning: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct EventConfig {
    pub complete: bool,
    pub confirm: bool,
    pub warning: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct MessageConfig {
    pub include_raw: bool,
    pub raw_max_chars: usize,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct TelegramConfig {
    pub enabled: bool,
    pub api_base_url: String,
    pub proxy_url: String,
    pub timeout_ms: u64,
    pub instances: Vec<TelegramInstance>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct TelegramInstance {
    pub name: String,
    pub enabled: bool,
    pub bot_token: String,
    pub chat_id: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct FeishuConfig {
    pub enabled: bool,
    pub proxy_url: String,
    pub timeout_ms: u64,
    pub instances: Vec<FeishuInstance>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct FeishuInstance {
    pub name: String,
    pub enabled: bool,
    pub webhook_url: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct AiSummaryConfig {
    pub enabled: bool,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub proxy_url: String,
    pub timeout_ms: u64,
    pub max_input_chars: usize,
    pub max_output_chars: usize,
    pub system_prompt: String,
    pub user_prompt: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,
            enabled: false,
            debug: false,
            min_duration_seconds: 0,
            local: LocalConfig::default(),
            events: EventConfig::default(),
            message: MessageConfig::default(),
            telegram: TelegramConfig::default(),
            feishu: FeishuConfig::default(),
            ai_summary: AiSummaryConfig::default(),
        }
    }
}

impl Default for LocalConfig {
    fn default() -> Self {
        Self {
            desktop_enabled: true,
            voice_enabled: true,
            voice: VoiceMessages::default(),
        }
    }
}

impl Default for VoiceMessages {
    fn default() -> Self {
        Self {
            complete: "任务完成".to_string(),
            confirm: "需要权限确认".to_string(),
            idle: "等待你的输入".to_string(),
            elicitation: "需要输入信息".to_string(),
            warning: "任务遇到问题".to_string(),
        }
    }
}

impl VoiceMessages {
    pub fn for_event(&self, event: EventKind) -> &str {
        match event {
            EventKind::Complete => &self.complete,
            EventKind::Confirm => &self.confirm,
            EventKind::Idle => &self.idle,
            EventKind::Elicitation => &self.elicitation,
            EventKind::Warning => &self.warning,
        }
    }
}

impl Default for EventConfig {
    fn default() -> Self {
        Self {
            complete: true,
            confirm: true,
            warning: true,
        }
    }
}

impl EventConfig {
    pub fn enabled_for(&self, event: EventKind) -> bool {
        match event {
            EventKind::Complete => self.complete,
            EventKind::Warning => self.warning,
            EventKind::Confirm | EventKind::Idle | EventKind::Elicitation => self.confirm,
        }
    }
}

impl Default for MessageConfig {
    fn default() -> Self {
        Self {
            include_raw: false,
            raw_max_chars: 500,
        }
    }
}

impl Default for TelegramConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_base_url: "https://api.telegram.org".to_string(),
            proxy_url: String::new(),
            timeout_ms: 5_000,
            instances: vec![TelegramInstance {
                name: "default".to_string(),
                ..Default::default()
            }],
        }
    }
}

impl Default for FeishuConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            proxy_url: String::new(),
            timeout_ms: 5_000,
            instances: vec![FeishuInstance {
                name: "default".to_string(),
                ..Default::default()
            }],
        }
    }
}

impl Default for AiSummaryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            base_url: "https://api.deepseek.com".to_string(),
            api_key: String::new(),
            model: "deepseek-chat".to_string(),
            proxy_url: String::new(),
            timeout_ms: 5_000,
            max_input_chars: 4_000,
            max_output_chars: 500,
            system_prompt: DEFAULT_SYSTEM_PROMPT.to_string(),
            user_prompt: DEFAULT_USER_PROMPT.to_string(),
        }
    }
}
