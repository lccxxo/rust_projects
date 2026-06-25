# Ink-Wash Pomodoro Timer — Design Spec

## Overview

Transform the current chat-app TUI into a Pomodoro timer with a Kindle e-ink
aesthetic (宣纸暖白 + 松烟墨色). Pure keyboard control, classic 25/5/15 cycle.

## State Machine

```
Work(25′) → ShortBreak(5′) → Work → ShortBreak → Work → ShortBreak → Work → LongBreak(15′) → loop
```

- 4 completed work sessions (pomodoros) trigger a long break; count resets after.
- States: `Running | Paused`. Space toggles. R resets current phase. Esc/Q quits.
- Run: countdown ticks every second. Pause: timer frozen, phase preserved.
- On phase completion: auto-transition + optional terminal bell (`\x07`).

## File Layout

```
src/
├── main.rs    # terminal init, event loop, App dispatch
├── timer.rs   # PomodoroFSM: phase transitions, tick, pause/resume, pomodoro count
└── ui.rs      # render(): full-frame ink-wash layout
```

## UI Layout

```
┌──────────────────────────────────────┐
│                                      │
│         ┌──────────────────┐         │
│         │     25:00        │         │  ← large countdown (center)
│         └──────────────────┘         │
│                                      │
│              工 作 中                │  ← phase label
│                                      │
│          ● ● ● ○                    │  ← pomodoro dots (4 max)
│                                      │
│      空格 开始  R 重置  Q 退出       │  ← help bar at bottom
└──────────────────────────────────────┘
```

- Large countdown: centered, no border, rendered with `text::Line` / `Span`.
- Phase label: centered below timer.
- Pomodoro dots: ● for completed, ○ for pending. Reset with cycle.
- Help bar: bottom-aligned, dim text. Shows context-sensitive hints
  (e.g. "空格 暂停" when running).

## Color Palette (Kindle E-Ink)

| Role           | Hex       | Notes                        |
|----------------|-----------|------------------------------|
| Background     | `#F2ECD8` | 宣纸暖白, warm off-white    |
| Primary text   | `#2C2416` | 松烟墨, deep warm charcoal  |
| Border/separator | `#8B8178` | 淡墨灰, muted gray-taupe |
| Dot-done       | `#3D3226` | 浓墨, richer ink tone       |
| Dot-pending    | `#CCC4B8` | 淡墨晕, warm light gray     |
| Active label   | `#4A6B5D` | 墨绿, subdued sage green     |
| Help text      | `#8B8178` | same as border dim gray      |
| Timer digits   | `#2C2416` | same as primary text         |

No bright colors, no fluorescent, no blinking. All tones muted and warm.

## Key Bindings

| Key   | Action                     |
|-------|----------------------------|
| Space | toggle Run/Pause           |
| R     | reset current phase        |
| Q/Esc | quit                        |

## Edge Cases

- Terminal too small (< 10 cols / 5 rows): show "Terminal too small" centered.
- Phase auto-transition: timer reaches 00:00 → bell → next phase auto-starts running.

## What's NOT Included

- No config file. Hardcoded 25/5/15 durations and 4-pomodoro cycle.
- No sound beyond terminal bell.
- No session persistence or history.
