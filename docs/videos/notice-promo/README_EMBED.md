# GitHub README 嵌入片段

GitHub README 不适合直接引用仓库内的大 MP4 文件。推荐做法：

1. 打开 GitHub 的 Issue、Release 或 README 编辑器。
2. 将 `docs/videos/notice-promo/renders/notice-promo-16x9.mp4` 拖进去上传。
3. GitHub 会生成一个 `https://github.com/user-attachments/assets/...` 链接。
4. 把链接单独放在 README 中即可自动展示视频播放器。

示例：

```md
## 项目演示

https://github.com/user-attachments/assets/<uploaded-video-id>
```

如果需要短视频平台版本，上传：

```text
docs/videos/notice-promo-vertical/renders/notice-promo-9x16.mp4
```
