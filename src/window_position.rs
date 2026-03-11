#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DockWindowPlacement {
    pub x: i32,
    pub y: i32,
}

pub fn bottom_center_placement(
    work_area_x: i32,
    work_area_y: i32,
    work_area_width: u32,
    work_area_height: u32,
    window_width: u32,
    window_height: u32,
    bottom_margin: i32,
) -> DockWindowPlacement {
    let x = work_area_x + ((work_area_width as i32 - window_width as i32) / 2);
    let y = work_area_y + work_area_height as i32 - window_height as i32 - bottom_margin;

    DockWindowPlacement { x, y }
}
