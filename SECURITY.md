# 安全说明

Codex Notice 会处理 Codex hook payload、本地命令信息和通知渠道配置，因此安全边界很重要。

## 本地边界

- 本地 HTTP 服务只监听 `127.0.0.1:3746`。
- Hook 请求必须携带本地生成的 `X-Notice-Token`。
- 飞书 Webhook 和签名密钥使用 macOS Keychain 保存。
- SQLite 中只保存非敏感配置和脱敏展示信息。

## 请不要提交

- 飞书 Webhook
- 飞书签名密钥
- Codex / OpenAI / 其他服务的 API Key
- 本地数据库
- Hook token
- 打包产物中的个人签名材料

## 报告安全问题

如果你发现可能导致敏感信息泄露、越权审批、远程请求绕过本地鉴权的问题，请不要公开贴出可利用细节。

你可以先在 GitHub Issue 中描述影响范围，避免附带真实密钥或真实 payload。项目维护者确认后再进一步沟通细节。

