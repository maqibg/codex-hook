use std::process::Command;
use crate::config::Config;

pub fn notify(title: &str, message: &str) {
    let msg = if message.chars().count() > 200 {
        format!("{}...", message.chars().take(200).collect::<String>())
    } else {
        message.to_string()
    };
    let _ = notify_rust::Notification::new()
        .summary(title)
        .body(&msg)
        .appname("codex-hook")
        .timeout(notify_rust::Timeout::Milliseconds(10000))
        .show();
}

/// 语音播报
pub fn speak(cfg: &Config) {
    if !cfg.voice_enable || cfg.voice_stop.is_empty() { return; }
    let script = format!(
        "Add-Type -AssemblyName System.Speech; (New-Object System.Speech.Synthesis.SpeechSynthesizer).Speak('{}')",
        cfg.voice_stop
    );
    let _ = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}
