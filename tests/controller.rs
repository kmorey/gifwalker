use gifwalker::controller::{
    key_action_for_enter, next_search_generation, scroll_value_for_row, GridSelection, InputIntent,
    KeyAction, LiveSearchAction, GIF_GRID_COLUMNS,
};

#[test]
fn parses_settings_intent() {
    let intent = InputIntent::from_input(" /settings ");
    assert_eq!(intent, InputIntent::OpenSettings);
}

#[test]
fn parses_query_intent() {
    let intent = InputIntent::from_input(" dancing cats ");
    assert_eq!(intent, InputIntent::Search("dancing cats".to_string()));
}

#[test]
fn empty_input_is_empty_intent() {
    let intent = InputIntent::from_input("   ");
    assert_eq!(intent, InputIntent::Empty);
}

#[test]
fn move_right_then_down_selects_expected_index() {
    let mut selection = GridSelection::new(12, 4);
    selection.move_right();
    selection.move_down();

    assert_eq!(selection.index(), 5);
}

#[test]
fn down_moves_by_column_count() {
    let mut selection = GridSelection::new(10, 3);
    selection.move_down();

    assert_eq!(selection.index(), 3);
}

#[test]
fn move_left_steps_back_one_item() {
    let mut selection = GridSelection::new(10, 3);
    selection.move_right();
    selection.move_right();
    selection.move_left();

    assert_eq!(selection.index(), 1);
}

#[test]
fn move_up_steps_back_by_column_count() {
    let mut selection = GridSelection::new(10, 3);
    selection.move_down();
    selection.move_down();
    selection.move_up();

    assert_eq!(selection.index(), 3);
}

#[test]
fn enter_routes_to_copy_image() {
    assert_eq!(
        key_action_for_enter(false, false, '\n'),
        KeyAction::CopyImageAndClose
    );
}

#[test]
fn shift_enter_routes_to_copy_url() {
    assert_eq!(
        key_action_for_enter(true, false, '\n'),
        KeyAction::CopyUrlAndClose
    );
}

#[test]
fn alt_enter_does_not_route_to_copy_url_anymore() {
    assert_eq!(
        key_action_for_enter(false, true, '\n'),
        KeyAction::CopyImageAndClose
    );
}

#[test]
fn non_enter_key_does_not_trigger_copy_action() {
    assert_eq!(key_action_for_enter(true, false, 'c'), KeyAction::None);
}

#[test]
fn live_search_opens_settings_immediately() {
    assert_eq!(
        LiveSearchAction::from_input("/settings"),
        LiveSearchAction::OpenSettings
    );
}

#[test]
fn live_search_debounces_normal_queries() {
    assert_eq!(
        LiveSearchAction::from_input(" dancing cats "),
        LiveSearchAction::Debounced("dancing cats".to_string())
    );
}

#[test]
fn live_search_clears_for_empty_input() {
    assert_eq!(LiveSearchAction::from_input("   "), LiveSearchAction::Clear);
}

#[test]
fn gif_grid_uses_four_columns() {
    assert_eq!(GIF_GRID_COLUMNS, 4);
}

#[test]
fn search_generation_increments_monotonically() {
    let mut generation = 0;

    let first = next_search_generation(&mut generation);
    let second = next_search_generation(&mut generation);

    assert_eq!(first, 1);
    assert_eq!(second, 2);
    assert_eq!(generation, 2);
}

#[test]
fn scrolls_down_when_selected_row_falls_below_viewport() {
    let value = scroll_value_for_row(3, 180.0, 12.0, 400.0, 0.0);

    assert_eq!(value, 356.0);
}

#[test]
fn keeps_existing_scroll_when_row_is_already_visible() {
    let value = scroll_value_for_row(1, 180.0, 12.0, 400.0, 0.0);

    assert_eq!(value, 0.0);
}
