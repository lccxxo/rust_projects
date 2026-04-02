# OxiJade Plan 1 — Local Terminal MVP

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在 egui 窗口中运行一个本地 PowerShell 会话，具备 OxiJade 主题风格、侧边栏会话列表和标签页。

**Architecture:** 三层 Cargo workspace：`oxijade-config`（配置持久化）→ `oxijade-core`（PTY + VT100 解析 + 会话管理）→ `oxijade-app`（egui UI）。Core 层通过 `tokio::sync::mpsc` channel 异步向 App 层推送终端输出事件。

**Tech Stack:** `egui 0.29` + `eframe 0.29`（UI）、`vte 0.13`（VT100 解析）、`portable-pty 0.8`（Windows PTY）、`tokio 1`（异步运行时）、`serde 1` + `serde_json 1`（配置序列化）

---

## 文件结构

```
OxiJade/
├── Cargo.toml                          # workspace root
├── oxijade-config/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                      # re-exports
│       ├── profile.rs                  # SessionProfile, SessionGroup
│       └── settings.rs                 # AppSettings, Keybindings
├── oxijade-core/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                      # re-exports
│       ├── terminal/
│       │   ├── mod.rs
│       │   ├── cell.rs                 # Cell, CellColor, CellFlags
│       │   └── grid.rs                 # TerminalGrid + vte::Perform impl
│       └── session/
│           ├── mod.rs                  # SessionEvent, SessionId
│           ├── local.rs                # LocalSession (portable-pty)
│           └── manager.rs              # SessionManager
└── oxijade-app/
    ├── Cargo.toml
    └── src/
        ├── main.rs                     # 入口
        ├── app.rs                      # OxiJadeApp + eframe::App impl
        ├── theme.rs                    # 颜色、字体、间距常量
        └── panels/
            ├── mod.rs
            ├── sidebar.rs              # 左侧会话树面板
            ├── tab_bar.rs              # 顶部标签栏
            └── terminal.rs             # 终端渲染 widget
```

---

## Task 1: Cargo Workspace 搭建

**Files:**
- Create: `OxiJade/Cargo.toml`
- Create: `oxijade-config/Cargo.toml`
- Create: `oxijade-core/Cargo.toml`
- Create: `oxijade-app/Cargo.toml`

- [ ] **Step 1: 创建 workspace Cargo.toml**

```toml
# OxiJade/Cargo.toml
[workspace]
members = [
    "oxijade-config",
    "oxijade-core",
    "oxijade-app",
]
resolver = "2"
```

- [ ] **Step 2: 创建 oxijade-config/Cargo.toml**

```toml
[package]
name = "oxijade-config"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
dirs = "5"
uuid = { version = "1", features = ["v4", "serde"] }
```

- [ ] **Step 3: 创建 oxijade-core/Cargo.toml**

```toml
[package]
name = "oxijade-core"
version = "0.1.0"
edition = "2021"

[dependencies]
oxijade-config = { path = "../oxijade-config" }
vte = "0.13"
portable-pty = "0.8"
tokio = { version = "1", features = ["full"] }
```

- [ ] **Step 4: 创建 oxijade-app/Cargo.toml**

```toml
[package]
name = "oxijade-app"
version = "0.1.0"
edition = "2021"

[dependencies]
oxijade-config = { path = "../oxijade-config" }
oxijade-core = { path = "../oxijade-core" }
eframe = "0.29"
egui = "0.29"
tokio = { version = "1", features = ["full"] }

[profile.release]
opt-level = 3
```

- [ ] **Step 5: 创建所有 src/lib.rs 和 src/main.rs 占位文件**

```rust
// oxijade-config/src/lib.rs
pub mod profile;
pub mod settings;
```

```rust
// oxijade-core/src/lib.rs
pub mod session;
pub mod terminal;
```

```rust
// oxijade-app/src/main.rs
fn main() {
    println!("OxiJade starting...");
}
```

- [ ] **Step 6: 验证 workspace 能编译**

```bash
cargo build
```
Expected: 编译成功，无错误（有 warnings 正常）

- [ ] **Step 7: Commit**

```bash
git add OxiJade/Cargo.toml oxijade-config/ oxijade-core/ oxijade-app/
git commit -m "feat: init cargo workspace with three crates"
```

---

## Task 2: oxijade-config — Profile 类型

**Files:**
- Create: `oxijade-config/src/profile.rs`
- Create: `oxijade-config/src/lib.rs` (update)
- Test: `oxijade-config/src/profile.rs` (inline tests)

- [ ] **Step 1: 写失败的测试**

```rust
// oxijade-config/src/profile.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_json_roundtrip() {
        let group = SessionGroup {
            name: "Production".to_string(),
            sessions: vec![
                SessionProfile::Ssh(SshProfile {
                    id: "test-id".to_string(),
                    name: "web-server".to_string(),
                    host: "192.168.1.10".to_string(),
                    port: 22,
                    username: "user".to_string(),
                    auth: SshAuth::Password,
                    proxy_jump: None,
                }),
            ],
        };
        let json = serde_json::to_string(&group).unwrap();
        let decoded: SessionGroup = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.name, "Production");
        assert_eq!(decoded.sessions.len(), 1);
    }

    #[test]
    fn test_local_profile_json_roundtrip() {
        let profile = SessionProfile::Local(LocalProfile {
            id: "local-1".to_string(),
            name: "PowerShell".to_string(),
            shell: "powershell.exe".to_string(),
        });
        let json = serde_json::to_string(&profile).unwrap();
        let decoded: SessionProfile = serde_json::from_str(&json).unwrap();
        match decoded {
            SessionProfile::Local(p) => assert_eq!(p.shell, "powershell.exe"),
            _ => panic!("wrong variant"),
        }
    }
}
```

- [ ] **Step 2: 运行测试，确认失败**

```bash
cargo test -p oxijade-config
```
Expected: 编译错误（类型未定义）

- [ ] **Step 3: 实现 Profile 类型**

```rust
// oxijade-config/src/profile.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalProfile {
    pub id: String,
    pub name: String,
    pub shell: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshProfile {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth: SshAuth,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_jump: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SshAuth {
    Password,
    Key { path: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SessionProfile {
    Local(LocalProfile),
    Ssh(SshProfile),
}

impl SessionProfile {
    pub fn id(&self) -> &str {
        match self {
            SessionProfile::Local(p) => &p.id,
            SessionProfile::Ssh(p) => &p.id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            SessionProfile::Local(p) => &p.name,
            SessionProfile::Ssh(p) => &p.name,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionGroup {
    pub name: String,
    pub sessions: Vec<SessionProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfileStore {
    pub groups: Vec<SessionGroup>,
}
```

- [ ] **Step 4: 运行测试，确认通过**

```bash
cargo test -p oxijade-config
```
Expected: 2 tests passed

- [ ] **Step 5: Commit**

```bash
git add oxijade-config/src/profile.rs oxijade-config/src/lib.rs
git commit -m "feat(config): add SessionProfile and SessionGroup types"
```

---

## Task 3: oxijade-config — Settings 类型 + JSON 读写

**Files:**
- Create: `oxijade-config/src/settings.rs`
- Modify: `oxijade-config/src/lib.rs`

