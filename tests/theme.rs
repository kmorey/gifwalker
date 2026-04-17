use gifwalker::theme::ThemePalette;

#[test]
fn parses_required_walker_colors() {
    let css = r#"
        @define-color selected-text #eceff4;
        @define-color text #d8dee9;
        @define-color base rgba(16, 18, 24, 0.86);
        @define-color border #4c566a;
    "#;

    let palette = ThemePalette::from_walker_css(css);

    assert_eq!(palette.selected_text, "#eceff4");
    assert_eq!(palette.text, "#d8dee9");
    assert_eq!(palette.base, "rgba(16, 18, 24, 0.86)");
    assert_eq!(palette.border, "#4c566a");
}

#[test]
fn falls_back_to_defaults_when_values_missing() {
    let css = "@define-color text #ffffff;";

    let palette = ThemePalette::from_walker_css(css);

    assert!(!palette.selected_text.is_empty());
    assert_eq!(palette.text, "#ffffff");
    assert!(!palette.base.is_empty());
    assert!(!palette.border.is_empty());
}
