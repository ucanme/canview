<div align="center">

# can-viewer

![can-viewer Logo](assets/svg/logo-256x256.svg)

**开源跨平台 CAN/LIN 总线数据分析工具**

[![Build](https://img.shields.io/github/actions/workflow/status/cantool/can-viewer/build.yml?branch=main&style=flat-square)](https://github.com/cantool/can-viewer/actions)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE.txt)
[![Rust](https://img.shields.io/badge/rust-nightly-orange?style=flat-square)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Windows%20|%20macOS%20|%20Linux-lightgrey?style=flat-square)](https://github.com/cantool/can-viewer/releases)

完全开源，你的 ⭐ 是我持续开发的动力！

[English](README.md) | [中文文档](README_zh.md)

</div>

---

## 简介

can-viewer 是用 Rust 构建的汽车总线数据分析工具，集成 BLF 日志解析、DBC/LDF 信号库管理和 [GPUI](https://gpui.rs/) GPU 加速界面。

- 🚀 **Rust 原生** — 零拷贝解析，启动快
- ⚡ **并行 BLF 加载** — 通过 rayon 跨核 zlib 解压;9 MB / 86 万对象文件 ~280ms 加载完成
- 🖥️ **GPU 加速 UI** — 基于 GPUI，渲染流畅
- 📊 **交互式波形图** — 缩放、平移、悬停查看
- 📚 **信号库管理** — 多版本 DBC/LDF 库，一键激活，自动加载
- 🎯 **信号集** — 在某个信号库上保存命名的"通道+消息+信号"组合,批量应用到波形侧栏快速回放同一组信号
- 🔗 **局域网共享** — 通过 HTTP 在局域网内共享信号库，一键导入
- 🔌 **多总线支持** — CAN / CAN FD / LIN / FlexRay / Ethernet
- 🖱️ **拖拽加载** — 把 `.blf` 文件(或包含它们的文件夹)直接拖到窗口;macOS 上拖到 Dock 图标也可加载
- 📁 **多文件按真实时间合并** — 并行加载多个 BLF,消息按绝对时间戳(`measurement_start_time + object_time_stamp`)全局排序,不同时间录制的文件能正确对齐
- ⚠️ **解析错误提示** — 失败的解析在状态栏显示 ⚠️ 警告;Loaded Files 浮框列出每个文件的 ❌ 错误详情
- 🌐 **跨平台** — Windows、macOS、Linux

---

## 截图

| 日志浏览 | 信号绘图 |
|:---:|:---:|
| ![BLF 日志查看器](assets/blf_logs_screenshot.png) | ![信号折线图](assets/plot_screen.png) |

| 信号库管理 |
|:---:|
| ![信号库管理](assets/dbc_library.png) |

---

## 快速开始

**环境要求：** Rust nightly、Git

\`\`\`bash
# Linux 额外依赖
sudo apt-get install libxkbcommon-dev libx11-dev libegl1-mesa-dev

# 构建运行
git clone https://github.com/cantool/can-viewer.git
cd can-viewer
cargo run --release --bin viewer
\`\`\`

预编译包：[Releases](https://github.com/cantool/can-viewer/releases)（Windows / macOS / Linux）

---

## 使用

1. **打开日志** — 点击 File → Open BLF…(单文件)或 Open Multiple BLF…(追加)。也可以**直接拖拽** `.blf` 文件到窗口 — 拖拽会先清空当前会话再加载。macOS 上拖到 Dock 图标同样可用。
2. **管理信号库** — 切换到 Library 标签页，添加 DBC/LDF 文件，激活版本
3. **信号集** — 在某个信号库上把通道+消息+信号组合保存为命名集合。从波形侧栏顶部打开集合下拉,可批量应用(一次勾选整组)、重命名或删除 — 同一组信号在不同录制间快速回放很方便。
4. **浏览日志** — 切换到 Log 标签页查看解码后的信号值
5. **绘制波形** — 切换到 Signal Plot，选择信号，支持缩放和悬停查看
6. **管理已加载文件** — 当加载 ≥ 2 个文件(或任意文件有解析错误)时,点击左下角状态栏的 **📂 N files** 段打开 Loaded Files 浮框。每行 ✕ 可单独移除,或 **Remove All** 全部清空。

拖拽规则:仅接受 `.blf`，文件夹展开一层;总大小 > 1 GB 弹确认对话框;加载中再拖新文件会取消当前加载并排队新文件。

配置自动保存至 `multi_channel_config.json`。

📖 **完整使用文档：** [docs/USAGE_zh.md](docs/USAGE_zh.md) — 窗口布局、日志/波形视图、信号库管理、信号集、局域网共享、键盘快捷键、故障排查。

---

## 路线图

### 已完成

- [x] BLF 解析核心（CAN / CAN FD / LIN / FlexRay / Ethernet）
- [x] DBC / LDF 数据库解析
- [x] GPUI 桌面 UI（GPU 加速,跨平台）
- [x] 消息过滤（按 ID 和通道）与信号解码
- [x] 信号波形图 — 缩放、悬停、绝对时间、选中信号实时绘图、无数据占位
- [x] 多版本信号库管理（创建/激活/局域网共享/一键导入）
- [x] 信号集 — 在信号库上保存命名的"通道+消息+信号"组合,批量应用到波形
- [x] 多文件加载按时间合并（Open BLF... 替换，Open Multiple BLF... 追加）
- [x] 拖拽加载 BLF（窗口内 + macOS Dock 图标）,文件夹展开,大文件守卫
- [x] 状态栏多文件段 + ⚠️ 解析错误提示 + Loaded Files 浮框
- [x] 并行 BLF 加载（rayon 跨核 zlib 解压 + 索引排序合并,多核 CPU 上 ~3× 提速）

### 计划中

- [ ] 实时流模式（socketcan / 虚拟总线）
- [ ] 导出 CSV / JSON
- [ ] 诊断规则 DSL（自定义告警/错误条件）
- [ ] 根据观测流量生成 DBC
- [ ] 回放 / 重发（把消息发回总线）

---

## 开发

\`\`\`bash
cargo test --workspace   # 运行测试
cargo fmt --all          # 格式化
cargo clippy --workspace # 静态检查
\`\`\`

跨平台构建详见 [BUILD.md](BUILD.md)。

---

## 贡献

欢迎 Fork 并提交 Pull Request。请确保代码通过 `cargo fmt` 和 `cargo clippy` 检查。

---

## 许可证

[MIT License](LICENSE.txt) © 2026 can-viewer

---

<div align="center">

**使用 Rust 用 ❤️ 构建**

[Issues](https://github.com/cantool/can-viewer/issues) · [Discussions](https://github.com/cantool/can-viewer/discussions) · admin@ucan.me

</div>
