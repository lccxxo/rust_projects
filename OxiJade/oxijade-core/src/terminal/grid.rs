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
            cells.push(Cell {
                c: ' ',
                ..Default::default()
            });
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
            cells.push(Cell {
                c: ' ',
                ..Default::default()
            });
        }
        self.cells = cells;
        self.cursor_x = self.cursor_x.min(cols.saturating_sub(1));
        self.cursor_y = self.cursor_y.min(rows.saturating_sub(1));
    }

    pub fn cell_at(&self, x: usize, y: usize) -> &Cell {
        &self.cells[y * self.cols + x]
    }

    pub fn process_byte(&mut self, byte: u8) {
        self.process_bytes(&[byte]);
    }

    pub fn process_bytes(&mut self, bytes: &[u8]) {
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
            self.cells.push(Cell {
                c: ' ',
                ..Default::default()
            });
        }
    }
}

struct GridPerformer<'a> {
    grid: &'a mut TerminalGrid,
}

impl<'a> Perform for GridPerformer<'a> {
    fn print(&mut self, c: char) {
        self.grid
            .set_cell(self.grid.cursor_x, self.grid.cursor_y, c);
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
                if self.grid.cursor_x > 0 {
                    self.grid.cursor_x -= 1;
                    self.grid
                        .set_cell(self.grid.cursor_x, self.grid.cursor_y, ' ');
                }
            }
            _ => {}
        }
    }

    fn csi_dispatch(
        &mut self,
        params: &Params,
        _intermediates: &[u8],
        _ignore: bool,
        action: char,
    ) {
        let ps: Vec<u16> = params.iter().map(|s| s[0]).collect();
        match action {
            'A' => {
                let n = ps.first().copied().unwrap_or(1).max(1) as usize;
                self.grid.cursor_y = self.grid.cursor_y.saturating_sub(n);
            }
            'B' => {
                let n = ps.first().copied().unwrap_or(1).max(1) as usize;
                self.grid.cursor_y = (self.grid.cursor_y + n).min(self.grid.rows - 1);
            }
            'C' => {
                let n = ps.first().copied().unwrap_or(1).max(1) as usize;
                self.grid.cursor_x = (self.grid.cursor_x + n).min(self.grid.cols - 1);
            }
            'D' => {
                let n = ps.first().copied().unwrap_or(1).max(1) as usize;
                self.grid.cursor_x = self.grid.cursor_x.saturating_sub(n);
            }
            'H' | 'f' => {
                let row = ps.first().copied().unwrap_or(1).max(1) as usize - 1;
                let col = ps.get(1).copied().unwrap_or(1).max(1) as usize - 1;
                self.grid.cursor_y = row.min(self.grid.rows - 1);
                self.grid.cursor_x = col.min(self.grid.cols - 1);
            }
            'J' => {
                let mode = ps.first().copied().unwrap_or(0);
                match mode {
                    0 => {
                        let start = self.grid.cursor_y * self.grid.cols + self.grid.cursor_x;
                        for i in start..self.grid.cells.len() {
                            self.grid.cells[i] = Cell {
                                c: ' ',
                                ..Default::default()
                            };
                        }
                    }
                    2 | 3 => {
                        for cell in &mut self.grid.cells {
                            *cell = Cell {
                                c: ' ',
                                ..Default::default()
                            };
                        }
                        self.grid.cursor_x = 0;
                        self.grid.cursor_y = 0;
                    }
                    _ => {}
                }
            }
            'K' => {
                let mode = ps.first().copied().unwrap_or(0);
                match mode {
                    0 => {
                        for x in self.grid.cursor_x..self.grid.cols {
                            self.grid.set_cell(x, self.grid.cursor_y, ' ');
                        }
                    }
                    1 => {
                        for x in 0..=self.grid.cursor_x {
                            self.grid.set_cell(x, self.grid.cursor_y, ' ');
                        }
                    }
                    2 => {
                        for x in 0..self.grid.cols {
                            self.grid.set_cell(x, self.grid.cursor_y, ' ');
                        }
                    }
                    _ => {}
                }
            }
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
                        100..=107 => {
                            self.grid.current_bg = CellColor::Indexed(ps[i] as u8 - 100 + 8)
                        }
                        38 if ps.get(i + 1) == Some(&2) && ps.len() > i + 4 => {
                            self.grid.current_fg =
                                CellColor::Rgb(ps[i + 2] as u8, ps[i + 3] as u8, ps[i + 4] as u8);
                            i += 4;
                        }
                        48 if ps.get(i + 1) == Some(&2) && ps.len() > i + 4 => {
                            self.grid.current_bg =
                                CellColor::Rgb(ps[i + 2] as u8, ps[i + 3] as u8, ps[i + 4] as u8);
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
        for b in b"\x1b[2J" {
            grid.process_byte(*b);
        }
        assert_eq!(grid.cell_at(0, 0).c, ' ');
        assert_eq!(grid.cursor_x, 0);
    }

    #[test]
    fn test_sgr_color() {
        let mut grid = TerminalGrid::new(80, 24);
        for b in b"\x1b[32mA" {
            grid.process_byte(*b);
        }
        match grid.cell_at(0, 0).fg {
            CellColor::Indexed(n) => assert_eq!(n, 2),
            _ => panic!("expected indexed color"),
        }
    }
}
