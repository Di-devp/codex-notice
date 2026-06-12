# Notice - Developer Notification Center

## 项目简介

Notice 是一个运行在 macOS 菜单栏中的开发者通知中心（Developer Notification Center）。

用于统一接收 AI Agent、开发工具、CI/CD 系统以及自动化任务的状态通知，并通过飞书、企业微信、邮件等渠道实时推送给开发者。

Notice 的目标不是成为某个工具的插件，而是成为开发者工作流中的统一通知中枢。

------

# 当前最容易被忽略的问题

在进入开发前，需要先把以下边界写清楚，否则后续实现很容易返工：

- **Codex Hook 兼容性**：Hook 事件、配置格式和返回协议必须以当前 Codex 官方版本为准，不能假设固定存在 `hooks.json` 或某个开关名。
- **本地服务鉴权**：即使只监听 `127.0.0.1`，同一台机器上的其他进程仍然可以伪造请求，因此需要本地 shared secret 或一次性 token。
- **远程审批边界**：第一阶段只做“通知 + 回到电脑处理”，不要承诺手机端远程审批；远程审批涉及鉴权、重放防护、超时、审计和误触风险。
- **高风险命令识别**：不能只靠关键词包含，例如 `delete` 过宽、`rm -rf "$DIR"` 不一定能被简单字符串匹配；需要命令解析、规则等级和误报处理。
- **通知可靠性**：飞书 Webhook 失败、限流、超时、重复发送都需要策略，否则关键通知可能丢失或轰炸。
- **敏感信息泄露**：Hook payload、命令参数、环境变量、日志、通知正文都可能包含 token，需要统一脱敏管道。
- **数据保留策略**：最近 100 条事件不足以支撑排障，需要区分 UI 展示数量和数据库保留周期。
- **Hook 安装安全**：修改用户 `~/.codex` 配置前必须备份、可回滚、可检测冲突，不能覆盖用户已有 hook。

------

# 项目定位

一句话描述：

> Notice 是一个面向 AI Agent 与开发工具的统一通知中心。

解决的问题：

开发者在使用 Codex、Trae、Claude Code、Cursor 等工具时，经常会遇到：

- 任务执行完成
- 任务执行失败
- Agent 请求用户确认
- 高风险命令执行
- 长时间任务运行
- 部署成功或失败

由于开发者并不总是守在电脑前，因此希望能够通过飞书等方式实时接收通知。

Notice 提供统一的通知接入与转发能力。

------

# 产品目标

## 第一阶段

支持：

- Codex
- 飞书

实现：

- 任务完成通知
- 任务失败通知
- 用户确认通知

形成完整闭环：

```text
Codex
  ↓
Hook
  ↓
Notice
  ↓
飞书
  ↓
手机通知
```

第一阶段明确不做：

- 手机端远程批准或拒绝命令
- 公网 Webhook 接入
- 多设备同步
- 多用户账号体系
- CI/CD 平台接入
- AI 失败原因分析

第一阶段成功标准：

- 能可靠安装、检测、卸载 Codex Hook，且不破坏用户已有配置
- Codex 任务结束、失败、需要用户确认时能进入 Notice 本地事件库
- 飞书通知发送失败时有重试、记录和 UI 可见状态
- 所有通知正文经过敏感信息脱敏
- 退出 Notice 后不会留下不可控的后台进程或失效 Hook

------

## 第二阶段

新增：

- Claude Code
- Trae

通知渠道：

- 企业微信

------

## 第三阶段

新增：

- GitHub Actions
- Jenkins
- Docker

------

## 第四阶段

新增：

- 飞书交互卡片
- 远程审批
- 多设备同步

------

# 总体架构

```text
              Codex
                 │
                 ▼

            Hook Script

                 │
                 ▼

 ┌─────────────────────────────┐
 │           Notice            │
 │                             │
 │   Provider Layer            │
 │   Rule Engine               │
 │   Notification Center       │
 │                             │
 └─────────────────────────────┘

          │             │
          ▼             ▼

       飞书          企业微信

          │             │
          ▼             ▼

        手机通知      手机通知
```

------

# 技术选型

## 桌面端

- Tauri 2
- Rust

## 前端

- Vue3
- TypeScript
- Vite
- Pinia

## UI

推荐：

- Naive UI

原因：

- 与 Tauri 兼容性好
- 包体积小
- 深色模式支持完善

------

# 核心模块设计

## Provider Layer

负责接收不同来源的事件。

统一转换为 NoticeEvent。

### 第一期支持

```text
Codex Provider
Webhook Provider
```

MVP 中 Webhook Provider 仅作为本机调试入口，默认只接受：

```text
127.0.0.1
```

并要求请求携带本地生成的 shared secret。

不要在第一阶段暴露公网监听能力。

### 后续扩展

