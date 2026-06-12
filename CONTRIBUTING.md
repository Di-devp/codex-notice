# 贡献指南

感谢你关注 Codex Notice。

## 开发准备

```bash
pnpm install
```

启动开发模式：

```bash
pnpm tauri:dev
```

运行前端构建：

```bash
pnpm build
```

运行 Rust 测试：

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

## 提交 PR 前请确认

- 不提交 `node_modules/`、`dist/`、`src-tauri/target/`、`.dmg` 等生成物。
- 不提交真实飞书 Webhook、签名密钥、token、数据库文件。
- 涉及 Codex hooks 的改动，需要说明对 `~/.codex/config.toml` 的影响。
- 涉及通知内容的改动，需要确认敏感信息会经过脱敏。
- 涉及 UI 的改动，需要兼顾中英文文案。

## 代码风格

- 前端使用 Vue 3 + TypeScript。
- 后端使用 Rust，提交前运行 `cargo fmt`。
- 尽量保持改动聚焦，不把无关重构混在功能修复里。

## Issue 建议

提交问题时，请尽量提供：

- macOS 版本
- Codex 版本或使用方式
- Notice 版本
- 复现步骤
- 是否已安装 Codex Hooks
- `curl -sS http://127.0.0.1:3746/health` 的结果

