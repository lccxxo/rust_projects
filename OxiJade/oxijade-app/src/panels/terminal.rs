// oxijade-app/src/panels/terminal.rs
use crate::app::OxiJadeApp;
use crate::theme::Theme;
use egui::{FontId, Pos2, Rect, Sense, Ui, Vec2};
use std::io::Write;

pub fn show(ui: &mut Ui, app: &mut OxiJadeApp) {
    let Some(tab_id) = app.active_tab.clone() else {
        ui.centered_and_justified(|ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label(
                    egui::RichText::new("⬡ OxiJade")
                        .color(Theme::ACCENT_SSH)
                        .size(28.0)
                        .strong(),
                );
                ui.add_space(12.0);
                ui.label(
                    egui::RichText::new("点击左侧会话列表中的会话来打开终端")
                        .color(Theme::TEXT_MUTED)
                        .size(14.0),
                );
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new("Click a session in the left panel to open a terminal")
                        .color(Theme::TEXT_MUTED)
                        .size(12.0),
                );
            });
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

    let font_id = FontId::monospace(14.0);
    // Approximate monospace cell dimensions for 14px font
    let cell_w = 8.4_f32;
    let cell_h = 18.0_f32;

    let available = ui.available_rect_before_wrap();
    let painter = ui.painter_at(available);

    // Paint each cell
    for row in 0..grid.rows {
        for col in 0..grid.cols {
            let cell = grid.cell_at(col, row);

            let x = available.left() + col as f32 * cell_w;
            let y = available.top() + row as f32 * cell_h;

            // Only render if within visible area
            if x > available.right() || y > available.bottom() {
                break;
            }

            let cell_rect = Rect::from_min_size(Pos2::new(x, y), Vec2::new(cell_w, cell_h));

            // Background
            let bg = Theme::resolve_color(&cell.bg, Theme::BG_PRIMARY);
            if bg != Theme::BG_PRIMARY {
                painter.rect_filled(cell_rect, 0.0, bg);
            }

            // Character
            if cell.c != ' ' {
                let fg = Theme::resolve_color(&cell.fg, Theme::TEXT_PRIMARY);
                painter.text(
                    Pos2::new(x, y + 2.0),
                    egui::Align2::LEFT_TOP,
                    cell.c.to_string(),
                    font_id.clone(),
                    fg,
                );
            }
        }
    }

    // Cursor block
    let cx = available.left() + grid.cursor_x as f32 * cell_w;
    let cy = available.top() + grid.cursor_y as f32 * cell_h;
    let cursor_rect = Rect::from_min_size(Pos2::new(cx, cy), Vec2::new(cell_w, cell_h));
    painter.rect_filled(cursor_rect, 0.0, Theme::ACCENT_SSH.linear_multiply(0.6));

    drop(grid); // release lock before borrowing app again

    // Collect keyboard input
    let events = ui.input(|i| i.events.clone());

    // Get writer handle (avoids borrow conflict with app.rt below)
    let writer = app
        .running
        .get(&tab_id)
        .and_then(|r| r.local.as_ref())
        .map(|l| l.writer.clone());

    if let Some(writer) = writer {
        for event in &events {
            let bytes = event_to_bytes(event);
            if !bytes.is_empty() {
                let w = writer.clone();
                app.rt.spawn(async move {
                    let mut guard = w.lock().unwrap();
                    let _ = guard.write_all(&bytes);
                    let _ = guard.flush();
                });
            }
        }
    }

    // Allocate full area as interactive so we can receive keyboard focus
    let _response = ui.allocate_rect(available, Sense::click());

    // Continuously repaint to show live terminal output
    ui.ctx().request_repaint();
}

fn event_to_bytes(event: &egui::Event) -> Vec<u8> {
    match event {
        egui::Event::Text(text) => text.as_bytes().to_vec(),
        egui::Event::Key {
            key,
            pressed: true,
            modifiers,
            ..
        } => {
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
                Key::C if modifiers.ctrl => b"\x03".to_vec(),
                Key::D if modifiers.ctrl => b"\x04".to_vec(),
                Key::L if modifiers.ctrl => b"\x0c".to_vec(),
                _ => vec![],
            }
        }
        _ => vec![],
    }
}