- [ ] **Step 1: 写失败的测试**

```rust
// oxijade-config/src/settings.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_defaults() {
        let s = AppSettings::default();
        assert_eq!(s.font_size, 14.0);
        assert_eq!(s.font_family, "JetBrains Mono");
    }

    #[test]
    fn test_settings_json_roundtrip() {
        let s = AppSettings::default();
        let json = serde_json::to_string(&s).unwrap();
        let decoded: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.font_size, s.font_size);
        assert_eq!(decoded.keybindings.new_tab, s.keybindings.new_tab);
    }
}
```

- [ ] **Step 2: 运行测试，确认失败**

```bash
cargo test -p oxijade-config settings
```
Expected: 编译错误

- [ ] **Step 3: 实现 Settings 类型**

```rust
// oxijade-config/src/settings.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybindings {
    pub new_tab: String,
    pub close_tab: String,
    pub split_horizontal: String,
    pub split_vertical: String,
    pub search: String,
}

impl Default for Keybindings {
    fn default() -> Self {
        Self {
            new_tab: "Ctrl+T".to_string(),
            close_tab: "Ctrl+W".to_string(),
            split_horizontal: "Ctrl+Shift+H".to_string(),
            split_vertical: "Ctrl+Shift+V".to_string(),
            search: "Ctrl+F".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub font_family: String,
    pub font_size: f32,
    pub keybindings: Keybindings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            font_family: "JetBrains Mono".to_string(),
            font_size: 14.0,
            keybindings: Keybindings::default(),
        }
    }
}

/// 返回配置目录：%APPDATA%\OxiJade\
pub fn config_dir() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("OxiJade")
}

pub fn load_settings() -> AppSettings {
    let path = config_dir().join("settings.json");
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_settings(settings: &AppSettings) -> std::io::Result<()> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir)?;
    let json = serde_json::to_string_pretty(settings).unwrap();
    std::fs::write(dir.join("settings.json"), json)
}

pub fn load_profiles() -> crate::profile::ProfileStore {
    let path = config_dir().join("profiles.json");
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_profiles(store: &crate::profile::ProfileStore) -> std::io::Result<()> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir)?;
    let json = serde_json::to_string_pretty(store).unwrap();
    std::fs::write(dir.join("profiles.json"), json)
}
```

- [ ] **Step 4: 更新 lib.rs**

```rust
// oxijade-config/src/lib.rs
pub mod profile;
pub mod settings;

pub use profile::{LocalProfile, ProfileStore, SessionGroup, SessionProfile, SshAuth, SshProfile};
pub use settings::{load_profiles, load_settings, save_profiles, save_settings, AppSettings, Keybindings};
```

- [ ] **Step 5: 运行测试，确认通过**

```bash
cargo test -p oxijade-config
```
Expected: 4 tests passed

- [ ] **Step 6: Commit**

```bash
git add oxijade-config/src/settings.rs oxijade-config/src/lib.rs
git commit -m "feat(config): add AppSettings and JSON load/save helpers"
```

---

## Task 4: oxijade-core — Terminal Cell 和 Grid

**Files:**
- Create: `oxijade-core/src/terminal/mod.rs`
- Create: `oxijade-core/src/terminal/cell.rs`
- Create: `oxijade-core/src/terminal/grid.rs`
- Modify: `oxijade-core/src/lib.rs`

- [ ] **Step 1: 写失败的测试**

```rust
// oxijade-core/src/terminal/grid.rs (末尾)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_char_moves_cursor() {
        let mut grid = TerminalGrid::new(80, 24);
        grid.process_byte(b'A');
        assert_eq!(grid.cursor_x, 1);
        assert_eq!(grid.cursor_y, 0);
        assert_eq!(grid.cell_at(0, 0).c, 'A');
    }

    #[test]
    fn test_newline_moves_cursor_down() {
        let mut grid = TerminalGrid::new(80, 24);
        grid.process_byte(b'\r');
        grid.process_byte(b'\n');
        assert_eq!(grid.cursor_x, 0);
        assert_eq!(grid.cursor_y, 1);
    }

    #[test]
    fn test_clear_screen() {
        let mut grid = TerminalGrid::new(80, 24);
        grid.process_byte(b'A');
        // ESC[2J — clear screen
        for b in b"\x1b[2J" {
            grid.process_byte(*b);
        }
        assert_eq!(grid.cell_at(0, 0).c, ' ');
        assert_eq!(grid.cursor_x, 0);
    }

    #[test]
    fn test_sgr_color() {
        let mut grid = TerminalGrid::new(80, 24);
        // ESC[32m — set fg to green (ANSI color 2)
        for b in b"\x1b[32mA" {
            grid.process_byte(*b);
        }
        match grid.cell_at(0, 0).fg {
            CellColor::Indexed(n) => assert_eq!(n, 2),
            _ => panic!("expected indexed color"),
        }
    }
}
```

- [ ] **Step 2: 运行测试，确认失败**

```bash
cargo test -p oxijade-core terminal
```
Expected: 编译错误

- [ ] **Step 3: 实现 Cell 类型**

```rust
// oxijade-core/src/terminal/cell.rs
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CellColor {
    Default,
    Indexed(u8),
    Rgb(u8, u8, u8),
}

impl Default for CellColor {
    fn default() -> Self {
        CellColor::Default
    }
}

bitflags::bitflags! {
    #[derive(Default, Clone, Copy, Debug, PartialEq)]
    pub struct CellFlags: u8 {
        const BOLD      = 0b0000_0001;
        const ITALIC    = 0b0000_0010;
        const UNDERLINE = 0b0000_0100;
        const BLINK     = 0b0000_1000;
        const REVERSE   = 0b0001_0000;
    }
}

#[derive(Debug, Clone, Default)]
pub struct Cell {
    pub c: char,
    pub fg: CellColor,
    pub bg: CellColor,
    pub flags: CellFlags,
}
```

> 需要在 oxijade-core/Cargo.toml 中添加：`bitflags = "2"`

- [ ] **Step 4: 实现 TerminalGrid + vte::Perform**

