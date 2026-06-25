# Ink-Wash Pomodoro Timer — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Transform the chat TUI into a Kindle-eink-style Pomodoro timer with classic 25/5/15 cycle, pure keyboard control.

**Architecture:** Three modules — `timer` (FSM state machine), `ui` (ink-wash renderer), `main` (terminal init + event loop). The event loop polls keyboard with 100ms timeout, ticks the timer each iteration, and redraws.

**Tech Stack:** Rust, ratatui 0.30.2, crossterm (via ratatui re-export)

## Global Constraints

- ratatui 0.30.2 (already in Cargo.toml, no new deps)
- Classic Pomodoro durations: 25′/5′/15′, 4-pomodoro cycle
- Ink-wash palette: warm white bg `#F2ECD8`, ink `#2C2416`, mute sage green `#4A6B5D`
- Keyboard only: Space toggle, R reset, Q/Esc quit
- Terminal bell on phase transition
- No config, no persistence, no tests (TUI visual verification)

---

### Task 1: Create timer.rs — PomodoroFSM State Machine

**Files:**
- Create: `src/timer.rs`

**Interfaces:**
- Produces:
  - `Phase` enum: `{Work, ShortBreak, LongBreak}` — Copy+PartialEq
  - `RunState` enum: `{Running, Paused}` — Copy+PartialEq
  - `PomodoroFSM` struct with public fields `phase: Phase`, `run_state: RunState`, `completed: u8`, `remaining: u16`
  - `PomodoroFSM::new() -> Self`
  - `PomodoroFSM::tick(&mut self) -> bool` — advance countdown; returns true on auto-transition
  - `PomodoroFSM::toggle_pause(&mut self)`
  - `PomodoroFSM::reset_phase(&mut self)` — reset current phase duration to full, stays in same run_state
  - `PomodoroFSM::phase_label(&self) -> &'static str`

Step 1: Write src/timer.rs

```rust
use std::time::{Duration, Instant};

const WORK_SECS: u16 = 25 * 60;
const SHORT_BREAK_SECS: u16 = 5 * 60;
const LONG_BREAK_SECS: u16 = 15 * 60;
const POMODOROS_PER_CYCLE: u8 = 4;

#[derive(Clone, Copy, PartialEq)]
pub enum Phase {
    Work,
    ShortBreak,
    LongBreak,
}

#[derive(Clone, Copy, PartialEq)]
pub enum RunState {
    Running,
    Paused,
}

pub struct PomodoroFSM {
    pub phase: Phase,
    pub run_state: RunState,
    pub completed: u8,
    pub remaining: u16,
    last_tick: Instant,
}

impl PomodoroFSM {
    pub fn new() -> Self {
        Self {
            phase: Phase::Work,
            run_state: RunState::Paused,
            completed: 0,
            remaining: WORK_SECS,
            last_tick: Instant::now(),
        }
    }

    pub fn tick(&mut self) -> bool {
        if self.run_state != RunState::Running {
            return false;
        }
        let elapsed = self.last_tick.elapsed();
        if elapsed < Duration::from_secs(1) {
            return false;
        }
        self.last_tick = Instant::now();
        let secs = elapsed.as_secs().min(u16::MAX as u64) as u16;
        if secs >= self.remaining {
            self.remaining = 0;
            self.advance_phase();
            return true;
        }
        self.remaining -= secs;
        false
    }

    fn advance_phase(&mut self) {
        match self.phase {
            Phase::Work => {
                self.completed += 1;
                if self.completed >= POMODOROS_PER_CYCLE {
                    self.phase = Phase::LongBreak;
                    self.remaining = LONG_BREAK_SECS;
                    self.completed = 0;
                } else {
                    self.phase = Phase::ShortBreak;
                    self.remaining = SHORT_BREAK_SECS;
                }
            }
            Phase::ShortBreak | Phase::LongBreak => {
                self.phase = Phase::Work;
                self.remaining = WORK_SECS;
            }
        }
    }

    pub fn toggle_pause(&mut self) {
        self.run_state = match self.run_state {
            RunState::Running => RunState::Paused,
            RunState::Paused => RunState::Running,
        };
        self.last_tick = Instant::now();
    }

    pub fn reset_phase(&mut self) {
        self.remaining = match self.phase {
            Phase::Work => WORK_SECS,
            Phase::ShortBreak => SHORT_BREAK_SECS,
            Phase::LongBreak => LONG_BREAK_SECS,
        };
        self.last_tick = Instant::now();
    }

    pub fn phase_label(&self) -> &'static str {
        match self.phase {
            Phase::Work => "工 作",
            Phase::ShortBreak => "短 休",
            Phase::LongBreak => "长 休",
        }
    }
}
```

Step 2: Commit

```bash
git add src/timer.rs && git commit -m "feat: add PomodoroFSM state machine with 25/5/15 cycle"
```

---

### Task 2: Create ui.rs — Ink-Wash Renderer

**Files:**
- Create: `src/ui.rs`

