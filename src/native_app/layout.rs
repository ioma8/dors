#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PanelPlacement {
    pub x: i32,
    pub y: i32,
}

pub fn bottom_center_panel_placement(
    monitor_x: i32,
    monitor_y: i32,
    monitor_width: u32,
    monitor_height: u32,
    panel_width: u32,
    panel_height: u32,
) -> PanelPlacement {
    let x = monitor_x + ((monitor_width as i32 - panel_width as i32) / 2);
    let y = monitor_y + monitor_height as i32 - panel_height as i32;

    PanelPlacement { x, y }
}
