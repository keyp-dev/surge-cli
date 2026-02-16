# Surge TUI

[English](README.md) | [简体中文](README.zh-CN.md)

macOS Surge 代理工具的终端界面（TUI）。

## 特性

- ✅ **终端友好** - 纯文本界面，支持 SSH 远程使用
- ✅ **多语言支持** - 编译时语言选择（中文/英文），零运行时开销
- ✅ **Clean Architecture** - 清晰的分层架构
- ✅ **降级策略** - HTTP API → CLI → System 三层降级
- ✅ **Alert 机制** - 用户控制的错误处理（不自动修改配置）
- ✅ **实时监控** - 策略、请求、连接、DNS 状态
- ✅ **搜索功能** - 策略组/请求/连接独立搜索
- ✅ **分组模式** - 请求/连接按应用名分组展示
- ✅ **连接管理** - 终止单个连接，支持确认对话框
- ✅ **DNS 管理** - DNS 缓存查看和一键清空
- ✅ **帮助系统** - 内置帮助弹窗，快捷键说明

## 架构

```
surge-tui/
├── src/
│   ├── domain/          # 核心业务逻辑（零依赖）
│   │   ├── models.rs    # 数据模型
│   │   ├── entities.rs  # 业务实体
│   │   └── errors.rs    # 错误定义
│   ├── infrastructure/  # 基础设施实现
│   │   ├── http_client.rs   # HTTP API 客户端
│   │   ├── cli_client.rs    # surge-cli 客户端
│   │   └── system_client.rs # 系统命令客户端
│   ├── application/     # 业务协调层
│   │   └── surge_client.rs  # 统一接口
│   ├── ui/             # 用户界面
│   │   ├── app.rs       # 主应用状态
│   │   └── components/  # UI 组件
│   └── config/         # 配置管理
└── docs/              # 设计文档
```

## 安装

### 通过 Homebrew（推荐）

```bash
# 英文版本（默认）
brew tap keyp-dev/tap
brew install surge-tui

# 中文版本
brew install surge-tui-zh
```

### 通过 Nix

```bash
# 英文版本
nix profile install github:keyp-dev/surge-cli

# 中文版本
nix profile install github:keyp-dev/surge-cli#surge-tui-zh
```

### 从源码构建

```bash
# 英文版本（默认）
cargo build --release
cargo install --path .

# 中文版本
cargo build --release --no-default-features --features lang-zh-cn
cargo install --path . --no-default-features --features lang-zh-cn
```

**支持的语言：**
- 🇺🇸 美国英语（`lang-en-us`）- 默认
- 🇨🇳 简体中文（`lang-zh-cn`）

## 快速开始

### 1. 前置条件

确保 macOS 上已安装并运行 Surge，并在配置文件中启用 HTTP API：

```ini
[General]
http-api = your-secret-key@127.0.0.1:6171
http-api-tls = false
```

### 2. 配置

创建配置文件 `surge-tui.toml`：

```toml
[surge]
http_api_host = "127.0.0.1"
http_api_port = 6171
http_api_key = "your-secret-key"  # 与 Surge 配置中的 API Key 一致

[ui]
refresh_interval = 1
max_requests = 100
```

或通过环境变量配置：

```bash
export SURGE_HTTP_API_KEY="your-secret-key"
```

### 3. 运行

```bash
# 本地运行
surge-tui

# 通过 SSH 远程运行
ssh user@mac-host surge-tui
```

## 使用说明

### 核心功能

- ✅ **非阻塞延迟测试** - 按 `T` 键测试所有策略延迟，UI 保持响应
- ✅ **嵌套策略组支持** - 递归显示策略组链中的最终策略延迟
- ✅ **智能通知系统** - 实时状态栏通知 + 历史记录查看（`N` 键）
- ✅ **搜索功能** - 按 `/` 键搜索策略组/请求/连接，实时过滤
- ✅ **分组模式** - 按 `G` 键切换请求/连接按应用名分组
- ✅ **帮助系统** - 按 `H` 键打开帮助弹窗，显示所有快捷键
- ✅ **连接管理** - 按 `K` 键终止选中的连接，带确认对话框
- ✅ **DNS 管理** - 第5个视图查看 DNS 缓存，按 `F` 键清空所有缓存
- ✅ **功能切换** - 快捷键切换出站模式（`M`）、MITM（`I`）、流量捕获（`C`）
- ✅ **增强请求详情** - Notes 语法高亮、HTTP Body 标记
- ✅ **开发者工具** - 按 <code>`</code> 键打开开发工具查看调试日志
- ✅ **延迟颜色编码** - 青色(<100ms) / 黄色(100-300ms) / 红色(>300ms)
- ✅ **测试结果缓存** - 刷新或切换视图后保留延迟数据

### 快捷键

| 按键 | 功能 | 说明 |
|------|------|------|
| `q` | 退出 | 退出程序 |
| `r` | 刷新 | 手动刷新快照 / 重新加载配置（Alert 提示时）|
| `1-5` | 切换视图 | 概览/策略/请求/连接/DNS |
| `↑/↓` | 导航 | 在列表中上下移动 |
| `Enter` | 进入/确认 | 进入策略组或切换策略 |
| `Esc` | 返回/关闭 | 退出策略组或关闭弹窗 |
| `h` / `H` | 帮助 | 打开帮助弹窗显示所有快捷键 |
| `/` | 搜索 | 搜索策略组/请求/连接 |
| `g` / `G` | 分组模式 | 请求/连接按应用名分组 |
| `t` / `T` | 测试延迟 | 非阻塞测试所有策略延迟 |
| `m` / `M` | 切换模式 | 循环切换直连/代理/规则 |
| `i` / `I` | 切换 MITM | 在概览视图中切换 MITM 状态 |
| `c` / `C` | 切换捕获 | 在概览视图中切换流量捕获状态 |
| `k` / `K` | 终止连接 | 在连接视图中终止选中的连接（带确认）|
| `f` / `F` | 清空缓存 | 在 DNS 视图中清空 DNS 缓存 |
| `n` / `N` | 通知历史 | 查看完整通知历史（50 条）|
| <code>`</code> | 开发工具 | 打开开发者调试工具 |
| `s` / `S` | 启动 Surge | 仅在 Alert 提示时可用 |