```rust
// oxijade-core/src/terminal/grid.rs
use crate::terminal::cell::{Cell, CellColor, CellFlags};
use vte::{Params, Parser, Perform};

pub struct TerminalGrid {
    pub cols: usize,
    pub rows: usize,
    cells: Vec<Cell>,
    pub cursor_x: usize,
    pub cursor_y: usize,
    current_fg: CellColor,
    current_bg: CellColor,
    current_flags: CellFlags,
    parser: Parser,
}

impl TerminalGrid {
    pub fn new(cols: usize, rows: usize) -> Self {
        let mut cells = Vec::with_capacity(cols * rows);
        for _ in 0..cols * rows {
            cells.push(Cell { c: ' ', ..Default::default() });
        }
        Self {
            cols,
            rows,
            cells,
            cursor_x: 0,
            cursor_y: 0,
            current_fg: CellColor::Default,
            current_bg: CellColor::Default,
            current_flags: CellFlags::empty(),
            parser: Parser::new(),
        }
    }

    pub fn resize(&mut self, cols: usize, rows: usize) {
        self.cols = cols;
        self.rows = rows;
        let mut cells = Vec::with_capacity(cols * rows);
        for _ in 0..cols * rows {
            cells.push(Cell { c: ' ', ..Default::default() });
        }
        self.cells = cells;
        self.cursor_x = self.cursor_x.min(cols.saturating_sub(1));
        self.cursor_y = self.cursor_y.min(rows.saturating_sub(1));
    }

    pub fn cell_at(&self, x: usize, y: usize) -> &Cell {
        &self.cells[y * self.cols + x]
    }

    /// 处理单个字节（委托给 process_bytes 以保持 parser 状态跨调用）
    pub fn process_byte(&mut self, byte: u8) {
        self.process_bytes(&[byte]);
    }

    pub fn process_bytes(&mut self, bytes: &[u8]) {
        // 用 mem::replace 取出 parser，避免同时可变借用 self
        let mut parser = std::mem::replace(&mut self.parser, Parser::new());
        let mut performer = GridPerformer { grid: self };
        for &byte in bytes {
            parser.advance(&mut performer, byte);
        }
        self.parser = parser;
    }

    fn set_cell(&mut self, x: usize, y: usize, c: char) {
        if x < self.cols && y < self.rows {
            let idx = y * self.cols + x;
            self.cells[idx] = Cell {
                c,
                fg: self.current_fg,
                bg: self.current_bg,
                flags: self.current_flags,
            };
        }
    }

    fn scroll_up(&mut self) {
        self.cells.drain(0..self.cols);
        for _ in 0..self.cols {
            self.cells.push(Cell { c: ' ', ..Default::default() });
        }
    }
}

struct GridPerformer<'a> {
    grid: &'a mut TerminalGrid,
}

impl<'a> Perform for GridPerformer<'a> {
    fn print(&mut self, c: char) {
        self.grid.set_cell(self.grid.cursor_x, self.grid.cursor_y, c);
        self.grid.cursor_x += 1;
        if self.grid.cursor_x >= self.grid.cols {
            self.grid.cursor_x = 0;
            self.grid.cursor_y += 1;
            if self.grid.cursor_y >= self.grid.rows {
                self.grid.scroll_up();
                self.grid.cursor_y = self.grid.rows - 1;
            }
        }
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\r' => self.grid.cursor_x = 0,
            b'\n' => {
                self.grid.cursor_y += 1;
                if self.grid.cursor_y >= self.grid.rows {
                    self.grid.scroll_up();
                    self.grid.cursor_y = self.grid.rows - 1;
                }
            }
            b'\x08' => {
                // backspace
                if self.grid.cursor_x > 0 {
                    self.grid.cursor_x -= 1;
                    self.grid.set_cell(self.grid.cursor_x, self.grid.cursor_y, ' ');
                }
            }
            _ => {}
        }
    }

    fn csi_dispatch(&mut self, params: &Params, _intermediates: &[u8], _ignore: bool, action: char) {
        let ps: Vec<u16> = params.iter().map(|s| s[0]).collect();
        match action {
            // Cursor movement
            'A' => { let n = ps.first().copied().unwrap_or(1).max(1) as usize; self.grid.cursor_y = self.grid.cursor_y.saturating_sub(n); }
            'B' => { let n = ps.first().copied().unwrap_or(1).max(1) as usize; self.grid.cursor_y = (self.grid.cursor_y + n).min(self.grid.rows - 1); }
            'C' => { let n = ps.first().copied().unwrap_or(1).max(1) as usize; self.grid.cursor_x = (self.grid.cursor_x + n).min(self.grid.cols - 1); }
            'D' => { let n = ps.first().copied().unwrap_or(1).max(1) as usize; self.grid.cursor_x = self.grid.cursor_x.saturating_sub(n); }
            // Cursor position: ESC[row;colH
            'H' | 'f' => {
                let row = ps.first().copied().unwrap_or(1).max(1) as usize - 1;
                let col = ps.get(1).copied().unwrap_or(1).max(1) as usize - 1;
                self.grid.cursor_y = row.min(self.grid.rows - 1);
                self.grid.cursor_x = col.min(self.grid.cols - 1);
            }
            // Erase in display: ESC[Jor ESC[2J
            'J' => {
                let mode = ps.first().copied().unwrap_or(0);
                match mode {
                    0 => { // cursor to end
                        let start = self.grid.cursor_y * self.grid.cols + self.grid.cursor_x;
                        for i in start..self.grid.cells.len() {
                            self.grid.cells[i] = Cell { c: ' ', ..Default::default() };
                        }
                    }
                    2 | 3 => { // entire screen
                        for cell in &mut self.grid.cells {
                            *cell = Cell { c: ' ', ..Default::default() };
                        }
                        self.grid.cursor_x = 0;
                        self.grid.cursor_y = 0;
                    }
                    _ => {}
                }
            }
            // Erase in line: ESC[K
            'K' => {
                let mode = ps.first().copied().unwrap_or(0);
                match mode {
                    0 => { // cursor to end of line
                        for x in self.grid.cursor_x..self.grid.cols {
                            self.grid.set_cell(x, self.grid.cursor_y, ' ');
                        }
                    }
                    1 => { // start of line to cursor
                        for x in 0..=self.grid.cursor_x {
                            self.grid.set_cell(x, self.grid.cursor_y, ' ');
                        }
                    }
                    2 => { // entire line
                        for x in 0..self.grid.cols {
                            self.grid.set_cell(x, self.grid.cursor_y, ' ');
                        }
                    }
                    _ => {}
                }
            }
            // SGR: ESC[...m — colors and attributes
            'm' => {
                if ps.is_empty() {
                    self.grid.current_fg = CellColor::Default;
                    self.grid.current_bg = CellColor::Default;
                    self.grid.current_flags = CellFlags::empty();
                    return;
                }
                let mut i = 0;
                while i < ps.len() {
                    match ps[i] {
                        0 => {
                            self.grid.current_fg = CellColor::Default;
                            self.grid.current_bg = CellColor::Default;
                            self.grid.current_flags = CellFlags::empty();
                        }
                        1 => self.grid.current_flags |= CellFlags::BOLD,
                        3 => self.grid.current_flags |= CellFlags::ITALIC,
                        4 => self.grid.current_flags |= CellFlags::UNDERLINE,
                        22 => self.grid.current_flags.remove(CellFlags::BOLD),
                        30..=37 => self.grid.current_fg = CellColor::Indexed(ps[i] as u8 - 30),
                        39 => self.grid.current_fg = CellColor::Default,
                        40..=47 => self.grid.current_bg = CellColor::Indexed(ps[i] as u8 - 40),
                        49 => self.grid.current_bg = CellColor::Default,
                        90..=97 => self.grid.current_fg = CellColor::Indexed(ps[i] as u8 - 90 + 8),
                        100..=107 => self.grid.current_bg = CellColor::Indexed(ps[i] as u8 - 100 + 8),
                        38 if ps.get(i+1) == Some(&2) && ps.len() > i+4 => {
                            self.grid.current_fg = CellColor::Rgb(ps[i+2] as u8, ps[i+3] as u8, ps[i+4] as u8);
                            i += 4;
                        }
                        48 if ps.get(i+1) == Some(&2) && ps.len() > i+4 => {
                            self.grid.current_bg = CellColor::Rgb(ps[i+2] as u8, ps[i+3] as u8, ps[i+4] as u8);
                            i += 4;
                        }
                        _ => {}
                    }
                    i += 1;
                }
            }
            _ => {}
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {}
    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _action: char) {}
    fn put(&mut self, _byte: u8) {}
    fn unhook(&mut self) {}
    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}
}
```

