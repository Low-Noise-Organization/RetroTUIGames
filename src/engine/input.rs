use crossterm::event::{self, Event as CEvent, KeyCode, KeyEvent, KeyEventKind};

pub enum InputEvent {
    Key(i32, char),
    Resize(u16, u16),
    None,
}

pub fn poll_input(timeout_ms: u128) -> InputEvent {
    if event::poll(std::time::Duration::from_millis(timeout_ms as u64)).unwrap_or(false) {
        if let Ok(event) = event::read() {
            match event {
                CEvent::Key(KeyEvent { code, kind: KeyEventKind::Press | KeyEventKind::Repeat, .. }) => {
                    let (k, c) = key_code(code);
                    return InputEvent::Key(k, c);
                }
                CEvent::Resize(w, h) => return InputEvent::Resize(w, h),
                _ => {}
            }
        }
    }
    InputEvent::None
}

fn key_code(code: KeyCode) -> (i32, char) {
    match code {
        KeyCode::Up => (38, '\0'),
        KeyCode::Down => (40, '\0'),
        KeyCode::Left => (37, '\0'),
        KeyCode::Right => (39, '\0'),
        KeyCode::Enter => (10, '\n'),
        KeyCode::Esc => (27, '\0'),
        KeyCode::Backspace => (8, '\0'),
        KeyCode::Tab => (9, '\t'),
        KeyCode::BackTab => (9, '\t'),
        KeyCode::Delete => (127, '\0'),
        KeyCode::Home => (36, '\0'),
        KeyCode::End => (35, '\0'),
        KeyCode::PageUp => (33, '\0'),
        KeyCode::PageDown => (34, '\0'),
        KeyCode::Insert => (-1, '\0'),
        KeyCode::CapsLock => (-1, '\0'),
        KeyCode::ScrollLock => (-1, '\0'),
        KeyCode::NumLock => (-1, '\0'),
        KeyCode::PrintScreen => (-1, '\0'),
        KeyCode::Pause => (-1, '\0'),
        KeyCode::Menu => (-1, '\0'),
        KeyCode::KeypadBegin => (-1, '\0'),
        KeyCode::F(n) => (111 + n as i32, '\0'),
        KeyCode::Char(c) => (c as i32, c),
        KeyCode::Null => (-1, '\0'),
        KeyCode::Media(_) | KeyCode::Modifier(_) => (-1, '\0'),
    }
}
