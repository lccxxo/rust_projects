mod timer;
mod ui;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};
use std::io;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let mut app = App::new();
    while !app.quit {
        terminal.draw(|f| ui(f, &mut app))?;
        if let Err(e) = app.handle_input() {
            eprintln!("Error: {e}");
            break;
        }
    }
    ratatui::restore();
    Ok(())
}

struct App {
    messages: Vec<Message>,
    input: String,
    scroll: u16,
    quit: bool,
}

struct Message {
    sender: String,
    text: String,
}

impl App {
    fn new() -> Self {
        Self { messages: Vec::new(), input: String::new(), scroll: 0, quit: false }
    }

    fn handle_input(&mut self) -> io::Result<()> {
        use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                return Ok(());
            }
            match key.code {
                KeyCode::Esc => self.quit = true,
                KeyCode::Enter => {
                    // Alt+Enter / Ctrl+Enter for newline; plain Enter sends
                    if key
                        .modifiers
                        .intersects(KeyModifiers::ALT | KeyModifiers::CONTROL)
                    {
                        self.input.push('\n');
                    } else {
                        let msg = std::mem::take(&mut self.input);
                        if !msg.trim().is_empty() {
                            self.messages.push(Message { sender: "你".into(), text: msg.clone() });
                            self.messages.push(Message { sender: "机器人".into(), text: msg });
                            // auto-scroll to bottom
                            self.scroll = 0;
                        }
                    }
                }
                KeyCode::Backspace => {
                    self.input.pop();
                }
                KeyCode::Up => {
                    self.scroll = self.scroll.saturating_add(1);
                }
                KeyCode::Down => {
                    self.scroll = self.scroll.saturating_sub(1);
                }
                KeyCode::PageUp => {
                    self.scroll = self.scroll.saturating_add(5);
                }
                KeyCode::PageDown => {
                    self.scroll = self.scroll.saturating_sub(5);
                }
                KeyCode::Char(c) => {
                    self.input.push(c);
                }
                _ => {}
            }
        }
        Ok(())
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let area = f.area();

    // ── Vertical split: messages | input ──
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(input_height(area))])
        .split(area);

    // ── Message area ──
    let msg_lines: Vec<Line> = app
        .messages
        .iter()
        .map(|m| message_line(m))
        .collect();

    // Calculate how many lines the paragraph would render
    let total_lines = msg_lines.len().max(1) as u16;
    let visible_lines = chunks[0].height.saturating_sub(2); // accounting for border
    let max_scroll = total_lines.saturating_sub(visible_lines);
    let scroll = app.scroll.min(max_scroll);

    let msg_para = Paragraph::new(Text::from(msg_lines))
        .block(Block::default().borders(Borders::ALL).title("对话"))
        .scroll((scroll, 0))
        .wrap(Wrap { trim: false });

    f.render_widget(msg_para, chunks[0]);

    // Scrollbar
    if max_scroll > 0 {
        let mut scrollbar_state =
            ScrollbarState::new(max_scroll as usize).position(scroll as usize);
        f.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓")),
            chunks[0].inner(Margin { horizontal: 0, vertical: 1 }),
            &mut scrollbar_state,
        );
    }

    // ── Input area ──
    let input_block = Block::default()
        .borders(Borders::ALL)
        .title("输入 (Enter 发送, Alt+Enter 换行, Esc 退出)");

    let input_text = if app.input.is_empty() {
        Text::from(Span::styled("▎", Style::default().fg(Color::DarkGray)))
    } else {
        Text::from(app.input.as_str())
    };

    let input_para = Paragraph::new(input_text)
        .block(input_block)
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: false });

    f.render_widget(input_para, chunks[1]);

    // Set cursor position after rendering input
    let cursor_col = cursor_col_for(&app.input) as u16;
    let row = app.input.lines().count().saturating_sub(1) as u16;
    f.set_cursor_position((chunks[1].x + 1 + cursor_col, chunks[1].y + 1 + row));
}

fn message_line(m: &Message) -> Line<'_> {
    let (label, color) = if m.sender == "你" {
        ("你", Color::Cyan)
    } else {
        ("机器人", Color::Green)
    };
    let prefix = Span::styled(
        format!("══ {label} ═"),
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    );
    Line::from(vec![prefix, Span::raw(" "), Span::raw(&m.text)])
}

fn input_height(area: ratatui::layout::Rect) -> u16 {
    // 3 lines min, up to 40% of screen
    (area.height / 3).clamp(3, 10)
}

fn cursor_col_for(input: &str) -> usize {
    input.lines().last().map(|l| l.len()).unwrap_or(0)
}

use ratatui::layout::Margin;
