// oxijade-app/src/panels/terminal.rs
use crate::app::{OxiJadeApp, SplitDir};
use crate::theme::Theme;
use egui::{FontId, Pos2, Rect, Sense, Ui, Vec2};
use std::io::Write;
use unicode_width::UnicodeWidthChar;

// Approximate monospace cell dimensions for 14px font
const CELL_W: f32 = 8.4;
const CELL_H: f32 = 18.0;

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

    if !app.running.contains_key(&tab_id) {
        ui.centered_and_justified(|ui| {
            ui.label(egui::RichText::new("会话未启动").color(Theme::TEXT_MUTED));
        });
        return;
    }

    // 显示启动错误
    if let Some(err) = app
        .running
        .get(&tab_id)
        .and_then(|r| r.error.as_ref())
        .cloned()
    {
        ui.centered_and_justified(|ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label(
                    egui::RichText::new("✗ 会话启动失败")
                        .color(egui::Color32::from_rgb(243, 139, 168))
                        .size(16.0)
                        .strong(),
                );
                ui.add_space(8.0);
                ui.label(egui::RichText::new(&err).color(Theme::TEXT_MUTED).size(12.0));
            });
        });
        return;
    }

    let available = ui.available_rect_before_wrap();

    // 读取分屏状态（避免重复借用）
    let split_info = app
        .running
        .get(&tab_id)
        .and_then(|r| r.split.as_ref())
        .map(|s| (s.session_id.clone(), s.direction, s.ratio, s.focus_secondary));

    // 检测鼠标点击位置（用于切换分屏焦点）
    let pointer_pos = ui.input(|i| i.pointer.press_origin());

    if let Some((sec_id, dir, ratio, focus_sec)) = split_info {
        const DIVIDER_W: f32 = 4.0;

        let (primary_rect, secondary_rect, divider_rect) = match dir {
            SplitDir::Horizontal => {
                let split_x = available.left() + available.width() * ratio;
                let prim = Rect::from_min_max(
                    available.min,
                    Pos2::new(split_x - DIVIDER_W / 2.0, available.max.y),
                );
                let sec = Rect::from_min_max(
                    Pos2::new(split_x + DIVIDER_W / 2.0, available.min.y),
                    available.max,
                );
                let div = Rect::from_min_max(
                    Pos2::new(split_x - DIVIDER_W / 2.0, available.min.y),
                    Pos2::new(split_x + DIVIDER_W / 2.0, available.max.y),
                );
                (prim, sec, div)
            }
            SplitDir::Vertical => {
                let split_y = available.top() + available.height() * ratio;
                let prim = Rect::from_min_max(
                    available.min,
                    Pos2::new(available.max.x, split_y - DIVIDER_W / 2.0),
                );
                let sec = Rect::from_min_max(
                    Pos2::new(available.min.x, split_y + DIVIDER_W / 2.0),
                    available.max,
                );
                let div = Rect::from_min_max(
                    Pos2::new(available.min.x, split_y - DIVIDER_W / 2.0),
                    Pos2::new(available.max.x, split_y + DIVIDER_W / 2.0),
                );
                (prim, sec, div)
            }
        };

        // 分隔线
        ui.painter().rect_filled(divider_rect, 0.0, Theme::BG_PANEL);

        // 拖拽分隔线改变 ratio
        let divider_response = ui.allocate_rect(divider_rect, Sense::drag());
        if divider_response.dragged() {
            let delta = divider_response.drag_delta();
            if let Some(primary) = app.running.get_mut(&tab_id) {
                if let Some(ref mut split) = primary.split {
                    let drag = match dir {
                        SplitDir::Horizontal => delta.x / available.width(),
                        SplitDir::Vertical => delta.y / available.height(),
                    };
                    split.ratio = (split.ratio + drag).clamp(0.15, 0.85);
                }
            }
        }

        // 点击切换焦点 pane
        if let Some(pos) = pointer_pos {
            if primary_rect.contains(pos) && focus_sec {
                if let Some(r) = app.running.get_mut(&tab_id) {
                    if let Some(ref mut s) = r.split {
                        s.focus_secondary = false;
                    }
                }
            } else if secondary_rect.contains(pos) && !focus_sec {
                if let Some(r) = app.running.get_mut(&tab_id) {
                    if let Some(ref mut s) = r.split {
                        s.focus_secondary = true;
                    }
                }
            }
        }

        // 渲染两个 pane
        render_pane(ui, primary_rect, &tab_id, !focus_sec, app);
        render_pane(ui, secondary_rect, &sec_id, focus_sec, app);
    } else {
        // 单 pane 模式
        render_pane(ui, available, &tab_id, true, app);
    }

    ui.ctx().request_repaint();
}

