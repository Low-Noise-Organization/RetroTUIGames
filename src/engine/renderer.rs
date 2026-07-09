use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

pub fn set_char(buf: &mut Buffer, x: u16, y: u16, ch: char, fg: Color, bg: Color) {
    let cell = buf.cell_mut((x, y)).expect("cell out of bounds");
    cell.set_symbol(ch.to_string().as_str());
    cell.set_fg(fg);
    cell.set_bg(bg);
}

pub fn draw_text(buf: &mut Buffer, x: u16, y: u16, text: &str, fg: Color, bg: Color) {
    for (i, c) in text.chars().enumerate() {
        if let Some(cell) = buf.cell_mut((x + i as u16, y)) {
            cell.set_symbol(c.to_string().as_str());
            cell.set_fg(fg);
            cell.set_bg(bg);
        }
    }
}

pub fn fill_rect(buf: &mut Buffer, rect: Rect, bg: Color) {
    for y in rect.y..rect.bottom() {
        for x in rect.x..rect.right() {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_bg(bg);
                if cell.symbol() == " " { cell.set_symbol(" "); }
            }
        }
    }
}

pub fn fill_rect_fg(buf: &mut Buffer, rect: Rect, ch: char, fg: Color, bg: Color) {
    for y in rect.y..rect.bottom() {
        for x in rect.x..rect.right() {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_symbol(ch.to_string().as_str());
                cell.set_fg(fg);
                cell.set_bg(bg);
            }
        }
    }
}

pub fn draw_border(buf: &mut Buffer, r: Rect, fg: Color, bg: Color) {
    if r.width < 2 || r.height < 2 { return; }
    let right = r.right() - 1;
    let bottom = r.bottom() - 1;
    for x in r.x + 1..right { set_char(buf, x, r.y, '─', fg, bg); }
    for x in r.x + 1..right { set_char(buf, x, bottom, '─', fg, bg); }
    for y in r.y + 1..bottom { set_char(buf, r.x, y, '│', fg, bg); }
    for y in r.y + 1..bottom { set_char(buf, right, y, '│', fg, bg); }
    set_char(buf, r.x, r.y, '┌', fg, bg);
    set_char(buf, right, r.y, '┐', fg, bg);
    set_char(buf, r.x, bottom, '└', fg, bg);
    set_char(buf, right, bottom, '┘', fg, bg);
}

pub fn draw_border_double(buf: &mut Buffer, r: Rect, fg: Color, bg: Color) {
    if r.width < 2 || r.height < 2 { return; }
    let right = r.right() - 1;
    let bottom = r.bottom() - 1;
    for x in r.x + 1..right { set_char(buf, x, r.y, '═', fg, bg); }
    for x in r.x + 1..right { set_char(buf, x, bottom, '═', fg, bg); }
    for y in r.y + 1..bottom { set_char(buf, r.x, y, '║', fg, bg); }
    for y in r.y + 1..bottom { set_char(buf, right, y, '║', fg, bg); }
    set_char(buf, r.x, r.y, '╔', fg, bg);
    set_char(buf, right, r.y, '╗', fg, bg);
    set_char(buf, r.x, bottom, '╚', fg, bg);
    set_char(buf, right, bottom, '╝', fg, bg);
}