```text
Trae Provider
Claude Provider
Cursor Provider
GitHub Provider
Jenkins Provider
Docker Provider
```

------

## Rule Engine

负责判断哪些事件需要通知。

例如：

```text
任务完成
需要通知

任务开始
不通知

连续失败10次
合并通知
```

Rule Engine 需要区分三类动作：

```text
record
只记录，不通知

notify
立即通知

aggregate
进入聚合窗口，等待 Stop/超时/失败阈值触发
```

默认规则建议：

- `SessionStart`：只记录
- `PostToolUse` 成功：聚合
- `PostToolUse` 失败：聚合；连续失败达到阈值时通知
- `PermissionRequest` / 用户确认：立即通知
- `Stop`：发送本轮聚合摘要
- 高风险 `PreToolUse`：本地弹窗确认，飞书只提醒“请回到电脑处理”

------

## Notification Center

负责发送消息。

支持：

```text
飞书
企业微信
邮件
Bark
PushDeer
ntfy
Telegram
```

------

# 统一事件模型

```typescript
export interface NoticeEvent {
  id: string

  version: 1

  provider: string

  eventType: string

  sessionId?: string

  runId?: string

  dedupeKey?: string

  title: string

  content: string

  level: "info" | "success" | "warning" | "error"

  project?: string

  cwd?: string

  command?: string

  exitCode?: number

  durationMs?: number

  timestamp: string

  receivedAt: string

  rawPayload?: any
}
```

说明：

- `id`：事件唯一 ID。
- `sessionId`：一次 Codex 会话的稳定标识，用于聚合。
- `runId`：一次 agent turn 或任务执行的标识。
- `dedupeKey`：用于飞书重试和去重，避免重复通知。
- `rawPayload`：只允许本地存储，进入通知前必须脱敏。

------

# 事件类型

```text
TASK_START

TASK_FINISH

TASK_FAIL

USER_CONFIRM

WARNING

ERROR

DEPLOY_SUCCESS

DEPLOY_FAIL

CUSTOM
```

------

# MVP 功能清单

## 配置管理

支持：

- 飞书 Webhook 配置
- 测试通知
- 开关控制

------

## Codex 集成

支持：

- 自动安装 Hook
- 自动检测 Hook
- 自动卸载 Hook

实现注意：

- 安装前读取并备份 `~/.codex` 相关配置。
- 只追加 Notice 管理的配置块，不覆盖用户已有 hook。
- 所有 Notice 写入内容都加稳定标记，方便检测和卸载。
- Hook 命令不要直接写复杂逻辑，优先调用 Notice 提供的独立 wrapper 脚本。
- wrapper 脚本负责读取 Codex hook stdin、调用 `127.0.0.1`、处理超时，并返回 Codex 当前版本支持的 hook 输出格式。
- 如果 Notice 未运行，Hook 不应长期阻塞 Codex；低风险事件快速降级为本地文件队列或直接放行，高风险事件按用户配置决定放行或阻断。

------

## 通知能力

支持：

- 用户确认通知
- 任务完成通知
- 任务失败通知

------

## 本地记录

支持：

- 最近100条事件
- 搜索
- 筛选

------

## 菜单栏应用

支持：

- 常驻菜单栏
- 当前状态显示
- 最近事件查看

------

# 智能聚合设计

避免通知轰炸。

例如：

错误做法：

```text
失败
失败
失败
失败
失败
```

正确做法：

```text
项目：LightFlow

最近5分钟：

执行命令：36次

成功：31次

失败：4次

等待确认：1次
```

------

# 高风险命令识别

默认关键词：

```json
[
  "rm -rf",
  "sudo",
  "chmod 777",
  "drop table",
  "truncate",
  "delete"
]
```

关键词只作为 MVP 的第一层启发式规则，不能作为唯一判断依据。

建议规则分级：

```text
critical
默认需要本地确认，例如 rm -rf /、sudo rm、磁盘格式化、数据库 drop/truncate

high
默认通知并可配置是否确认，例如 chmod 777、删除大量文件、修改系统目录

medium
只记录或聚合，例如普通 delete、git clean、docker prune
```

误报控制：

- `delete` 不能默认作为高风险关键词直接拦截。
- 需要识别命令边界，避免匹配到普通文本。
- 需要允许用户按项目配置白名单。
- 每次拦截都要记录规则命中原因，便于后续调试。

触发通知：

```text
🚨 检测到高风险命令

项目：
lightflow

命令：
rm -rf dist
```

------

# 飞书通知模板

发送策略：

- 超时：单次请求 5 秒。
- 重试：最多 3 次，使用指数退避。
- 去重：同一个 `dedupeKey` 在短窗口内不重复发送。
- 限流：聚合类通知每个项目默认 5 分钟最多 1 条。
- 失败记录：发送失败必须写入事件库，并在 UI 中显示。
- Webhook 安全：支持飞书签名密钥时优先开启签名。

