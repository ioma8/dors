pub const DOCK_WINDOW_LEVEL: isize = 7;
pub const VISIBLE_DOCK_WINDOW_LAYER: isize = 20;
pub const OVERLAY_WINDOW_LEVEL: isize = 21;
pub const NONACTIVATING_PANEL_STYLE_MASK: usize = 1 << 7;

pub fn overlay_style_mask(style_mask: usize) -> usize {
    style_mask | NONACTIVATING_PANEL_STYLE_MASK
}