> **注意**：`process_bytes` 中需要避免借用冲突。将 `parser` 从 `TerminalGrid` 中移出，改为在调用处 own 一个 `Parser`。修改如下：

```rust
// grid.rs 中删除 parser 字段，改为：
pub fn process_bytes(&mut self, bytes: &[u8]) {
    // 临时取出 parser 避免借用冲突
    let mut parser = std::mem::replace(&mut self.parser, Parser::new());
    let mut performer = GridPerformer { grid: self };
    for &byte in bytes {
        parser.advance(&mut performer, byte);
    }
    self.parser = parser;
}
```

- [ ] **Step 5: 创建 terminal/mod.rs**

```rust
// oxijade-core/src/terminal/mod.rs
pub mod cell;
pub mod grid;

pub use cell::{Cell, CellColor, CellFlags};
pub use grid::TerminalGrid;
```

- [ ] **Step 6: 更新 oxijade-core/src/lib.rs**

```rust
// oxijade-core/src/lib.rs
pub mod session;
pub mod terminal;
```

创建空的 session/mod.rs：
```rust
// oxijade-core/src/session/mod.rs
```

- [ ] **Step 7: 运行测试，确认通过**

```bash
cargo test -p oxijade-core terminal
```
Expected: 4 tests passed

- [ ] **Step 8: Commit**

```bash
git add oxijade-core/
git commit -m "feat(core): add TerminalGrid with VT100 parsing via vte"
```

---

## Task 5: oxijade-core — LocalSession (PTY)

**Files:**
- Create: `oxijade-core/src/session/mod.rs`
- Create: `oxijade-core/src/session/local.rs`

- [ ] **Step 1: 定义 SessionEvent（test 也依赖它）**

```rust
// oxijade-core/src/session/mod.rs
pub mod local;
pub mod manager;

/// App 层接收到的终端事件
#[derive(Debug)]
pub enum SessionEvent {
    /// 收到新的终端输出字节
    Output(Vec<u8>),
    /// 会话已退出
    Exited,
}

/// 唯一标识一个会话
pub type SessionId = String;
```

- [ ] **Step 2: 写失败的测试**

```rust
// oxijade-core/src/session/local.rs (末尾)
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn test_local_session_receives_output() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(32);
        let mut session = LocalSession::new("test-id".to_string(), "cmd.exe".to_string(), tx)
            .expect("failed to create session");

        // 发送 echo 命令
        session.write(b"echo hello_oxijade\r\n").await.unwrap();

        // 等待输出中包含 hello_oxijade
        let found = timeout(Duration::from_secs(5), async {
            let mut buf = String::new();
            while let Some(event) = rx.recv().await {
                match event {
                    SessionEvent::Output(bytes) => {
                        buf.push_str(&String::from_utf8_lossy(&bytes));
                        if buf.contains("hello_oxijade") {
                            return true;
                        }
                    }
                    SessionEvent::Exited => return false,
                }
            }
            false
        }).await;

        assert_eq!(found.unwrap(), true);
        session.kill();
    }
}
```

- [ ] **Step 3: 运行测试，确认失败**

```bash
cargo test -p oxijade-core local_session
```
Expected: 编译错误

- [ ] **Step 4: 实现 LocalSession**

```rust
// oxijade-core/src/session/local.rs
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use tokio::sync::mpsc::Sender;
use crate::session::SessionEvent;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

pub struct LocalSession {
    pub id: String,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    child: Box<dyn portable_pty::Child + Send>,
}

impl LocalSession {
    pub fn new(
        id: String,
        shell: String,
        tx: Sender<SessionEvent>,
    ) -> anyhow::Result<Self> {
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut cmd = CommandBuilder::new(&shell);
        cmd.cwd(std::env::current_dir().unwrap_or_default());

        let child = pair.slave.spawn_command(cmd)?;
        let writer = Arc::new(Mutex::new(pair.master.take_writer()?));
        let mut reader = pair.master.try_clone_reader()?;

        // 后台任务：读取 PTY 输出并发送到 channel
        tokio::task::spawn_blocking(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => {
                        let _ = tx.blocking_send(SessionEvent::Exited);
                        break;
                    }
                    Ok(n) => {
                        let _ = tx.blocking_send(SessionEvent::Output(buf[..n].to_vec()));
                    }
                }
            }
        });

        Ok(Self { id, writer, child })
    }

    pub async fn write(&self, data: &[u8]) -> anyhow::Result<()> {
        let writer = self.writer.clone();
        let data = data.to_vec();
        tokio::task::spawn_blocking(move || {
            let mut w = writer.lock().unwrap();
            w.write_all(&data)?;
            w.flush()
        })
        .await??;
        Ok(())
    }

    pub fn resize(&self, cols: u16, rows: u16) {
        // portable-pty master resize
        // Note: requires master to support resize; handled by ConPTY on Windows
    }

    pub fn kill(&mut self) {
        let _ = self.child.kill();
    }
}
```

> 需要在 oxijade-core/Cargo.toml 中添加：`anyhow = "1"`

- [ ] **Step 5: 运行测试，确认通过**

```bash
cargo test -p oxijade-core local_session -- --nocapture
```
Expected: test passed（可能需要等待几秒）

- [ ] **Step 6: Commit**

```bash
git add oxijade-core/src/session/
git commit -m "feat(core): add LocalSession with PTY via portable-pty"
```

---

## Task 6: oxijade-core — SessionManager

**Files:**
- Create: `oxijade-core/src/session/manager.rs`
- Modify: `oxijade-core/src/session/mod.rs`

- [ ] **Step 1: 写失败的测试**

```rust
// oxijade-core/src/session/manager.rs (末尾)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_get_session() {
        let mut manager = SessionManager::new();
        manager.add_terminal("id-1".to_string());
        assert!(manager.get_terminal("id-1").is_some());
        assert!(manager.get_terminal("id-2").is_none());
    }

    #[test]
    fn test_remove_session() {
        let mut manager = SessionManager::new();
        manager.add_terminal("id-1".to_string());
        manager.remove("id-1");
        assert!(manager.get_terminal("id-1").is_none());
    }

    #[test]
    fn test_session_ids() {
        let mut manager = SessionManager::new();
        manager.add_terminal("a".to_string());
        manager.add_terminal("b".to_string());
        let ids = manager.session_ids();
        assert!(ids.contains(&"a".to_string()));
        assert!(ids.contains(&"b".to_string()));
    }
}
```

- [ ] **Step 2: 运行测试，确认失败**

```bash
cargo test -p oxijade-core manager
```
Expected: 编译错误

- [ ] **Step 3: 实现 SessionManager**

