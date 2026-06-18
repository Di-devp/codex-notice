# Notice 项目介绍/推广视频方案

## 成片定位

- 横屏主版本：`index.html`，16:9，1920x1080，48 秒，适合 GitHub README、官网首页、B 站/YouTube。
- 竖屏适配版：`../notice-promo-vertical/index.html`，9:16，1080x1920，48 秒，适合短视频平台二次渲染。
- 风格：macOS 原生感、开发者工具感、轻科技、简洁、专业。
- 核心叙事：Notice 不关心每一次工具调用，而是关注“这一轮 Codex 对话任务”的整体状态。

## 素材使用

| 用途 | 文件 | 状态 |
| --- | --- | --- |
| 运行中仪表盘 | `assets/notice/running.png` | 已使用 |
| 已完成仪表盘 | `assets/notice/done.png` | 已使用 |
| App 图标 | `assets/notice/icon.png` | 已使用 |
| 飞书审批通知 | CSS 脱敏卡片 | 已使用 |
| 飞书完成通知 | CSS 脱敏卡片 | 已使用 |
| 黄灯审批独立截图 | CSS 占位状态卡 | 需要后续替换真实截图 |
| 红灯失败独立截图 | CSS 占位状态卡 | 需要后续替换真实截图 |
| Codex 正在执行任务截图 | CSS 脱敏模拟窗口 | 可后续替换真实 Codex 截图 |

## 分镜脚本

| 时间 | 画面 | 字幕 | 动效 | 素材 |
| --- | --- | --- | --- | --- |
| 0-6s | 深色 macOS 桌面，左侧模拟 Codex 对话/终端任务流，右侧状态模糊 | `Codex 在跑任务时，你真的知道它现在卡在哪一步吗？` | 轻微推近，命令行状态逐行淡入 | CSS 脱敏模拟 Codex 窗口 |
| 6-12s | Notice 红绿灯小组件浮入，三色灯开始跑马灯 | `Notice：让 Codex 任务状态一眼可见` | 小组件从右侧滑入，灯光轮询 | CSS 小组件 |
| 12-20s | 展示真实 Notice 运行中仪表盘，叠加 UserPromptSubmit 事件标签 | `新任务开始，自动进入运行态` | 截图轻微推近，Running 标签出现 | `running.png` |
| 20-28s | 黄灯亮起，飞书审批通知卡片滑入 | `需要你确认时，才提醒你` | 黄灯聚焦，通知从右侧滑入 | CSS 脱敏飞书审批卡片 |
| 28-36s | 红灯失败占位卡，随后新任务开始，红灯清除并回到跑马灯 | `失败清晰可见，重新发起后自动恢复` | 红灯短亮，状态切换为 Running | CSS 失败占位状态卡 |
| 36-44s | 真实已完成仪表盘，绿灯亮起，飞书完成通知滑入 | `任务完成，再通知你` | 绿灯亮起，通知淡入 | `done.png` + CSS 脱敏飞书完成卡片 |
| 44-48s | Notice 图标和产品名收尾 | `Codex 任务状态，一眼看清。` / `少一点打扰，多一点确定感。` | 图标淡入，红黄绿微光收束 | `icon.png` |

## 可替换截图位置

视频内有 3 个占位区域可以后续替换为真实截图：

1. 第一幕的 Codex 执行中画面：当前为 CSS 脱敏模拟窗口。
2. 第四幕的审批黄灯界面：当前为 CSS 状态卡和脱敏飞书通知。
3. 第五幕的失败红灯界面：当前为 CSS 状态卡。

替换方式：把对应截图放到 `assets/notice/`，然后在 `index.html` / `vertical.html` 里替换对应 `.mock-*` 区块为 `<img>`。

## 运行命令

```bash
cd docs/videos/notice-promo
npm run check
npm run render -- --output renders/notice-promo-16x9.mp4 --quality standard
```

预览：

```bash
cd docs/videos/notice-promo
npm run dev
```

README 嵌入建议：

```md
https://github.com/user-attachments/assets/<uploaded-video-id>
```

GitHub README 不能直接播放仓库内大视频文件，建议把渲染后的 MP4 拖到 GitHub Issue/Release/README 编辑器上传，得到 `user-attachments` 链接后嵌入。
