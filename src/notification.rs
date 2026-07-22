use crate::channels::{feishu, telegram, windows};
use crate::config::Config;
use crate::event::{EventKind, NotificationRequest, RenderedNotification};
use crate::http::build_client;
use crate::summarizer::{generate, local_summary, truncate_chars};
use tokio::task::JoinSet;

#[derive(Debug, Default)]
pub struct DeliveryReport {
    pub sent: usize,
    pub failed: usize,
    pub skipped: bool,
}

pub async fn dispatch(config: &Config, request: NotificationRequest) -> DeliveryReport {
    let local_text = local_summary(&request.content, config.ai_summary.max_output_chars);
    let local_text = if local_text.is_empty() {
        request.content.clone()
    } else {
        local_text
    };
    if config.local.desktop_enabled {
        windows::notify(
            &request.title,
            &with_extra(&local_text, request.extra.as_deref()),
        );
    }
    if config.local.voice_enabled {
        windows::speak(config.local.voice.for_event(request.event));
    }

    if !config.remote_enabled_for(request.event) {
        return DeliveryReport {
            skipped: true,
            ..Default::default()
        };
    }
    let telegram_targets = config.ready_telegram_instances();
    let feishu_targets = config.ready_feishu_instances();
    if telegram_targets.is_empty() && feishu_targets.is_empty() {
        return DeliveryReport {
            skipped: true,
            ..Default::default()
        };
    }

    let summary = if request.event == EventKind::Complete {
        generate(&config.ai_summary, &request.content, config.debug).await
    } else {
        local_text
    };
    let raw = if config.message.include_raw && request.event == EventKind::Complete {
        truncate_chars(&request.content, config.message.raw_max_chars)
    } else {
        String::new()
    };
    let rendered = RenderedNotification {
        event: request.event,
        title: request.title,
        summary,
        raw,
        extra: request.extra,
    };

    let mut tasks = JoinSet::new();
    let mut report = DeliveryReport::default();
    match build_client(&config.telegram.proxy_url, config.telegram.timeout_ms) {
        Ok(client) => {
            for instance in telegram_targets {
                let client = client.clone();
                let channel_config = config.telegram.clone();
                let notification = rendered.clone();
                tasks.spawn(async move {
                    let result =
                        telegram::send(&client, &channel_config, &instance, &notification).await;
                    ("telegram", instance.name, result)
                });
            }
        }
        Err(error) => {
            for instance in telegram_targets {
                report.failed += 1;
                eprintln!(
                    "[codex-hook] telegram {} 失败: {}",
                    instance.name, error.code
                );
            }
        }
    }
    match build_client(&config.feishu.proxy_url, config.feishu.timeout_ms) {
        Ok(client) => {
            for instance in feishu_targets {
                let client = client.clone();
                let notification = rendered.clone();
                tasks.spawn(async move {
                    let result = feishu::send(&client, &instance, &notification).await;
                    ("feishu", instance.name, result)
                });
            }
        }
        Err(error) => {
            for instance in feishu_targets {
                report.failed += 1;
                eprintln!("[codex-hook] feishu {} 失败: {}", instance.name, error.code);
            }
        }
    }

    while let Some(result) = tasks.join_next().await {
        match result {
            Ok((_, _, Ok(()))) => report.sent += 1,
            Ok((channel, name, Err(error))) => {
                report.failed += 1;
                eprintln!("[codex-hook] {channel} {name} 失败: {}", error.code);
            }
            Err(_) => report.failed += 1,
        }
    }
    report
}

fn with_extra(message: &str, extra: Option<&str>) -> String {
    match extra {
        Some(extra) if !extra.is_empty() => format!("{message}\n{extra}"),
        _ => message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extra_is_only_added_when_present() {
        assert_eq!(with_extra("done", None), "done");
        assert_eq!(with_extra("done", Some("耗时 2s")), "done\n耗时 2s");
    }
}