```rust
// oxijade-core/src/session/manager.rs
use std::collections::HashMap;
use tokio::sync::mpsc::{channel, Receiver};
use crate::session::{SessionEvent, SessionId};
use crate::terminal::TerminalGrid;

pub struct ManagedSession {
    pub grid: TerminalGrid,
    pub rx: Receiver<SessionEvent>,
}

pub struct SessionManager {
    sessions: HashMap<SessionId, ManagedSession>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self { sessions: HashMap::new() }
    }

    /// 创建一个新的本地终端会话（用于测试，不启动真实 PTY）
    pub fn add_terminal(&mut self, id: SessionId) {
        let (_tx, rx) = channel(32);
        self.sessions.insert(id, ManagedSession {
            grid: TerminalGrid::new(80, 24),
            rx,
        });
    }

    pub fn get_terminal(&self, id: &str) -> Option<&ManagedSession> {
        self.sessions.get(id)
    }

    pub fn get_terminal_mut(&mut self, id: &str) -> Option<&mut ManagedSession> {
        self.sessions.get_mut(id)
    }

    pub fn remove(&mut self, id: &str) {
        self.sessions.remove(id);
    }

    pub fn session_ids(&self) -> Vec<SessionId> {
        self.sessions.keys().cloned().collect()
    }
}
```

- [ ] **Step 4: 更新 session/mod.rs**

```rust
// oxijade-core/src/session/mod.rs
pub mod local;
pub mod manager;

#[derive(Debug)]
pub enum SessionEvent {
    Output(Vec<u8>),
    Exited,
}

pub type SessionId = String;
```

- [ ] **Step 5: 运行测试，确认通过**

```bash
cargo test -p oxijade-core manager
```
Expected: 3 tests passed

- [ ] **Step 6: Commit**

```bash
git add oxijade-core/src/session/manager.rs oxijade-core/src/session/mod.rs
git commit -m "feat(core): add SessionManager"
```

---

## Task 7: oxijade-app — Theme 常量

**Files:**
- Create: `oxijade-app/src/theme.rs`
- Create: `oxijade-app/src/panels/mod.rs`

- [ ] **Step 1: 实现 Theme**

（此模块无复杂逻辑，直接实现无需失败测试）

```rust
// oxijade-app/src/theme.rs
use egui::Color32;

pub struct Theme;

impl Theme {
    // 背景
    pub const BG_PRIMARY: Color32 = Color32::from_rgb(0x0d, 0x11, 0x17);
    pub const BG_PANEL: Color32 = Color32::from_rgb(0x16, 0x1b, 0x22);
    pub const BG_SELECTED: Color32 = Color32::from_rgb(0x1f, 0x29, 0x37);

    // 强调色
    pub const ACCENT_SSH: Color32 = Color32::from_rgb(0xcb, 0xa6, 0xf7);   // 紫
    pub const ACCENT_LOCAL: Color32 = Color32::from_rgb(0x89, 0xb4, 0xfa); // 蓝

    // 文字
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0xcd, 0xd6, 0xf4);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(0x6c, 0x70, 0x86);

    // 终端 ANSI 颜色表（16色）
    pub const ANSI_COLORS: [Color32; 16] = [
        Color32::from_rgb(0x45, 0x47, 0x5a), // 0 black
        Color32::from_rgb(0xf3, 0x8b, 0xa8), // 1 red
        Color32::from_rgb(0xa6, 0xe3, 0xa1), // 2 green
        Color32::from_rgb(0xf9, 0xe2, 0xaf), // 3 yellow
        Color32::from_rgb(0x89, 0xb4, 0xfa), // 4 blue
        Color32::from_rgb(0xcb, 0xa6, 0xf7), // 5 magenta
        Color32::from_rgb(0x89, 0xdc, 0xeb), // 6 cyan
        Color32::from_rgb(0xcd, 0xd6, 0xf4), // 7 white
        Color32::from_rgb(0x58, 0x5b, 0x70), // 8 bright black
        Color32::from_rgb(0xf3, 0x8b, 0xa8), // 9 bright red
        Color32::from_rgb(0xa6, 0xe3, 0xa1), // 10 bright green
        Color32::from_rgb(0xf9, 0xe2, 0xaf), // 11 bright yellow
        Color32::from_rgb(0x89, 0xb4, 0xfa), // 12 bright blue
        Color32::from_rgb(0xcb, 0xa6, 0xf7), // 13 bright magenta
        Color32::from_rgb(0x89, 0xdc, 0xeb), // 14 bright cyan
        Color32::from_rgb(0xcd, 0xd6, 0xf4), // 15 bright white
    ];

    pub fn resolve_color(color: &oxijade_core::terminal::CellColor, default: Color32) -> Color32 {
        use oxijade_core::terminal::CellColor;
        match color {
            CellColor::Default => default,
            CellColor::Indexed(n) => {
                if (*n as usize) < Self::ANSI_COLORS.len() {
                    Self::ANSI_COLORS[*n as usize]
                } else {
                    default
                }
            }
            CellColor::Rgb(r, g, b) => Color32::from_rgb(*r, *g, *b),
        }
    }
}

/// egui 全局 visuals 风格
pub fn apply_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = Theme::BG_PRIMARY;
    visuals.window_fill = Theme::BG_PANEL;
    visuals.override_text_color = Some(Theme::TEXT_PRIMARY);
    visuals.widgets.noninteractive.bg_fill = Theme::BG_PANEL;
    visuals.widgets.inactive.bg_fill = Theme::BG_PANEL;
    visuals.widgets.hovered.bg_fill = Theme::BG_SELECTED;
    visuals.widgets.active.bg_fill = Theme::BG_SELECTED;
    ctx.set_visuals(visuals);
}
```

```rust
// oxijade-app/src/panels/mod.rs
pub mod sidebar;
pub mod tab_bar;
pub mod terminal;
```

- [ ] **Step 2: 验证编译**

```bash
cargo build -p oxijade-app
```
Expected: 编译成功

- [ ] **Step 3: Commit**

```bash
git add oxijade-app/src/theme.rs oxijade-app/src/panels/
git commit -m "feat(app): add OxiJade theme constants and panel module"
```

---

## Task 8: oxijade-app — egui 主窗口骨架

**Files:**
- Create: `oxijade-app/src/app.rs`
- Modify: `oxijade-app/src/main.rs`

- [ ] **Step 1: 实现 OxiJadeApp**

```rust
// oxijade-app/src/app.rs
use egui::Context;
use crate::theme::apply_theme;

pub struct OxiJadeApp {
    // 当前选中的标签 session id
    pub active_tab: Option<String>,
    // 侧边栏宽度
    pub sidebar_width: f32,
}

impl Default for OxiJadeApp {
    fn default() -> Self {
        Self {
            active_tab: None,
            sidebar_width: 200.0,
        }
    }
}

impl eframe::App for OxiJadeApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        apply_theme(ctx);

        // 顶部标签栏
        egui::TopBottomPanel::top("tab_bar").show(ctx, |ui| {
            crate::panels::tab_bar::show(ui, self);
        });

        // 左侧侧边栏
        egui::SidePanel::left("sidebar")
            .default_width(self.sidebar_width)
            .width_range(150.0..=300.0)
            .show(ctx, |ui| {
                crate::panels::sidebar::show(ui, self);
            });

        // 中央终端区域
        egui::CentralPanel::default().show(ctx, |ui| {
            crate::panels::terminal::show(ui, self);
        });
    }
}
```

