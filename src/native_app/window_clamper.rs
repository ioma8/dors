#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScreenFrame {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorkingArea {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WindowFrame {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

pub fn build_allowed_work_area(
    screen: ScreenFrame,
    top_reserved_height: i32,
    dock_height: i32,
) -> WorkingArea {
    let y = screen.y + dock_height.max(0);
    let height = (screen.height - top_reserved_height.max(0) - dock_height.max(0)).max(0);

    WorkingArea {
        x: screen.x,
        y,
        width: screen.width.max(0),
        height,
    }
}

pub fn clamp_window_frame(frame: WindowFrame, area: WorkingArea) -> Option<WindowFrame> {
    let clamped_height = frame.height.min(area.height).max(0);
    let top = area.y + area.height;
    let max_y = top - clamped_height;
    let clamped_y = frame.y.clamp(area.y, max_y);
    let clamped = WindowFrame {
        x: frame.x,
        y: clamped_y,
        width: frame.width,
        height: clamped_height,
    };

    (clamped != frame).then_some(clamped)
}
