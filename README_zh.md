<div align="center">

# CANVIEW

![CANVIEW Logo](assets/svg/logo-256x256.svg)

**开源跨平台 CAN/LIN 总线数据分析工具**

[![Build](https://img.shields.io/github/actions/workflow/status/ucanme/canview/build.yml?branch=main&style=flat-square)](https://github.com/ucanme/canview/actions)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE.txt)
[![Rust](https://img.shields.io/badge/rust-nightly-orange?style=flat-square)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Windows%20|%20macOS%20|%20Linux-lightgrey?style=flat-square)](https://github.com/ucanme/canview/releases)

完全开源，你的 ⭐ 是我持续开发的动力！

[English](README.md) | [中文文档](README_zh.md)

</div>

---

## 项目简介

CANVIEW 是一个使用 Rust 构建的高性能汽车总线数据分析工具。它集成了 BLF 日志解析、DBC/LDF 信号数据库解析和基于 [GPUI](https://gpui.rs/) 的 GPU 加速桌面界面，为汽车电子工程师提供流畅、现代的信号分析体验。

### 核心亮点

- 🚀 **Rust 原生** — 零拷贝解析，启动和加载速度快
- 🖥️ **GPU 加速 UI** — 基于 Zed 编辑器的 GPUI 框架，渲染流畅
- 📊 **信号绘图** — 交互式波形图，支持缩放、平移、悬停查看
- 🔌 **多总线支持** — CAN / CAN FD / LIN / FlexRay / Ethernet
- 🌐 **跨平台** — Windows、macOS、Linux 一套代码

---

## 截图

| 日志浏览 | 信号绘图 |
|:---:|:---:|
| ![BLF 日志查看器](assets/blf_logs_screenshot.png) | ![信号折线图](assets/plot_screen.png) |

---

## 功能特性

### 日志解析
- 解析 Vector BLF 格式日志文件
- 支持 CAN、CAN FD、LIN、FlexRay、Ethernet 等消息类型
- 支持压缩日志容器，高效处理大文件

### 信号数据库
- 解析 DBC（CAN 信号定义）和 LDF（LIN 信号定义）文件
- 多版本数据库管理，支持库分组与通道映射
- 加载后自动解码信号值

### 桌面界面
- 深色主题，Zed 风格的现代 UI
- 按 ID、通道、消息类型过滤
- HEX / DEC ID 显示切换
- 交互式信号波形图（缩放、拖拽、悬停提示、绝对时间显示）
- 自定义滚动条，流畅滚动
- 状态栏实时显示文件统计

### 支持的消息类型

| 总线 | 类型 |
|------|------|
| CAN | CanMessage, CanMessage2, CanFdMessage, CanFdMessage64 |
| CAN 错误 | CanErrorFrame, CanDriverError, CanDriverStatistic |
| LIN | LinMessage, LinMessage2 |
| FlexRay | 消息、状态、周期事件 |
| Ethernet | Ethernet 帧 |
| 系统 | 应用触发器、注释标记、全局标记 |

---

## 快速开始

### 环境要求

- Rust nightly（项目使用 edition 2024）
- Git

#### Linux 额外依赖

```bash
sudo apt-get install libxkbcommon-dev libx11-dev libegl1-mesa-dev
```

### 从源码构建

```bash
git clone https://github.com/ucanme/canview.git
cd canview
cargo run --release --bin view
```

### 下载预编译包

前往 [Releases](https://github.com/ucanme/canview/releases) 下载：

| 平台 | 架构 |
|------|------|
| Windows | x86_64 |
| macOS | Apple Silicon / Intel |
| Linux | x86_64 |

---

## 使用指南

1. **打开日志** — 点击 "Open BLF File" 选择 `.blf` 文件
2. **加载数据库** — 切换到 Config 标签页，添加 DBC 或 LDF 文件并映射通道
3. **浏览信号** — 回到 Log 标签页查看解码后的信号值
4. **筛选消息** — 点击列表中的 ID 或通道列快速过滤
5. **绘制波形** — 选择信号进入 Chart 视图，支持缩放和悬停查看

配置自动保存至 `canview_config.json`。

---

## 项目结构

```
canview/
├── src/
│   ├── blf/                # BLF 解析库
│   │   └── src/
│   │       ├── objects/    # 消息对象（CAN/LIN/FlexRay/Ethernet/MOST/WLAN）
│   │       ├── parser.rs   # 文件解析器
│   │       └── lib.rs
│   │
│   ├── parser/             # 信号数据库解析库
│   │   └── src/
│   │       ├── dbc.rs      # DBC 解析
│   │       └── ldf.rs      # LDF 解析
│   │
│   └── view/               # 桌面应用
│       └── src/
│           ├── main.rs     # 入口
│           ├── views/      # 页面（日志/配置/图表/状态栏）
│           ├── ui/         # UI 组件与主题
│           ├── models/     # 数据模型
│           ├── controllers/# 控制器
│           └── handlers/   # 事件处理
│
├── assets/                 # 图标与品牌资源
├── tests/                  # 集成测试
├── .github/workflows/      # CI/CD（构建 + 发布）
├── Cargo.toml              # Workspace 配置
└── LICENSE.txt              # MIT 许可证
```

---

## 作为库使用

BLF 解析器和信号数据库解析器可以独立使用：

```toml
[dependencies]
blf = { git = "https://github.com/ucanme/canview.git" }
parser = { git = "https://github.com/ucanme/canview.git" }
```

```rust
// 解析 BLF 文件
let result = blf::read_blf_from_file("example.blf")?;
for obj in &result.objects {
    // 处理 CAN/LIN/FlexRay 消息 ...
}

// 解析 DBC 数据库
let db = parser::dbc::DbcParser::parse_file("example.dbc")?;
for msg in &db.messages {
    println!("{} (0x{:X})", msg.name, msg.id);
}
```

---

## 开发

```bash
cargo test --workspace        # 运行测试
cargo fmt --all               # 格式化
cargo clippy --workspace      # 静态检查
RUST_LOG=debug cargo run --bin view  # 调试运行
```

跨平台构建详见 [BUILD.md](BUILD.md)。

---

## 路线图

- [x] BLF 解析核心功能
- [x] DBC/LDF 数据库解析
- [x] GPUI 桌面 UI
- [x] 消息过滤与信号解码
- [x] 配置管理与跨平台构建
- [x] 信号折线图（缩放、悬停、绝对时间）
- [ ] 实时流模式（开发中）
- [ ] 导出 CSV / JSON
- [ ] 搜索功能
- [ ] 诊断规则 DSL
- [ ] 基于大模型的智能诊断

---

## 贡献

欢迎贡献！请遵循以下流程：

1. Fork 本仓库
2. 创建特性分支：`git checkout -b feature/your-feature`
3. 提交更改并推送
4. 发起 Pull Request

请确保代码通过 `cargo fmt` 和 `cargo clippy` 检查。

---

## 许可证

[MIT License](LICENSE.txt) © 2026 CANVIEW

---

## 致谢

- [Zed Editor](https://zed.dev/) — GPUI 框架
- [Vector Informatik](https://www.vector.com/) — BLF 格式规范
- Rust 社区 — 优秀的工具链和生态

---

<div align="center">

**使用 Rust 用 ❤️ 构建**

[Issues](https://github.com/ucanme/canview/issues) · [Discussions](https://github.com/ucanme/canview/discussions) · admin@ucan.me

</div>
