pub const GIF_GRID_COLUMNS: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    None,
    CopyImageAndClose,
    CopyUrlAndClose,
}

pub fn next_search_generation(generation: &mut u64) -> u64 {
    *generation += 1;
    *generation
}

pub fn scroll_value_for_row(
    row: usize,
    row_height: f64,
    row_spacing: f64,
    viewport_height: f64,
    current_value: f64,
) -> f64 {
    let item_top = row as f64 * (row_height + row_spacing);
    let item_bottom = item_top + row_height;
    let viewport_bottom = current_value + viewport_height;

    if item_top < current_value {
        item_top
    } else if item_bottom > viewport_bottom {
        (item_bottom - viewport_height).max(0.0)
    } else {
        current_value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputIntent {
    Empty,
    OpenSettings,
    Search(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiveSearchAction {
    Clear,
    OpenSettings,
    Debounced(String),
}

impl LiveSearchAction {
    pub fn from_input(input: &str) -> Self {
        match InputIntent::from_input(input) {
            InputIntent::Empty => Self::Clear,
            InputIntent::OpenSettings => Self::OpenSettings,
            InputIntent::Search(query) => Self::Debounced(query),
        }
    }
}

impl InputIntent {
    pub fn from_input(input: &str) -> Self {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            Self::Empty
        } else if trimmed == "/settings" {
            Self::OpenSettings
        } else {
            Self::Search(trimmed.to_string())
        }
    }

    pub fn copy_url_shortcut(alt_pressed: bool, key: char) -> bool {
        alt_pressed && (key == '\n' || key == '\r')
    }
}

pub fn key_action_for_enter(shift_pressed: bool, _alt_pressed: bool, key: char) -> KeyAction {
    if key != '\n' && key != '\r' {
        return KeyAction::None;
    }

    if shift_pressed {
        KeyAction::CopyUrlAndClose
    } else {
        KeyAction::CopyImageAndClose
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridSelection {
    len: usize,
    columns: usize,
    index: usize,
}

impl GridSelection {
    pub fn new(len: usize, columns: usize) -> Self {
        Self {
            len,
            columns: columns.max(1),
            index: 0,
        }
    }

    pub fn move_right(&mut self) {
        if self.index + 1 < self.len {
            self.index += 1;
        }
    }

    pub fn move_left(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.len == 0 {
            return;
        }

        self.index = (self.index + self.columns).min(self.len - 1);
    }

    pub fn move_up(&mut self) {
        self.index = self.index.saturating_sub(self.columns);
    }

    pub fn index(&self) -> usize {
        self.index
    }
}