/// 渲染单个终端 pane 到指定矩形区域。
/// `is_active` 控制是否处理键盘输入。
fn render_pane(
    ui: &mut Ui,
    rect: egui::Rect,
    session_id: &str,
    is_active: bool,
    app: &mut OxiJadeApp,
) {
    let cols = ((rect.width() / CELL_W) as usize).max(10);
    let rows = ((rect.height() / CELL_H) as usize).max(3);

    // 调整 grid 和 PTY 尺寸
    {
        let running = match app.running.get_mut(session_id) {
            Some(r) => r,
            None => return,
        };
        let mut grid = running.grid.lock().unwrap();
        if grid.cols != cols || grid.rows != rows {
            grid.resize(cols, rows);
            drop(grid);
            if let Some(ref s) = running.local {
                s.resize(cols as u16, rows as u16);
            }
        }
    }

    // 滚轮和键盘输入（只处理激活 pane）
    let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
    let has_key = ui.input(|i| {
        i.events
            .iter()
            .any(|e| matches!(e, egui::Event::Key { pressed: true, .. } | egui::Event::Text(_)))
    });

    if is_active {
        let running = app.running.get_mut(session_id).unwrap();
        let grid = running.grid.lock().unwrap();
        let max_scroll = grid.scrollback.len();
        drop(grid);
        if scroll_delta > 0.0 {
            let lines = (scroll_delta / CELL_H * 3.0).ceil() as usize;
            running.scroll_offset = (running.scroll_offset + lines).min(max_scroll);
        } else if scroll_delta < 0.0 {
            let lines = ((-scroll_delta) / CELL_H * 3.0).ceil() as usize;
            running.scroll_offset = running.scroll_offset.saturating_sub(lines);
        }
        if has_key {
            running.scroll_offset = 0;
        }
    }

    let running = match app.running.get(session_id) {
        Some(r) => r,
        None => return,
    };
    let scroll_offset = running.scroll_offset;
    let grid = running.grid.lock().unwrap();

    let font_id = FontId::monospace(14.0);
    let painter = ui.painter_at(rect);

    let total_rows = grid.total_rows();
    let view_bottom = total_rows.saturating_sub(scroll_offset);
    let view_top = view_bottom.saturating_sub(rows);

    for (display_row, src_row) in (view_top..view_bottom).enumerate() {
        for col in 0..cols {
            let Some(cell) = grid.cell_at_history(col, src_row) else {
                continue;
            };
            let char_width = UnicodeWidthChar::width(cell.c).unwrap_or(1).max(1);
            let render_w = CELL_W * char_width as f32;
            let x = rect.left() + col as f32 * CELL_W;
            let y = rect.top() + display_row as f32 * CELL_H;
            if x > rect.right() || y > rect.bottom() {
                break;
            }

            let cell_rect = Rect::from_min_size(Pos2::new(x, y), Vec2::new(render_w, CELL_H));
            let bg = Theme::resolve_color(&cell.bg, Theme::BG_PRIMARY);
            if bg != Theme::BG_PRIMARY {
                painter.rect_filled(cell_rect, 0.0, bg);
            }
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

    // 光标
    {
        let cursor_abs_row = grid.scrollback.len() as isize + grid.cursor_y as isize;
        let display_row = cursor_abs_row - view_top as isize;
        if display_row >= 0 && display_row < rows as isize {
            let cx = rect.left() + grid.cursor_x as f32 * CELL_W;
            let cy = rect.top() + display_row as f32 * CELL_H;
            let color = if is_active {
                Theme::ACCENT_SSH.linear_multiply(0.6)
            } else {
                Theme::TEXT_MUTED.linear_multiply(0.3)
            };
            painter.rect_filled(
                Rect::from_min_size(Pos2::new(cx, cy), Vec2::new(CELL_W, CELL_H)),
                0.0,
                color,
            );
        }
    }

    // 历史模式提示条
    if scroll_offset > 0 {
        let bar = Rect::from_min_size(rect.min, Vec2::new(rect.width(), CELL_H));
        painter.rect_filled(
            bar,
            0.0,
            egui::Color32::from_rgba_premultiplied(0x16, 0x1b, 0x22, 220),
        );
        painter.text(
            Pos2::new(rect.center().x, rect.top() + 2.0),
            egui::Align2::CENTER_TOP,
            format!("── 历史记录（向下滚动返回） ──  ↑{}行", scroll_offset),
            FontId::proportional(11.0),
            Theme::TEXT_MUTED,
        );
    }

    drop(grid);

    // 激活 pane 边框
    if is_active {
        painter.rect_stroke(
            rect.shrink(1.0),
            0.0,
            egui::Stroke::new(1.0, Theme::ACCENT_SSH.linear_multiply(0.4)),
        );
    }

    // 键盘输入（只有激活 pane 处理）
    if is_active {
        let writer = app
            .running
            .get(session_id)
            .and_then(|r| r.local.as_ref())
            .map(|l| l.writer.clone());

        if let Some(writer) = writer {
            let events = ui.input(|i| i.events.clone());
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
    }

    ui.allocate_rect(rect, Sense::click());
}

fn event_to_bytes(event: &egui::Event) -> Vec<u8> {
    match event {
        egui::Event::Text(text) => {
            // 过滤掉控制字符（\x00-\x1f 和 \x7f）
            // Backspace/Escape 等已通过 Key 事件单独处理，
            // 若 Text 事件也携带这些字节会导致重复发送。
            text.bytes()
                .filter(|&b| b >= 0x20 && b != 0x7f)
                .collect()
        }
        egui::Event::Key {
            key,
            pressed: true,
            modifiers,
            ..
        } => {
            use egui::Key;
            match key {
                Key::Enter => b"\r".to_vec(),
                // \x7f (DEL) 是现代终端（Windows Terminal/ConPTY）的标准退格信号
                // \x08 (BS/Ctrl+H) 在 PSReadLine 里绑定的是"删整个单词"，不能用
                Key::Backspace if modifiers.ctrl => b"\x08".to_vec(),
                Key::Backspace => b"\x7f".to_vec(),
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
