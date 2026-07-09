use ratatui::layout::Rect;

pub fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

pub fn horizontal_stack(area: Rect, widths: &[u16], spacing: u16) -> Vec<Rect> {
    let total_width: u16 = widths.iter().sum::<u16>() + spacing * (widths.len().saturating_sub(1) as u16);
    let start_x = area.x + (area.width.saturating_sub(total_width)) / 2;
    let mut result = Vec::new();
    let mut cx = start_x;
    for &w in widths {
        result.push(Rect::new(cx, area.y, w, area.height));
        cx += w + spacing;
    }
    result
}

pub fn vertical_stack(area: Rect, heights: &[u16], spacing: u16) -> Vec<Rect> {
    let total_height: u16 = heights.iter().sum::<u16>() + spacing * (heights.len().saturating_sub(1) as u16);
    let start_y = area.y + (area.height.saturating_sub(total_height)) / 2;
    let mut result = Vec::new();
    let mut cy = start_y;
    for &h in heights {
        result.push(Rect::new(area.x, cy, area.width, h));
        cy += h + spacing;
    }
    result
}