## 用户确认

```text
⚠️ Codex 需要用户确认

项目：
lightflow

操作：
执行 Bash 命令

时间：
2026-06-03 18:00

请返回电脑处理。
```

------

## 任务完成

```text
✅ 任务执行完成

项目：
lightflow

耗时：
3分20秒

时间：
2026-06-03 18:10
```

------

## 任务失败

```text
❌ 任务执行失败

项目：
lightflow

错误：
Exit Code 1

时间：
2026-06-03 18:15
```

------

# 项目结构

```text
notice/

├── frontend
│
├── src-tauri
│
├── providers
│   ├── codex
│   ├── webhook
│   ├── trae
│   ├── claude
│   └── github
│
├── channels
│   ├── feishu
│   ├── wecom
│   ├── email
│   ├── bark
│   └── ntfy
│
├── rules
│
├── storage
│
├── scripts
│
└── logs
```

------

# 页面设计

## Dashboard

展示：

- 当前状态
- 今日事件数量
- 今日成功数量
- 今日失败数量
- 今日确认次数

------

## Events

展示：

- 所有事件记录

支持：

- 搜索
- 筛选
- 清空

------

## Channels

展示：

- 飞书配置
- 企业微信配置
- 测试通知

------

## Providers

展示：

- Codex
- Trae
- Claude
- GitHub

状态：

```text
已连接
未连接
```

------

## Hook Manager

支持：

- 安装 Hook
- 卸载 Hook
- 备份 Hook
- 恢复 Hook

------

# 本地存储

配置文件：

```text
~/Library/Application Support/Notice/config.json
```

数据库：

```text
~/Library/Application Support/Notice/notice.db
```

日志：

```text
~/Library/Application Support/Notice/logs/
```

Hook 文件：

```text
~/Library/Application Support/Notice/hooks/
```

用于保存 Notice 生成的 Hook wrapper、安装备份和卸载元数据。

保留策略：

- UI 默认展示最近 100 条。
- SQLite 默认保留 30 天事件。
- 日志默认保留 7 天。
- 用户可一键清理事件、日志和 Hook 备份。

------

# 安全设计

## Webhook 安全

- 不展示完整地址
- 本地加密存储
- 导出配置时自动脱敏

------

## 本地服务安全

仅监听：

```text
127.0.0.1
```

不开放公网访问。

同时要求：

- 所有本地请求必须带 `X-Notice-Token`。
- token 首次启动时随机生成，存储在系统安全存储或加密配置中。
- Hook wrapper 读取 token 后再调用本地服务。
- 请求体大小需要限制，避免异常 payload 撑爆日志或数据库。
- 对 `rawPayload`、命令输出和错误信息统一脱敏后再入库、展示和发送。

------

## 敏感信息过滤

自动过滤：

```text
token
apikey
secret
password
authorization
```

------

# MVP 开发顺序

## Sprint 1

完成：

- Tauri项目初始化
- 飞书通知发送
- 配置管理

验收：

- 能保存、脱敏展示、测试飞书 Webhook。
- 飞书发送失败有错误提示和本地记录。
- 配置文件不存在、损坏、版本升级时都有明确处理。

------

## Sprint 2

完成：

- 本地HTTP服务
- Hook事件接收
- 事件记录

验收：

- 服务只监听 `127.0.0.1`。
- 无 token、错误 token、大请求体都会被拒绝。
- 事件能写入 SQLite，并能按项目、等级、时间筛选。

------

## Sprint 3

完成：

- Codex Hook安装器
- Hook检测
- Hook卸载

验收：

- 安装前自动备份用户配置。
- 卸载只删除 Notice 管理的配置。
- Notice 未运行时，普通事件不会卡死 Codex。
- Hook 配置格式经过当前 Codex 版本实测确认。

------

## Sprint 4

完成：

- 菜单栏应用
- Dashboard
- Events

------

## Sprint 5

完成：

- 智能聚合
- 高风险命令识别
- 通知限流

验收：

- 连续失败、任务结束、超时都能触发聚合摘要。
- 高风险命令能触发本地确认和飞书提醒。
- 敏感信息不会出现在飞书通知、事件详情和日志中。

------

# 后续规划

## V2

新增：

- Trae
- Claude Code
- 企业微信

------

## V3

新增：

- Jenkins
- GitHub Actions
- Docker

------

## V4

新增：

- 飞书交互卡片
- 远程审批
- AI失败原因分析

------

# 项目愿景

Notice 不只是 Codex 的通知工具。

Notice 将成为：

> 面向 AI Agent、开发工具和自动化平台的统一通知中心（Developer Notification Center）。

通过统一事件模型、统一规则引擎、统一通知渠道，让开发者无需频繁查看 IDE、终端或浏览器，即可实时掌握所有关键任务状态。
