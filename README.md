# codex-hook

Codex CLI 任务完成通知工具。监听 `agent-turn-complete` 事件，支持 AI 摘要 + Telegram / 飞书 / Windows 桌面通知 / 语音播报。

## 快速开始

**方式一：下载 Release**

1. 从 [Releases](https://github.com/maqibg/codex-hook/releases) 下载 zip
2. 解压得到 `codex-hook.exe` 和 `.env.example`
3. 将 `.env.example` 重命名为 `.env`，填入实际配置
4. 在 `~/.codex/config.toml` 中添加配置（见下方）

**方式二：源码编译**

```bash
git clone https://github.com/maqibg/codex-hook.git
cd codex-hook
cargo build --release
cp .env.example target/release/.env
# 编辑 target/release/.env 填入实际配置
```

## Codex CLI 配置

`~/.codex/config.toml`，路径替换为实际 exe 位置：

```toml
notify = ["/path/to/codex-hook.exe"]
```

## .env 配置

`.env` 文件放在 exe 同目录，详见 `.env.example`。

**基础配置：**

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `DEBUG` | 调试日志（输出到 stderr） | `false` |
| `HTTPS_PROXY` | 代理地址（Telegram 需要） | 空 |

**AI 摘要：**

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `AI_ENABLE` | 启用 AI 摘要 | `true` |
| `AI_API_KEY` | API Key | - |
| `AI_BASE_URL` | API 地址（自动补全 /v1） | `https://api.deepseek.com` |
| `AI_MODEL` | 模型名 | `deepseek-chat` |
| `AI_MAX_WORDS` | 摘要字数上限 | `500` |
| `AI_SYSTEM_PROMPT` | AI 系统提示词（定义角色和格式） | 内置默认 |
| `AI_USER_PROMPT` | AI 用户提示词（`{max_words}`/`{content}` 自动替换） | 内置默认 |

**Windows 通知与语音：**

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `WIN_NOTIFY_ENABLE` | 桌面通知 | `true` |
| `VOICE_ENABLE` | 语音播报 | `true` |
| `VOICE_STOP` | 任务完成语音 | `Codex任务完成` |

**渠道配置**（前缀索引，可配多个实例）：

```env
# Telegram：TG_1_*、TG_2_*...
TG_1_ENABLE=true
TG_1_NAME=通知1
TG_1_TOKEN=<从 @BotFather 获取>
TG_1_CHAT_ID=<从 @userinfobot 获取>

# 飞书：FS_1_*、FS_2_*...
FS_1_ENABLE=true
FS_1_NAME=飞书通知
FS_1_WEBHOOK_URL=<飞书群自定义机器人 Webhook URL>
```

## 手动测试

```bash
codex-hook.exe '{"type":"agent-turn-complete","last-assistant-message":"测试完成"}'
```

## License

MIT
