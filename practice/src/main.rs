use std::io;
use std::time::Duration;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

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
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => quit = true,
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