- [ ] **Step 2: 实现 main.rs**

```rust
// oxijade-app/src/main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod panels;
mod theme;

use eframe::NativeOptions;
use egui::ViewportBuilder;

fn main() -> eframe::Result<()> {
    let options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title("OxiJade")
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "OxiJade",
        options,
        Box::new(|_cc| Ok(Box::new(app::OxiJadeApp::default()))),
    )
}
```

- [ ] **Step 3: 创建空的 panel 占位实现**

```rust
// oxijade-app/src/panels/tab_bar.rs
pub fn show(ui: &mut egui::Ui, _app: &mut crate::app::OxiJadeApp) {
    ui.horizontal(|ui| {
        ui.label("⬡ OxiJade");
    });
}
```

```rust
// oxijade-app/src/panels/sidebar.rs
pub fn show(ui: &mut egui::Ui, _app: &mut crate::app::OxiJadeApp) {
    ui.label("Sessions");
}
```

```rust
// oxijade-app/src/panels/terminal.rs
pub fn show(ui: &mut egui::Ui, _app: &mut crate::app::OxiJadeApp) {
    ui.label("Terminal area");
}
```

- [ ] **Step 4: 运行，确认窗口能打开**

```bash
cargo run -p oxijade-app
```
Expected: 打开一个深色背景窗口，显示 "⬡ OxiJade" 标题栏、左侧 "Sessions"、中间 "Terminal area"

- [ ] **Step 5: Commit**

```bash
git add oxijade-app/src/
git commit -m "feat(app): add egui window skeleton with theme"
```

---

## Task 9: oxijade-app — Sidebar 面板

**Files:**
- Modify: `oxijade-app/src/panels/sidebar.rs`
- Modify: `oxijade-app/src/app.rs`

- [ ] **Step 1: 更新 OxiJadeApp 持有 ProfileStore**

```rust
// oxijade-app/src/app.rs（顶部 use + 结构体更新）
use oxijade_config::{ProfileStore, load_profiles};

pub struct OxiJadeApp {
    pub active_tab: Option<String>,
    pub sidebar_width: f32,
    pub profiles: ProfileStore,
}

impl Default for OxiJadeApp {
    fn default() -> Self {
        Self {
            active_tab: None,
            sidebar_width: 200.0,
            profiles: load_profiles(),
        }
    }
}
```

- [ ] **Step 2: 实现 Sidebar**

```rust
// oxijade-app/src/panels/sidebar.rs
use egui::{Color32, RichText, Ui};
use oxijade_config::SessionProfile;
use crate::app::OxiJadeApp;
use crate::theme::Theme;

pub fn show(ui: &mut Ui, app: &mut OxiJadeApp) {
    ui.visuals_mut().panel_fill = Theme::BG_PANEL;

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(6.0);

        // 无分组的本地会话和所有分组
        let profiles = app.profiles.clone();

        for group in &profiles.groups {
            ui.collapsing(
                RichText::new(format!("📁 {}", group.name))
                    .color(Theme::TEXT_MUTED)
                    .size(12.0),
                |ui| {
                    for session in &group.sessions {
                        session_row(ui, session, app);
                    }
                },
            );
        }

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);

        // 添加新会话按钮（占位）
        if ui.button(
            RichText::new("＋ 新建会话").color(Theme::ACCENT_LOCAL).size(12.0)
        ).clicked() {
            // TODO Plan 2: 打开新建会话对话框
        }
    });
}

fn session_row(ui: &mut Ui, profile: &SessionProfile, app: &mut OxiJadeApp) {
    let id = profile.id();
    let name = profile.name();
    let (icon, accent) = match profile {
        SessionProfile::Local(_) => ("🖥", Theme::ACCENT_LOCAL),
        SessionProfile::Ssh(_) => ("🔗", Theme::ACCENT_SSH),
    };

    let is_active = app.active_tab.as_deref() == Some(id);
    let bg = if is_active { Theme::BG_SELECTED } else { Color32::TRANSPARENT };

    let response = egui::Frame::none()
        .fill(bg)
        .rounding(4.0)
        .inner_margin(egui::Margin::symmetric(8.0, 3.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(icon).size(12.0));
                ui.label(
                    RichText::new(name)
                        .color(if is_active { Theme::TEXT_PRIMARY } else { Theme::TEXT_MUTED })
                        .size(12.0),
                );
                if is_active {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let dot = RichText::new("●").color(accent).size(8.0);
                        ui.label(dot);
                    });
                }
            });
        })
        .response;

    if response.interact(egui::Sense::click()).clicked() {
        app.active_tab = Some(id.to_string());
    }
}
```

- [ ] **Step 3: 在 profiles 中添加一个默认本地会话（首次运行）**

在 `OxiJadeApp::default()` 中，若 profiles 为空则插入默认本地会话：

```rust
// app.rs — Default impl 修改
impl Default for OxiJadeApp {
    fn default() -> Self {
        let mut profiles = load_profiles();
        if profiles.groups.is_empty() {
            use oxijade_config::{LocalProfile, SessionGroup, SessionProfile};
            profiles.groups.push(SessionGroup {
                name: "本地".to_string(),
                sessions: vec![
                    SessionProfile::Local(LocalProfile {
                        id: "local-powershell".to_string(),
                        name: "PowerShell".to_string(),
                        shell: "powershell.exe".to_string(),
                    }),
                ],
            });
        }
        Self {
            active_tab: None,
            sidebar_width: 200.0,
            profiles,
        }
    }
}
```

- [ ] **Step 4: 运行，确认侧边栏显示**

```bash
cargo run -p oxijade-app
```
Expected: 左侧显示 "📁 本地" 分组，内含 "🖥 PowerShell"，点击后高亮

- [ ] **Step 5: Commit**

```bash
git add oxijade-app/src/panels/sidebar.rs oxijade-app/src/app.rs
git commit -m "feat(app): implement sidebar with session list"
```

---

## Task 10: oxijade-app — Tab Bar

**Files:**
- Modify: `oxijade-app/src/panels/tab_bar.rs`
- Modify: `oxijade-app/src/app.rs`

- [ ] **Step 1: 更新 App 持有打开的标签列表**

```rust
// app.rs — 添加 open_tabs 字段
pub struct OxiJadeApp {
    pub active_tab: Option<String>,
    pub open_tabs: Vec<String>,  // 已打开的 session id，按顺序
    pub sidebar_width: f32,
    pub profiles: ProfileStore,
}

impl Default for OxiJadeApp {
    fn default() -> Self {
        // ... 同上，添加：
        Self {
            active_tab: None,
            open_tabs: Vec::new(),
            sidebar_width: 200.0,
            profiles: /* 同上 */,
        }
    }
}
```

在侧边栏点击时也要打开标签（更新 sidebar.rs 中点击逻辑）：

```rust
// sidebar.rs 中 session_row 的点击逻辑
if response.interact(egui::Sense::click()).clicked() {
    if !app.open_tabs.contains(&id.to_string()) {
        app.open_tabs.push(id.to_string());
    }
    app.active_tab = Some(id.to_string());
}
```

- [ ] **Step 2: 实现 Tab Bar**

