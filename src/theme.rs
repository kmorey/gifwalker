use std::{fs, path::PathBuf};

use directories::BaseDirs;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemePalette {
    pub selected_text: String,
    pub text: String,
    pub base: String,
    pub border: String,
}

impl Default for ThemePalette {
    fn default() -> Self {
        Self {
            selected_text: "#eceff4".to_string(),
            text: "#d8dee9".to_string(),
            base: "rgba(20, 24, 31, 0.88)".to_string(),
            border: "#4c566a".to_string(),
        }
    }
}

impl ThemePalette {
    pub fn from_walker_css(css: &str) -> Self {
        let mut palette = Self::default();

        for line in css.lines() {
            let line = line.trim();
            if !line.starts_with("@define-color") {
                continue;
            }

            let Some(body) = line
                .strip_prefix("@define-color")
                .map(str::trim)
                .and_then(|value| value.strip_suffix(';').map(str::trim))
            else {
                continue;
            };

            let Some((name, value)) = body.split_once(' ') else {
                continue;
            };
            let value = value.trim();

            match name.trim() {
                "selected-text" => palette.selected_text = value.to_string(),
                "text" => palette.text = value.to_string(),
                "base" => palette.base = value.to_string(),
                "border" => palette.border = value.to_string(),
                _ => {}
            }
        }

        palette
    }
}

pub fn load_walker_palette() -> ThemePalette {
    let css_path = walker_css_path();
    if let Ok(raw) = fs::read_to_string(css_path) {
        return ThemePalette::from_walker_css(&raw);
    }

    ThemePalette::default()
}

fn walker_css_path() -> PathBuf {
    if let Some(base) = BaseDirs::new() {
        return base
            .home_dir()
            .join(".config/omarchy/current/theme/walker.css");
    }

    PathBuf::from("walker.css")
}
