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
