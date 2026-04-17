use gtk4::prelude::*;
use gtk4::{
    Align, Application, ApplicationWindow, Box, Button, Entry, Grid, Label, Orientation,
    ScrolledWindow,
};

pub struct MainWindow {
    pub window: ApplicationWindow,
    pub search_entry: Entry,
    pub settings_button: Button,
    pub status_label: Label,
    pub scroller: ScrolledWindow,
    pub results_grid: Grid,
}

pub fn build_main_window(application: &Application) -> MainWindow {
    let window = ApplicationWindow::builder()
        .application(application)
        .title("gifwalker")
        .default_width(1100)
        .default_height(760)
        .decorated(false)
        .build();

    let panel = Box::new(Orientation::Vertical, 12);
    panel.add_css_class("panel");

    let top_row = Box::new(Orientation::Horizontal, 8);

    let search_entry = Entry::new();
    search_entry.set_hexpand(true);
    search_entry.set_placeholder_text(Some("Search GIFs or /settings"));

    let settings_button = Button::with_label("Settings");
    settings_button.add_css_class("flat");

    top_row.append(&search_entry);
    top_row.append(&settings_button);

    let status_label = Label::new(None);
    status_label.set_halign(Align::Start);
    status_label.add_css_class("dim-label");

    let results_grid = Grid::new();
    results_grid.set_column_spacing(12);
    results_grid.set_row_spacing(12);
    results_grid.set_column_homogeneous(true);
    results_grid.set_hexpand(true);

    let scroller = ScrolledWindow::new();
    scroller.set_hexpand(true);
    scroller.set_vexpand(true);
    scroller.set_child(Some(&results_grid));

    panel.append(&top_row);
    panel.append(&status_label);
    panel.append(&scroller);

    window.set_child(Some(&panel));

    MainWindow {
        window,
        search_entry,
        settings_button,
        status_label,
        scroller,
        results_grid,
    }
}