**Interfaces:**
- Consumes: `crate::timer::{PomodoroFSM, RunState}` (from Task 1)
- Produces: `pub fn render(f: &mut ratatui::Frame, timer: &PomodoroFSM)` — full-frame render

Step 1: Write src/ui.rs

```rust
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::timer::{PomodoroFSM, RunState};

pub fn render(f: &mut Frame, timer: &PomodoroFSM) {
    let area = f.area();

    if area.width < 28 || area.height < 10 {
        render_too_small(f, area);
        return;
    }

    let bg = Color::Rgb(242, 236, 216);
    let ink = Color::Rgb(44, 36, 22);
    let dim = Color::Rgb(139, 129, 120);
    let dot_done = Color::Rgb(61, 50, 38);
    let dot_pending = Color::Rgb(204, 196, 184);
    let accent = Color::Rgb(74, 107, 93);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(5),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(area);

    let bg_block = Block::default().style(Style::default().bg(bg));
    f.render_widget(bg_block, area);

    render_timer(f, chunks[1], timer, ink, accent);

    let label = timer.phase_label();
    let label_text = if timer.run_state == RunState::Paused {
        format!("{label}  ·  暂停")
    } else {
        format!("  {label}")
    };
    let label_para = Paragraph::new(label_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(accent));
    f.render_widget(label_para, chunks[3]);

    render_dots(f, chunks[4], timer.completed, dot_done, dot_pending);

    let help = match timer.run_state {
        RunState::Running => "空格 暂停   R 重置   Q 退出",
        RunState::Paused => "空格 开始   R 重置   Q 退出",
    };
    let help_para = Paragraph::new(help)
        .alignment(Alignment::Center)
        .style(Style::default().fg(dim));
    f.render_widget(help_para, chunks[6]);
}

fn render_timer(f: &mut Frame, area: Rect, timer: &PomodoroFSM, ink: Color, accent: Color) {
    let mins = timer.remaining / 60;
    let secs = timer.remaining % 60;
    let time_str = format!("{:02}:{:02}", mins, secs);

    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Fill(1), Constraint::Length(11), Constraint::Fill(1)])
        .split(area);

    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(3), Constraint::Length(1)])
        .split(h_chunks[1]);

    let time_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(accent));
    let time_para = Paragraph::new(time_str)
        .block(time_block)
        .alignment(Alignment::Center)
        .style(Style::default().fg(ink));
    f.render_widget(time_para, v_chunks[1]);
}

fn render_dots(f: &mut Frame, area: Rect, completed: u8, done: Color, pending: Color) {
    let mut spans: Vec<Span> = Vec::with_capacity(7);
    for i in 0..4u8 {
        if i > 0 {
            spans.push(Span::from("  "));
        }
        if i < completed {
            spans.push(Span::styled("●", Style::default().fg(done)));
        } else {
            spans.push(Span::styled("○", Style::default().fg(pending)));
        }
    }
    let dot_para = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
    f.render_widget(dot_para, area);
}

fn render_too_small(f: &mut Frame, area: Rect) {
    let msg = Paragraph::new("Terminal too small")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Rgb(139, 129, 120)));
    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(1), Constraint::Fill(1)])
        .split(area);
    f.render_widget(msg, v_chunks[1]);
}
```

Step 2: Commit

```bash
git add src/ui.rs && git commit -m "feat: add ink-wash renderer with Kindle e-ink palette"
```

---

### Task 3: Rewrite main.rs — Wire Event Loop

**Files:**
- Modify: `src/main.rs` (full rewrite)

**Interfaces:**
- Consumes: `mod timer` (Task 1), `mod ui` (Task 2), `PomodoroFSM::new()`, `ui::render()`
- Produces: working binary

Step 1: Rewrite src/main.rs

```rust
use std::io;
use std::time::Duration;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};

mod timer;
mod ui;

use timer::PomodoroFSM;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let mut timer = PomodoroFSM::new();
    let mut quit = false;

    while !quit {
        terminal.draw(|f| ui::render(f, &timer))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => quit = true,
                    KeyCode::Char(' ') => timer.toggle_pause(),
                    KeyCode::Char('r') => timer.reset_phase(),
                    _ => {}
                }
            }
        }

        if timer.tick() {
            print!("\x07");
        }
    }

    ratatui::restore();
    Ok(())
}
```

Step 2: Commit

```bash
git add src/main.rs && git commit -m "feat: wire ink-wash pomodoro timer event loop"
```

---

### Task 4: Build and Verify

Step 1: Build the project

```bash
cargo build 2>&1
```
Expected: `Finished` with no errors, no warnings.

Step 2: Run and visually verify

```bash
cargo run
```

Expected behavior checklist:
- Terminal fills with warm white background (`#F2ECD8`)
- Timer shows `25:00` centered with sage-green border
- Label shows `工 作  ·  暂停` (paused by default)
- Dots show `○  ○  ○  ○`
- Help bar shows `空格 开始   R 重置   Q 退出`
- Press Space: label changes, timer counts down
- Press R: timer resets to 25:00
- Press Q/Esc: clean exit, terminal restored