```rust
// oxijade-app/src/panels/tab_bar.rs
use egui::{Color32, RichText, Ui};
use oxijade_config::SessionProfile;
use crate::app::OxiJadeApp;
use crate::theme::Theme;

pub fn show(ui: &mut Ui, app: &mut OxiJadeApp) {
    ui.horizontal(|ui| {
        // Logo
        ui.add_space(8.0);
        ui.label(
            RichText::new("⬡")
                .color(Theme::ACCENT_SSH)
                .size(16.0),
        );
        ui.label(
            RichText::new("OxiJade")
                .color(Theme::TEXT_PRIMARY)
                .size(13.0)
                .strong(),
        );
        ui.add_space(12.0);
        ui.separator();
        ui.add_space(4.0);

        // 标签页
        let tabs = app.open_tabs.clone();
        let mut to_close: Option<String> = None;

        for tab_id in &tabs {
            let is_active = app.active_tab.as_deref() == Some(tab_id.as_str());
            let tab_name = find_profile_name(&app.profiles.groups, tab_id)
                .unwrap_or_else(|| tab_id.clone());
            let accent = find_profile_accent(&app.profiles.groups, tab_id);

            let bg = if is_active { Theme::BG_PANEL } else { Color32::TRANSPARENT };
            let border_color = if is_active { accent } else { Color32::TRANSPARENT };

            let response = egui::Frame::none()
                .fill(bg)
                .rounding(egui::Rounding { nw: 4.0, ne: 4.0, sw: 0.0, se: 0.0 })
                .inner_margin(egui::Margin::symmetric(10.0, 4.0))
                .stroke(egui::Stroke::new(
                    if is_active { 2.0 } else { 0.0 },
                    border_color,
                ))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(&tab_name)
                                .color(if is_active { Theme::TEXT_PRIMARY } else { Theme::TEXT_MUTED })
                                .size(12.0),
                        );
                        // 关闭按钮
                        if ui.small_button(
                            RichText::new("×").color(Theme::TEXT_MUTED)
                        ).clicked() {
                            to_close = Some(tab_id.clone());
                        }
                    });
                })
                .response;

            if response.interact(egui::Sense::click()).clicked() {
                app.active_tab = Some(tab_id.clone());
            }
        }

        // 关闭标签
        if let Some(id) = to_close {
            app.open_tabs.retain(|t| t != &id);
            if app.active_tab.as_deref() == Some(&id) {
                app.active_tab = app.open_tabs.last().cloned();
            }
        }

        // 新建标签按钮
        if ui.button(RichText::new("+").color(Theme::ACCENT_LOCAL)).clicked() {
            // TODO Plan 2
        }
    });
}

fn find_profile_name(
    groups: &[oxijade_config::SessionGroup],
    id: &str,
) -> Option<String> {
    for group in groups {
        for session in &group.sessions {
            if session.id() == id {
                return Some(session.name().to_string());
            }
        }
    }
    None
}

fn find_profile_accent(
    groups: &[oxijade_config::SessionGroup],
    id: &str,
) -> egui::Color32 {
    for group in groups {
        for session in &group.sessions {
            if session.id() == id {
                return match session {
                    SessionProfile::Local(_) => Theme::ACCENT_LOCAL,
                    SessionProfile::Ssh(_) => Theme::ACCENT_SSH,
                };
            }
        }
    }
    Theme::ACCENT_LOCAL
}
```

- [ ] **Step 3: 运行，确认标签栏**

```bash
cargo run -p oxijade-app
```
Expected: 点击侧边栏的 PowerShell 后，顶部标签栏出现带紫色下划线的标签，点 × 可关闭

- [ ] **Step 4: Commit**

```bash
git add oxijade-app/src/panels/tab_bar.rs oxijade-app/src/panels/sidebar.rs oxijade-app/src/app.rs
git commit -m "feat(app): implement tab bar with open/close logic"
```

---

## Task 11: oxijade-app — 终端渲染 Widget

**Files:**
- Modify: `oxijade-app/src/panels/terminal.rs`
- Modify: `oxijade-app/src/app.rs`

- [ ] **Step 1: 将 TerminalGrid 和 LocalSession 集成到 App**

在 app.rs 中添加运行时状态（使用 `Arc<Mutex<>>` 以便跨线程共享 grid）：

```rust
// app.rs — 新增 import 和字段
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use oxijade_core::terminal::TerminalGrid;
use oxijade_core::session::local::LocalSession;
use tokio::sync::mpsc::{channel, Receiver};
use oxijade_core::session::SessionEvent;

pub struct RunningSession {
    pub grid: Arc<Mutex<TerminalGrid>>,
    pub local: Option<LocalSession>,
}

pub struct OxiJadeApp {
    pub active_tab: Option<String>,
    pub open_tabs: Vec<String>,
    pub sidebar_width: f32,
    pub profiles: ProfileStore,
    pub running: HashMap<String, RunningSession>,
    pub rt: tokio::runtime::Runtime,
}

impl Default for OxiJadeApp {
    fn default() -> Self {
        let rt = tokio::runtime::Runtime::new().unwrap();
        // ... 其余同之前
        Self {
            active_tab: None,
            open_tabs: Vec::new(),
            sidebar_width: 200.0,
            profiles: /* 同上 */,
            running: HashMap::new(),
            rt,
        }
    }
}
```

在侧边栏点击时启动真实会话（在 sidebar.rs 中）：

```rust
// sidebar.rs — 点击时启动 LocalSession
if response.interact(egui::Sense::click()).clicked() {
    if !app.open_tabs.contains(&id.to_string()) {
        app.open_tabs.push(id.to_string());
        // 启动会话
        if !app.running.contains_key(id) {
            if let SessionProfile::Local(local_profile) = profile {
                let grid = Arc::new(Mutex::new(TerminalGrid::new(220, 50)));
                let grid_clone = grid.clone();
                let (tx, mut rx) = channel::<SessionEvent>(256);
                let session = LocalSession::new(
                    id.to_string(),
                    local_profile.shell.clone(),
                    tx,
                ).ok();

                // 后台任务：将 SessionEvent::Output 写入 grid
                app.rt.spawn(async move {
                    while let Some(event) = rx.recv().await {
                        match event {
                            SessionEvent::Output(bytes) => {
                                let mut g = grid_clone.lock().unwrap();
                                g.process_bytes(&bytes);
                            }
                            SessionEvent::Exited => break,
                        }
                    }
                });

                app.running.insert(id.to_string(), RunningSession {
                    grid,
                    local: session,
                });
            }
        }
    }
    app.active_tab = Some(id.to_string());
}
```

- [ ] **Step 2: 实现 Terminal 渲染 Widget**

