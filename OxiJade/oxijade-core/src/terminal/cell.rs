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
