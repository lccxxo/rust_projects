# OxiJade 终端模拟器设计文档

**日期**: 2026-04-02
**项目**: OxiJade
**平台**: Windows

---

## 概述

OxiJade 是一个用 Rust 编写的现代终端模拟器 + SSH 客户端，灵感来自 Alacritty 和 XShell。支持本地 Shell 和远程 SSH 会话，具有现代化 UI 风格（GitHub 暗色 + Catppuccin 紫蓝配色）。

---

## 功能范围

### 会话管理
- 支持本地 Shell（PowerShell）和 SSH 远程会话
- 会话列表支持分组/文件夹嵌套管理
- SSH 认证：密码、私钥文件、跳板机（ProxyJump）
- 配置持久化至 `%APPDATA%\OxiJade\profiles.json`
- 不保存连接历史，通过会话列表管理所有连接

### 终端功能
- 标签页式多会话管理（顶部标签栏）
- 分屏支持（水平/垂直）：每个标签页内部维护一棵递归二叉树（`SplitPane`），叶节点是终端实例，内部节点是分割方向和比例
- 文本搜索与高亮（正则匹配）
- 右键上下文菜单（复制/粘贴/搜索/分屏）
- 自定义快捷键（配置存储于 `settings.json`）
- VT100/ANSI 转义码完整支持

### 文件传输
- SSH 会话：通过 SFTP 上传/下载文件
- 拖拽文件到 SSH 终端窗口 → 触发 SFTP 上传
- 拖拽文件到本地终端 → 自动粘贴文件路径
- 底部状态栏显示传输进度

---

## 架构：分层架构（方案 B）

```
OxiJade/
├── oxijade-app/        # egui UI 层
│   ├── src/
│   │   ├── main.rs
│   │   ├── app.rs
│   │   ├── panels/
│   │   │   ├── sidebar.rs
│   │   │   ├── terminal.rs
│   │   │   └── tab_bar.rs
│   │   └── theme.rs
│   └── Cargo.toml
│
├── oxijade-core/       # 业务逻辑层
│   ├── src/
│   │   ├── lib.rs
│   │   ├── session/
│   │   │   ├── manager.rs
│   │   │   ├── local.rs
│   │   │   └── ssh.rs
│   │   ├── terminal/
│   │   │   └── emulator.rs
│   │   └── transfer/
│   │       └── sftp.rs
│   └── Cargo.toml
│
├── oxijade-config/     # 配置持久化层
│   ├── src/
│   │   ├── profile.rs
│   │   └── settings.rs
│   └── Cargo.toml
│
└── Cargo.toml          # workspace
```

---

## 技术栈

| 组件 | 库 |
|------|----|
| UI 框架 | `egui` + `eframe` |
| 终端 VT100 解析 | `alacritty-terminal` |
| 本地 PTY | `portable-pty` |
| SSH 协议 | `russh` |
| 文件传输 | `russh`（SFTP 子系统） |
| 异步运行时 | `tokio` |
| 配置序列化 | `serde` + `serde_json` |

---

## 数据流

```
用户输入 (键盘/鼠标)
    │
    ▼
oxijade-app (egui)
    │  tokio::sync::mpsc channel
    ▼
oxijade-core (SessionManager)
    ├── LocalSession → portable-pty → OS Shell (PowerShell)
    └── SshSession   → russh → 远程服务器
    │
    ▼ TerminalEvent (终端输出)
oxijade-app → alacritty-terminal 解析 → egui 渲染
```

UI 层与 Core 层通过异步 channel 通信，多会话并发不互相阻塞。

---

## UI 设计

### 整体布局

```
┌─────────────────────────────────────────────────────┐
│  ⬡ OxiJade  [● server-1] [○ local] [○ logs]  [+]   │  ← 标签栏
├──────────┬──────────────────────────────────────────┤
│          │  ┌──────────────┬──────────────────────┐ │
│ 📁 生产  │  │ SSH—server-1 │ LOCAL—PowerShell     │ │
│  🖥 web  │  │              │                      │ │
│  🖥 db   │  │  (终端内容)  │  (终端内容)          │ │
│ 📁 测试  │  │              │                      │ │
│  🖥 test │  └──────────────┴──────────────────────┘ │
│          │                                          │
│ [+ 新建] │  [传输进度: nginx.conf 42%]              │  ← 状态栏
└──────────┴──────────────────────────────────────────┘
```

### 主题配色（冷静紫蓝风格）

| 元素 | 颜色 |
|------|------|
| 主背景 | `#0d1117` |
| 面板背景 | `#161b22` |
| 活跃标签指示器 | `#cba6f7`（紫色） |
| SSH 会话边框 | `rgba(203, 166, 247, 0.15)` |
| 本地会话边框 | `rgba(137, 180, 250, 0.15)` |
| SSH 提示符 `❯` | `#cba6f7` |
| 本地提示符 `❯` | `#89b4fa` |
| 成功输出 | `#a6e3a1` |
| 警告输出 | `#f9e2af` |
| 错误输出 | `#f38ba8` |
| 默认文字 | `#cdd6f4` |
| 次要文字 | `#6c7086` |

### 字体
- 默认：JetBrains Mono 或 Cascadia Code
- 可在 `settings.json` 中配置字体名称和大小
- 圆角：面板 6px，标签 4px

---

## 配置文件格式

### profiles.json
```json
{
  "groups": [
    {
      "name": "生产环境",
      "sessions": [
        {
          "id": "uuid",
          "name": "web-server-1",
          "type": "ssh",
          "host": "192.168.1.10",
          "port": 22,
          "username": "user",
          "auth": { "type": "password" }
          // 注：密码在连接时弹窗输入，不以明文存储在配置文件中
        },
        {
          "id": "uuid",
          "name": "db-server",
          "type": "ssh",
          "host": "192.168.1.20",
          "port": 22,
          "username": "user",
          "auth": { "type": "key", "path": "C:\\Users\\lccxxo\\.ssh\\id_rsa" },
          "proxy_jump": "bastion.example.com"
        }
      ]
    }
  ],
  "local_sessions": [
    { "id": "uuid", "name": "PowerShell", "type": "local", "shell": "powershell.exe" }
  ]
}
```

### settings.json
```json
{
  "theme": "catppuccin-purple-blue",
  "font_family": "JetBrains Mono",
  "font_size": 14,
  "keybindings": {
    "new_tab": "Ctrl+T",
    "close_tab": "Ctrl+W",
    "split_horizontal": "Ctrl+Shift+H",
    "split_vertical": "Ctrl+Shift+V",
    "search": "Ctrl+F"
  }
}
```

---

## 错误处理

- SSH 连接失败：在会话标签上显示红色指示点，点击查看错误详情
- PTY 进程退出：显示"会话已结束"提示，提供重连按钮
- 文件传输失败：状态栏显示错误信息，不中断其他会话

---

## 暂不实现（超出当前范围）

- 连接历史记录
- 插件系统
- 宏录制/回放
- 多显示器/窗口分离
- macOS / Linux 支持
