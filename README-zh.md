# Markdown Reader

[English](./README.md)

极致轻量级+秒级冷启动 Windows Markdown Reader MVP。只读打开本地 `.md` / `.markdown` 文件，不做编辑、保存、云同步、插件、文件管理器或后台常驻。
![1](./neicun.png)
## MVP

- Tauri v2 + 原生 HTML/CSS/JS，不使用 Electron、Vite 和前端框架。
- Rust 本地读取 Markdown，生成 HTML、TOC、标题锚点和代码高亮。
- 默认 Catppuccin Mocha。
- 命令行打开：`md-reader.exe path\to\file.md`。
- 打包配置预留 `.md` / `.markdown` Windows 文件关联。

## 美观

- 语法高亮
![2](./gaoliang.png)

- letax数学公式渲染
![3](./ecoic.png)

## 依赖说明

- `tauri` / `tauri-build`：Windows 桌面壳、命令调用和打包能力。
- `pulldown-cmark`：轻量 Markdown 解析。
- `syntect`：本地语法高亮，无网络请求。
- `ammonia`：HTML 清理，限制脚本和不安全协议。
- `base64` / `mime_guess`：把本地相对图片内联为 data URL，避免前端直接读文件。
- `serde` / `thiserror` / `html-escape`：数据传输、错误和 HTML 转义。

## 验证

```powershell
pnpm install
cargo check --manifest-path src-tauri\Cargo.toml
pnpm build
```
