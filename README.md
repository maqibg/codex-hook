# codex-hook

Codex CLI 任务完成通知工具。监听 `agent-turn-complete` 事件，支持 AI 摘要 + Telegram / 飞书 / Windows 桌面通知 / 语音播报。

## 快速开始

**方式一：下载 Release**

1. 从 [Releases](https://github.com/maqibg/codex-hook/releases) 下载 zip
2. 解压得到 `codex-hook.exe` 和 `.env.example`
3. 将 `.env.example` 复制为 `.env`，填入实际配置
4. 在 `~/.codex/config.toml` 中添加配置（见下方）

**方式二：源码编译**

```powershell
git clone https://github.com/maqibg/codex-hook.git
Set-Location codex-hook
cargo build --release
Copy-Item .env.example target/release/.env
# 编辑 target/release/.env 填入实际配置
```

## Codex CLI 配置

`~/.codex/config.toml`，路径替换为实际 exe 位置：

```toml
notify = ["/path/to/codex-hook.exe"]
```

## .env 配置

程序只读取 exe 同目录的 `.env`，模板为 `.env.example`。配置 Telegram 或飞书实例后自动启用远程通知；本地桌面通知和语音由各自变量独立控制。

`TELEGRAM_PROXY_URL` 支持带自定义端口的 HTTP、HTTPS、SOCKS5 或 SOCKS5H 代理，例如 `http://127.0.0.1:7890`。未设置渠道专用代理时，会兼容回退到旧版 `HTTPS_PROXY` 或 `HTTP_PROXY`。

AI 只对 `agent-turn-complete` 生成摘要，`approval-requested` 使用确定性确认文本；单个远程实例失败不影响其他实例或 Codex。`MESSAGE_INCLUDE_RAW` 默认关闭。

**基础配置：**

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `DEBUG` | 调试日志（输出到 stderr） | `false` |

**事件与消息：**

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `EVENT_COMPLETE` | 发送任务完成事件 | `true` |
| `EVENT_CONFIRM` | 发送权限确认事件 | `true` |
| `EVENT_WARNING` | 发送警告事件 | `true` |
| `MESSAGE_INCLUDE_RAW` | 在远程通知中附带原始输出 | `false` |
| `MESSAGE_RAW_MAX_CHARS` | 原始输出字符上限 | `500` |

**Telegram 网络：**

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `TELEGRAM_PROXY_URL` | Telegram 代理完整 URL，端口可自定义 | 空 |
| `TELEGRAM_API_BASE_URL` | Telegram API 地址 | `https://api.telegram.org` |
| `TELEGRAM_TIMEOUT_MS` | Telegram 请求超时（毫秒） | `5000` |
| `HTTPS_PROXY` / `HTTP_PROXY` | 渠道未设置专用代理时的兼容回退 | 空 |

**AI 摘要：**

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `AI_ENABLE` | 启用 AI 摘要 | `true` |
| `AI_API_KEY` | API Key | - |
| `AI_BASE_URL` | API 地址（自动补全 /v1） | `https://api.deepseek.com` |
| `AI_MODEL` | 模型名 | `deepseek-chat` |
| `AI_MAX_INPUT_CHARS` | 发送给 AI 的输入字符上限 | `4000` |
| `AI_MAX_OUTPUT_CHARS` | 摘要输出字符上限 | `500` |
| `AI_TIMEOUT_MS` | AI 请求超时（毫秒） | `5000` |
| `AI_PROXY_URL` | AI 独立代理地址 | 空 |
| `AI_SYSTEM_PROMPT` | AI 系统提示词（定义角色和格式） | 内置默认 |
| `AI_USER_PROMPT` | AI 用户提示词（`{max_output_chars}`/`{content}` 自动替换） | 内置默认 |

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

飞书可通过 `FEISHU_PROXY_URL` 设置独立代理，通过 `FEISHU_TIMEOUT_MS` 设置请求超时；默认超时为 `5000` 毫秒。

## 手动测试

```powershell
codex-hook.exe '{"type":"agent-turn-complete","cwd":"C:\\work\\demo","session-id":"test-session","last-assistant-message":"测试完成"}'
codex-hook.exe '{"type":"approval-requested","reason":"需要文件写入权限"}'
```

通知行为和远程发送语义参考了 HelloAGENTS；详见 `NOTICE`。

## License

MIT