```rust
// oxijade-app/src/panels/terminal.rs
use egui::{Color32, FontId, Painter, Pos2, Rect, Sense, TextStyle, Ui, Vec2};
use oxijade_core::terminal::CellColor;
use crate::app::OxiJadeApp;
use crate::theme::Theme;

pub fn show(ui: &mut Ui, app: &mut OxiJadeApp) {
    let Some(tab_id) = app.active_tab.clone() else {
        ui.centered_and_justified(|ui| {
            ui.label(
                egui::RichText::new("选择或创建一个会话")
                    .color(Theme::TEXT_MUTED)
                    .size(14.0),
            );
        });
        return;
    };

    let Some(running) = app.running.get(&tab_id) else {
        ui.centered_and_justified(|ui| {
            ui.label(egui::RichText::new("会话未启动").color(Theme::TEXT_MUTED));
        });
        return;
    };

    let grid = running.grid.lock().unwrap();

    // 使用等宽字体计算单元格大小
    let font_id = FontId::monospace(14.0);
    let cell_w = 8.4_f32;  // 近似等宽字体宽度（14px）
    let cell_h = 18.0_f32;

    let available = ui.available_rect_before_wrap();
    let painter = ui.painter_at(available);

    // 渲染每行每列
    for row in 0..grid.rows {
        let mut col = 0;
        while col < grid.cols {
            let cell = grid.cell_at(col, row);

            let x = available.left() + col as f32 * cell_w;
            let y = available.top() + row as f32 * cell_h;
            let cell_rect = Rect::from_min_size(
                Pos2::new(x, y),
                Vec2::new(cell_w, cell_h),
            );

            // 背景
            let bg = Theme::resolve_color(&cell.bg, Theme::BG_PRIMARY);
            if bg != Theme::BG_PRIMARY {
                painter.rect_filled(cell_rect, 0.0, bg);
            }

            // 前景文字
            if cell.c != ' ' {
                let fg = Theme::resolve_color(&cell.fg, Theme::TEXT_PRIMARY);
                painter.text(
                    Pos2::new(x, y + cell_h * 0.15),
                    egui::Align2::LEFT_TOP,
                    cell.c.to_string(),
                    font_id.clone(),
                    fg,
                );
            }

            col += 1;
        }
    }

    // 光标
    let cx = available.left() + grid.cursor_x as f32 * cell_w;
    let cy = available.top() + grid.cursor_y as f32 * cell_h;
    let cursor_rect = Rect::from_min_size(
        Pos2::new(cx, cy),
        Vec2::new(cell_w, cell_h),
    );
    painter.rect_filled(cursor_rect, 0.0, Theme::ACCENT_SSH.linear_multiply(0.7));

    // 键盘输入
    drop(grid); // 释放 grid lock

    let response = ui.allocate_rect(available, Sense::click_and_drag());
    if response.hovered() {
        let events = ui.input(|i| i.events.clone());

        // 先取出 writer（避免同时借用 app.running 和 app.rt）
        let writer = app.running.get(&tab_id)
            .and_then(|r| r.local.as_ref())
            .map(|l| l.writer.clone());

        if let Some(writer) = writer {
            for event in &events {
                let bytes = event_to_bytes(event);
                if !bytes.is_empty() {
                    let w = writer.clone();
                    app.rt.spawn(async move {
                        use std::io::Write;
                        let mut guard = w.lock().unwrap();
                        let _ = guard.write_all(&bytes);
                        let _ = guard.flush();
                    });
                }
            }
        }
    }

    // 持续重绘以获取实时输出
    ui.ctx().request_repaint();
}

fn event_to_bytes(event: &egui::Event) -> Vec<u8> {
    match event {
        egui::Event::Text(text) => text.as_bytes().to_vec(),
        egui::Event::Key { key, pressed: true, modifiers, .. } => {
            use egui::Key;
            match key {
                Key::Enter => b"\r".to_vec(),
                Key::Backspace => b"\x08".to_vec(),
                Key::Tab => b"\t".to_vec(),
                Key::Escape => b"\x1b".to_vec(),
                Key::ArrowUp => b"\x1b[A".to_vec(),
                Key::ArrowDown => b"\x1b[B".to_vec(),
                Key::ArrowRight => b"\x1b[C".to_vec(),
                Key::ArrowLeft => b"\x1b[D".to_vec(),
                Key::Home => b"\x1b[H".to_vec(),
                Key::End => b"\x1b[F".to_vec(),
                Key::Delete => b"\x1b[3~".to_vec(),
                Key::C if modifiers.ctrl => b"\x03".to_vec(), // Ctrl+C
                Key::D if modifiers.ctrl => b"\x04".to_vec(), // Ctrl+D
                Key::L if modifiers.ctrl => b"\x0c".to_vec(), // Ctrl+L (clear)
                _ => vec![],
            }
        }
        _ => vec![],
    }
}
```

> **注意**：上面 `local.writer` 字段需要在 `LocalSession` 中设为 `pub`：
> ```rust
> // local.rs
> pub struct LocalSession {
>     pub id: String,
>     pub writer: Arc<Mutex<Box<dyn Write + Send>>>,
>     child: Box<dyn portable_pty::Child + Send>,
> }
> ```

- [ ] **Step 3: 运行，确认终端工作**

```bash
cargo run -p oxijade-app
```
Expected:
1. 点击侧边栏 "PowerShell"
2. 顶部出现标签页
3. 中间区域显示 PowerShell 提示符
4. 能输入命令并看到输出

- [ ] **Step 4: Commit**

```bash
git add oxijade-app/src/panels/terminal.rs oxijade-app/src/app.rs oxijade-core/src/session/local.rs
git commit -m "feat(app): implement terminal rendering and keyboard input"
```

---

## Task 12: 收尾 — 底部状态栏 + .gitignore

**Files:**
- Modify: `oxijade-app/src/app.rs`
- Create/Modify: `.gitignore`

- [ ] **Step 1: 添加底部状态栏**

在 `app.rs` 的 `update()` 中，在 `TopBottomPanel::top` 之后添加底部面板：

```rust
egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
    ui.horizontal(|ui| {
        ui.add_space(8.0);
        if let Some(tab_id) = &self.active_tab {
            ui.label(
                egui::RichText::new(format!("● {}", tab_id))
                    .color(Theme::ACCENT_LOCAL)
                    .size(11.0),
            );
        } else {
            ui.label(
                egui::RichText::new("空闲").color(Theme::TEXT_MUTED).size(11.0),
            );
        }
    });
});
```

- [ ] **Step 2: 更新 .gitignore**

在项目根 `.gitignore`（如不存在则创建）中添加：

```
.superpowers/
target/
*.pdb
```

- [ ] **Step 3: 全量测试**

```bash
cargo test
```
Expected: 所有测试通过

- [ ] **Step 4: 运行最终验证**

```bash
cargo run -p oxijade-app
```
Expected:
- 窗口标题：OxiJade
- 深色背景（`#0d1117`）
- 顶部标签栏带 ⬡ logo
- 左侧会话列表，有 "📁 本地 → 🖥 PowerShell"
- 点击 PowerShell → 打开标签 → 终端显示 PS 提示符
- 输入 `echo hello` 回车 → 看到输出
- 底部状态栏显示当前会话名

- [ ] **Step 5: 最终 Commit**

```bash
git add .
git commit -m "feat: OxiJade Plan 1 complete — local terminal MVP"
```

---

## 后续：Plan 2 预告

Plan 2 将在此基础上添加：
- `oxijade-core/src/session/ssh.rs` — russh SSH 会话
- 新建会话对话框（主机/用户名/认证方式）
- SFTP 文件上传下载
- 拖拽文件到终端窗口触发上传
- 分屏（SplitPane 二叉树）
- 文本搜索 + 高亮