pub fn draw_starfield(buf: &mut Buffer, area: Rect, time: f32) {
    if area.width == 0 || area.height == 0 { return; }
    let star_bases = [
        ( 5,10),(15, 3),(25,20),(35, 5),(45,15),(55, 8),(65,22),(75,12),(85,4),(95,18),
        (10,80),(20,90),(30,85),(40,95),(50,88),(60,92),(70,82),(80,96),(90,87),(98,93),
        ( 8,30),(18,50),(28,70),(38,40),(48,60),(58,35),(68,55),(78,45),(88,65),(98,25),
        ( 3,65),(12,75),(22,45),(32,15),(42,25),(52,55),(62,38),(72,18),(82,58),(92,78),
        ( 7,48),(17,68),(27,88),(37,28),(47, 8),(57,48),(67,78),(77,38),(87,58),(97,12),
        ( 4,48),(14, 8),(24,88),(34,68),(44,38),(54,18),(64,58),(74,98),(84,78),(94,28),
    ];
    for (i, &(xp, yp)) in star_bases.iter().enumerate() {
        let phase_x = i as f32 * 1.7;
        let phase_y = i as f32 * 2.3;
        let speed = 0.3 + (i as f32 * 0.0517).fract() * 0.5;
        let amp_x = 0.5 + (i as f32 * 0.37).fract() * 2.5;
        let amp_y = 0.5 + (i as f32 * 0.53).fract() * 2.0;

        let base_x = area.x as f32 + area.width as f32 * xp as f32 / 100.0;
        let base_y = area.y as f32 + area.height as f32 * yp as f32 / 100.0;
        let dx = (time * speed + phase_x).sin() * amp_x;
        let dy = (time * speed * 0.7 + phase_y).cos() * amp_y;

        let x = (base_x + dx) as u16;
        let y = (base_y + dy) as u16;

        if x >= area.right() || y >= area.bottom() { continue; }

        let twinkle = ((time * 2.0 + i as f32 * 0.7).sin() * 0.5 + 0.5).max(0.3);
        let b = (40.0 + 60.0 * twinkle) as u8;
        let b_blue = (b as f32 * 1.4).min(200.0) as u8;

        if let Some(cell) = buf.cell_mut((x, y)) {
            cell.set_symbol("·");
            cell.set_fg(Color::Rgb(b / 2, b, b_blue));
            cell.set_bg(Color::Rgb(0, 0, 0));
        }
    }
}

pub fn draw_arcade_cabinet(buf: &mut Buffer, area: Rect, fg: Color, accent: Color, bg: Color) {
    if area.width < 10 || area.height < 6 { return; }
    let right = area.right() - 1;
    let bottom = area.bottom() - 1;

    for y in area.y..=bottom {
        for x in area.x..=right {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_bg(Color::Rgb(0, 0, 0));
            }
        }
    }

    let title = " RETRO HUB ";
    let title_start = area.x + (area.width.saturating_sub(title.len() as u16)) / 2;

    let mut x = area.x;
    set_char(buf, x, area.y, '╔', accent, bg);
    x += 1;
    while x < title_start {
        set_char(buf, x, area.y, '═', fg, bg);
        x += 1;
    }
    for (i, c) in title.chars().enumerate() {
        set_char(buf, title_start + i as u16, area.y, c, accent, bg);
    }
    x = title_start + title.len() as u16;
    while x < right {
        set_char(buf, x, area.y, '═', fg, bg);
        x += 1;
    }
    set_char(buf, right, area.y, '╗', accent, bg);

    for y in area.y + 1..bottom {
        set_char(buf, area.x, y, '║', fg, bg);
        set_char(buf, right, y, '║', fg, bg);
    }

    set_char(buf, area.x, bottom, '╚', accent, bg);
    for x in area.x + 1..right {
        set_char(buf, x, bottom, '═', fg, bg);
    }
    set_char(buf, right, bottom, '╝', accent, bg);

    let scan_y = area.y + 1;
    if scan_y < bottom {
        for x in area.x + 1..right {
            let ch = if (x.wrapping_sub(area.x)) % 3 == 1 { '░' } else { ' ' };
            set_char(buf, x, scan_y, ch, accent, bg);
        }
    }

    let decor_bottom = bottom - 1;
    if decor_bottom > scan_y + 1 {
        for x in area.x + 2..right - 1 {
            let ch = if (x.wrapping_sub(area.x)) % 2 == 0 { '·' } else { ' ' };
            set_char(buf, x, decor_bottom, ch, fg, bg);
        }
    }
}
