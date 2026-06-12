# Codex Notice

一个面向 Codex 的本地通知、状态灯与审批提醒工具。

Codex Notice 是一个 macOS 桌面应用，用来监听 Codex lifecycle hooks，在本地记录任务状态，并在关键时刻通过飞书提醒你：什么时候需要审批、什么时候一轮任务成功结束。它还提供一个可拖动的红绿灯小组件，让你不用频繁切回 Codex，也能看到当前是否有任务正在运行。

## 功能特性

- Codex Hooks 接入：支持 `SessionStart`、`UserPromptSubmit`、`PermissionRequest`、`PreToolUse`、`PostToolUse`、`Stop`
- 本地状态灯：红绿灯小组件显示运行中、待审批、失败、完成/就绪
- 飞书通知：仅在需要用户审批和任务成功结束时发送，避免工具调用噪声
- 高风险命令审批：对危险命令进入本地审批队列
- 本地事件记录：SQLite 保存事件、投递状态和审批状态
- 本地安全边界：HTTP 服务只监听 `127.0.0.1:3746`，并使用本地 token 鉴权
- 敏感信息保护：飞书 Webhook 和签名密钥存入 macOS Keychain
- Hook 管理器：安装前展示预览，安装时备份 `~/.codex/config.toml`
- 开机自启动：可在设置里开启或关闭登录后自动启动
- 中英文界面：支持英文和简体中文切换

## 当前状态

项目处于 MVP 阶段，目前主要面向 macOS 使用。

已验证的核心能力：

- Codex 本地 hook 事件接收
- 飞书测试通知
- Codex hook 安装/卸载
- 红绿灯小组件拖动、置顶、隐藏
- macOS `.app` 与 `.dmg` 打包

## 技术栈

- 桌面端：Tauri 2 + Rust + Tokio
- 前端：Vue 3 + TypeScript + Vite + Pinia + Naive UI
- 本地服务：Axum
- 数据库：SQLite + SQLx
- 密钥存储：macOS Keychain
- 通知渠道：飞书机器人 Webhook

## 本地开发

### 环境要求

- macOS
- Node.js
- pnpm
- Rust
- Codex

### 安装依赖

```bash
pnpm install
```

### 启动开发模式

```bash
pnpm tauri:dev
```

### 构建前端

```bash
pnpm build
```

### 运行 Rust 测试

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

### 打包 macOS 安装包

```bash
pnpm dist:mac
```

产物位于：

```text
src-tauri/target/release/bundle/
```

## 使用方式

### 1. 启动应用

启动后，Codex Notice 会在本机启动一个本地服务：

```text
127.0.0.1:3746
```

健康检查：

```bash
curl -sS http://127.0.0.1:3746/health
```

正常应返回：

```text
ok
```

### 2. 配置飞书机器人

进入 `Channels` 页面：

1. 填入飞书机器人 Webhook
2. 如启用了飞书签名校验，填入签名密钥
3. 点击保存
4. 点击发送测试，确认手机能收到通知

敏感信息不会明文保存在 SQLite 中。Webhook 和签名密钥会写入 macOS Keychain，界面和数据库只保存脱敏展示值。

### 3. 安装 Codex Hooks

进入 `Providers` 页面：

1. 查看即将写入 `~/.codex/config.toml` 的 Notice 管理块
2. 点击安装
3. 重启 Codex 或新开一个 Codex 会话

安装时会备份原始 Codex 配置，并且只管理 Notice 自己的配置块，不覆盖用户已有 hooks。

### 4. 观察红绿灯小组件

红绿灯状态含义：

- 跑马灯：当前有 Codex 会话正在运行
- 黄灯：存在待审批命令或用户确认
- 红灯：存在失败状态，需要关注
- 绿灯：没有运行中、待审批或失败任务

小组件可以直接拖动。右键小组件可以设置置顶或隐藏。

## 通知策略

Codex Notice 不会把每一次工具调用都发送到飞书。

当前飞书只发送两类通知：

- 需要用户审批
- 一个会话中的任务成功结束

普通工具调用成功、任务中间状态、运行心跳不会发送飞书通知。

## 数据与隐私

Codex Notice 默认只在本机工作：

- HTTP 服务只绑定 `127.0.0.1`
- 本地请求必须携带 `X-Notice-Token`
- SQLite 数据库位于用户应用数据目录
- 飞书密钥使用 macOS Keychain
- 事件内容会经过脱敏处理

默认数据库位置：

```text
~/Library/Application Support/Notice/notice.db
```

## 项目结构

```text
.
├── src/                    # Vue 前端
├── src-tauri/              # Tauri / Rust 后端
│   ├── migrations/         # SQLite migration
│   ├── icons/              # 应用图标
│   └── src/
│       ├── channels/       # 飞书通知
│       ├── commands/       # Tauri commands
│       ├── domain/         # 领域类型
│       ├── hooks/          # Codex hook 安装与配置管理
│       ├── rules/          # 风险规则与脱敏
│       ├── server/         # 本地 Axum HTTP 服务
│       └── storage/        # SQLite 存储
├── docs/                   # 项目文档
└── package.json
```

## 已知限制

- 当前 MVP 主要支持 macOS。
- 飞书是当前唯一内置通知渠道。
- Codex 手动取消任务目前没有独立 lifecycle hook 事件可直接监听；应用通过 `Stop` 事件和工具调用心跳判断状态。
- 分发给其他电脑后，每台电脑都需要在应用内重新安装 Codex Hooks。
- 未签名/未公证的 macOS 包可能会被 Gatekeeper 拦截。

## 路线图

- 更多通知渠道：企业微信、钉钉、Telegram、系统通知
- 更完整的审批体验
- 更细粒度的规则配置
- release 自动构建、签名、公证
- 更完善的安装引导
- 更详细的事件过滤与搜索

## 贡献

欢迎提交 Issue 和 Pull Request。请先阅读 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 安全

如果你发现安全问题，请阅读 [SECURITY.md](SECURITY.md)。

## 许可证

[Apache License 2.0](LICENSE)
