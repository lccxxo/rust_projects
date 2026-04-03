// oxijade-app/src/theme.rs
use egui::Color32;

pub struct Theme;

impl Theme {
    pub const BG_PRIMARY: Color32 = Color32::from_rgb(0x0d, 0x11, 0x17);
    pub const BG_PANEL: Color32 = Color32::from_rgb(0x16, 0x1b, 0x22);
    pub const BG_SELECTED: Color32 = Color32::from_rgb(0x1f, 0x29, 0x37);

    pub const ACCENT_SSH: Color32 = Color32::from_rgb(0xcb, 0xa6, 0xf7);
    pub const ACCENT_LOCAL: Color32 = Color32::from_rgb(0x89, 0xb4, 0xfa);

    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0xcd, 0xd6, 0xf4);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(0x6c, 0x70, 0x86);

    pub const ANSI_COLORS: [Color32; 16] = [
        Color32::from_rgb(0x45, 0x47, 0x5a),
        Color32::from_rgb(0xf3, 0x8b, 0xa8),
        Color32::from_rgb(0xa6, 0xe3, 0xa1),
        Color32::from_rgb(0xf9, 0xe2, 0xaf),
        Color32::from_rgb(0x89, 0xb4, 0xfa),
        Color32::from_rgb(0xcb, 0xa6, 0xf7),
        Color32::from_rgb(0x89, 0xdc, 0xeb),
        Color32::from_rgb(0xcd, 0xd6, 0xf4),
        Color32::from_rgb(0x58, 0x5b, 0x70),
        Color32::from_rgb(0xf3, 0x8b, 0xa8),
        Color32::from_rgb(0xa6, 0xe3, 0xa1),
        Color32::from_rgb(0xf9, 0xe2, 0xaf),
        Color32::from_rgb(0x89, 0xb4, 0xfa),
        Color32::from_rgb(0xcb, 0xa6, 0xf7),
        Color32::from_rgb(0x89, 0xdc, 0xeb),
        Color32::from_rgb(0xcd, 0xd6, 0xf4),
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