快捷键直接显示在底部状态栏（类似 btop）。

### 视图

#### 1. 概览
- Surge 运行状态，HTTP API 可用性
- 当前出站模式（`M` 键快速切换）
- MITM 状态（`I` 键快速切换）
- 流量捕获状态（`C` 键快速切换）
- 系统统计信息

#### 2. 策略
- **左侧**：策略组列表，显示当前选中的策略
- **右侧**：策略详情和延迟（支持嵌套策略组）
- **搜索**：`/` 键搜索策略组
- **测试**：`T` 键测试所有策略延迟

#### 3. 请求
- 最近的请求记录（URL、策略、流量统计）
- **搜索**：`/` 键搜索请求
- **分组**：`G` 键按应用名分组
- **详情**：Notes 高亮、HTTP Body 标记

#### 4. 连接
- 当前活动的网络连接
- **搜索**：`/` 键搜索连接
- **分组**：`G` 键按应用名分组
- **管理**：`K` 键终止选中的连接（带确认）

#### 5. DNS 缓存
- DNS 缓存记录（域名、IP、TTL）
- **搜索**：`/` 键搜索域名
- **清空**：`F` 键清空所有 DNS 缓存

## 降级策略

surge-tui 实现了三层降级机制，确保在各种情况下都能工作：

1. **HTTP API**（优先）- 功能最全，性能最好
2. **surge-cli**（降级）- HTTP API 不可用时自动切换
3. **系统命令**（最后）- 检查进程状态，启动 Surge

### Alert 机制

不会自动修改配置，而是通过 Alert 提示用户：

- **Surge 未运行** → 显示"按 S 启动 Surge"
- **HTTP API 不可用** → 显示"按 R 重新加载配置"（用户需先手动添加 http-api 配置）

## 开发

### 构建

```bash
cargo check  # 检查代码
cargo build  # 调试构建
cargo build --release  # 发布构建
```

### 架构原则

- **单一职责** - 每个模块只负责一件事
- **依赖倒置** - 依赖流向：UI → Application → Infrastructure → Domain
- **开闭原则** - 易于扩展，无需修改现有代码
- **零 Domain 依赖** - Domain 层零外部依赖

## 故障排除

### Surge 未运行

```bash
# 检查 Surge 进程
pgrep -x Surge

# 启动 Surge
open -a Surge
```

### HTTP API 不可用

检查 Surge 配置文件（通常在 `~/.config/surge/` 或 Surge.app 配置目录）：

```ini
[General]
http-api = your-secret-key@127.0.0.1:6171
```

添加后，在 surge-tui 中按 `R` 键重新加载配置。

### surge-cli 未找到

surge-cli 位于 Surge.app 内部：

```bash
/Applications/Surge.app/Contents/Applications/surge-cli
```

可以在配置文件中手动指定路径。

## 技术栈

- **Rust** - 系统编程语言
- **tokio** - 异步运行时
- **ratatui** - 终端 UI 框架
- **reqwest** - HTTP 客户端
- **serde** - 序列化/反序列化

## 许可证

MIT

## 文档

`docs/` 目录中的详细文档：

- **[FEATURES.md](docs/FEATURES.md)** - 已实现功能的详细说明（推荐阅读）
- [requirements.md](docs/requirements.md) - 详细需求文档
- [research.md](docs/research.md) - Surge CLI/HTTP API 技术研究

---

通过 [Claude Code](https://claude.ai/code)
基于 [Happy](https://happy.engineering) 生成

Co-Authored-By: Claude <noreply@anthropic.com>
