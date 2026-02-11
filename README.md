# codex-hook

Codex CLI 任务完成通知工具。监听 `agent-turn-complete` 事件，支持 AI 摘要 + Telegram / 飞书 / Windows 桌面通知 / 语音播报。

## 构建

```bash
cargo build --release
```

## 配置

1. 复制 `.env.example` 为 `target/release/.env`
2. 编辑 `.env` 填入实际配置

## Codex CLI 集成

`~/.codex/config.toml`：

```toml
notify = ["D:/Code/codeSpace/Notice/codex-hook/target/release/codex-hook.exe"]
```

## 手动测试

```bash
codex-hook.exe '{"type":"agent-turn-complete","last-assistant-message":"测试完成"}'
```
