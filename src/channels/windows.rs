use std::process::Command;

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

pub fn speak(text: &str) {
    if text.is_empty() {
        return;
    }
    let escaped = text.replace('\'', "''");
    let script = format!(
        "Add-Type -AssemblyName System.Speech; (New-Object System.Speech.Synthesis.SpeechSynthesizer).Speak('{escaped}')"
    );
    let _ = Command::new("powershell.exe")
        .args(["-NoProfile", "-Command", &script])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}
