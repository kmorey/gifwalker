use std::cell::RefCell;
use std::process::Command;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use anyhow::{anyhow, Result};
use gdk_pixbuf::PixbufLoader;
use gio::File;
use gtk4::prelude::*;
use gtk4::{gdk, gio, glib, Application, CssProvider, EventControllerKey, Frame, Grid, Label, Picture, ScrolledWindow, Video};

use crate::clipboard::{copy_gif_bytes, copy_text};
use crate::config::{default_config_path, AppConfig, ConfigStore};
use crate::controller::{key_action_for_enter, next_search_generation, scroll_value_for_row, GridSelection, KeyAction, LiveSearchAction, GIF_GRID_COLUMNS};
use crate::giphy::GiphyClient;
use crate::model::GifItem;
use crate::theme::load_walker_palette;
use crate::ui::settings::open_settings_dialog;
use crate::ui::window::build_main_window;

pub fn clear_pending_source(pending: &mut Option<glib::SourceId>) {
    if let Some(source_id) = pending.take() {
        let context = glib::MainContext::default();
        if let Some(source) = context.find_source_by_id(&source_id) {
            if !source.is_destroyed() {
                source_id.remove();
            }
        }
    }
}

struct AppState {
    config: AppConfig,
    items: Vec<GifItem>,
    thumbnails: Vec<Option<gdk::Texture>>,
    selection: GridSelection,
}

struct SearchPayload {
    generation: u64,
    items: Vec<GifItem>,
    thumbnails: Vec<Option<Vec<u8>>>,
}

enum SearchMessage {
    Success(SearchPayload),
    Error(String),
}

impl AppState {
    fn new(config: AppConfig) -> Self {
        Self {
            config,
            items: Vec::new(),
            thumbnails: Vec::new(),
            selection: GridSelection::new(0, GIF_GRID_COLUMNS),
        }
    }

    fn clear_results(&mut self) {
        self.items.clear();
        self.thumbnails.clear();
        self.selection = GridSelection::new(0, GIF_GRID_COLUMNS);
    }

    fn set_results(&mut self, items: Vec<GifItem>, thumbnails: Vec<Option<gdk::Texture>>) {
        self.selection = GridSelection::new(items.len(), GIF_GRID_COLUMNS);
        self.items = items;
        self.thumbnails = thumbnails;
    }

    fn selected_item(&self) -> Option<&GifItem> {
        self.items.get(self.selection.index())
    }

    fn has_results(&self) -> bool {
        !self.items.is_empty()
    }
}

pub fn run() -> Result<()> {
    let runtime = Arc::new(tokio::runtime::Runtime::new()?);

    let app = Application::builder()
        .application_id("com.kevin.gifwalker")
        .build();

    app.connect_activate(move |application| {
        let store = ConfigStore::new(default_config_path());
        let initial_config = store.load().unwrap_or_default();
        let state = Rc::new(RefCell::new(AppState::new(initial_config)));
        let giphy_client = GiphyClient::new();
        let active_generation = Arc::new(AtomicU64::new(0));

        apply_css();

        let ui = build_main_window(application);
        let (search_tx, search_rx) = mpsc::channel::<SearchMessage>();
        let pending_search = Rc::new(RefCell::new(None::<glib::SourceId>));
        set_status_from_config(&ui.status_label, &state.borrow().config);
        refresh_results_view(&ui.results_grid, &ui.scroller, &state.borrow());

        {
            let window = ui.window.clone();
            let status_label = ui.status_label.clone();
            let state = Rc::clone(&state);
            let store = store.clone();
            ui.settings_button.connect_clicked(move |_| {
                show_settings_dialog(&window, Rc::clone(&state), store.clone(), &status_label);
            });
        }

        {
            let results_grid = ui.results_grid.clone();
            let scroller = ui.scroller.clone();
            let status_label = ui.status_label.clone();
            let state = Rc::clone(&state);
            let active_generation = Arc::clone(&active_generation);

            glib::timeout_add_local(Duration::from_millis(50), move || {
                while let Ok(message) = search_rx.try_recv() {
                    match message {
                        SearchMessage::Success(payload) => {
                            if payload.generation != active_generation.load(Ordering::Relaxed) {
                                continue;
                            }

                            let thumbnails = payload
                                .thumbnails
                                .into_iter()
                                .map(|bytes| bytes.and_then(|bytes| texture_from_bytes(&bytes).ok()))
                                .collect();

                            let mut state = state.borrow_mut();
                            state.set_results(payload.items, thumbnails);
                            status_label.set_label(&format!(
                                "Found {} result(s). Enter copies the GIF. Shift+Enter copies the URL.",
                                state.items.len()
                            ));
                            refresh_results_view(&results_grid, &scroller, &state);
                        }
                        SearchMessage::Error(error) => {
                            status_label.set_label(&format!("Search failed: {error}"));
                        }
                    }
                }

                glib::ControlFlow::Continue
            });
        }

        {
            let window = ui.window.clone();
            let status_label = ui.status_label.clone();
            let results_grid = ui.results_grid.clone();
            let scroller = ui.scroller.clone();
            let state = Rc::clone(&state);
            let store = store.clone();
            let giphy_client = giphy_client.clone();
            let search_tx = search_tx.clone();
            let pending_search = Rc::clone(&pending_search);
            let active_generation = Arc::clone(&active_generation);
            let local_generation = Rc::new(RefCell::new(0_u64));

            ui.search_entry.connect_changed(move |entry| {
                clear_pending_source(&mut pending_search.borrow_mut());

                match LiveSearchAction::from_input(entry.text().as_str()) {
                    LiveSearchAction::Clear => {
                        active_generation.fetch_add(1, Ordering::Relaxed);
                        state.borrow_mut().clear_results();
                        refresh_results_view(&results_grid, &scroller, &state.borrow());
                        set_status_from_config(&status_label, &state.borrow().config);
                    }
                    LiveSearchAction::OpenSettings => {
                        show_settings_dialog(&window, Rc::clone(&state), store.clone(), &status_label);
                    }
                    LiveSearchAction::Debounced(query) => {
                        let generation = next_search_generation(&mut local_generation.borrow_mut());
                        active_generation.store(generation, Ordering::Relaxed);
                        let status_label = status_label.clone();
                        let results_grid = results_grid.clone();
                        let scroller = scroller.clone();
                        let state = Rc::clone(&state);
                        let search_tx = search_tx.clone();
                        let giphy_client = giphy_client.clone();
                        let active_generation = Arc::clone(&active_generation);

                        *pending_search.borrow_mut() = Some(glib::timeout_add_local_once(
                            Duration::from_millis(250),
                            {
                                let pending_search = Rc::clone(&pending_search);
                                move || {
                                    clear_pending_source(&mut pending_search.borrow_mut());

                                    if generation != active_generation.load(Ordering::Relaxed) {
                                        return;
                                    }

                                let api_key = state.borrow().config.giphy_api_key.clone().unwrap_or_default();
                                if api_key.trim().is_empty() {
                                    status_label.set_label(
                                        "No Giphy API key set. Open /settings or Settings to save it.",
                                    );
                                    return;
                                }

                                status_label.set_label("Searching Giphy...");
                                state.borrow_mut().clear_results();
                                refresh_results_view(&results_grid, &scroller, &state.borrow());

                                spawn_search(
                                    generation,
                                    query,
                                    api_key,
                                    giphy_client.clone(),
                                    search_tx.clone(),
                                );
                                }
                            },
                        ));
                    }
                }
            });
        }

        {
            let status_label = ui.status_label.clone();
            let results_grid = ui.results_grid.clone();
            let scroller = ui.scroller.clone();
            let window = ui.window.clone();
            let state = Rc::clone(&state);
            let runtime = Arc::clone(&runtime);

            let key_controller = EventControllerKey::new();
            key_controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
            key_controller.connect_key_pressed(move |_, key, _, modifiers| {
                use gdk::Key;

                let alt_pressed = modifiers.contains(gdk::ModifierType::ALT_MASK);
                let shift_pressed = modifiers.contains(gdk::ModifierType::SHIFT_MASK);
                let handled = match key {
                    Key::Left => {
                        let mut state = state.borrow_mut();
                        if !state.has_results() {
                            false
                        } else {
                            state.selection.move_left();
                            refresh_results_view(&results_grid, &scroller, &state);
                            true
                        }
                    }
                    Key::Right => {
                        let mut state = state.borrow_mut();
                        if !state.has_results() {
                            false
                        } else {
                            state.selection.move_right();
                            refresh_results_view(&results_grid, &scroller, &state);
                            true
                        }
                    }
                    Key::Up => {
                        let mut state = state.borrow_mut();
                        if !state.has_results() {
                            false
                        } else {
                            state.selection.move_up();
                            refresh_results_view(&results_grid, &scroller, &state);
                            true
                        }
                    }
                    Key::Down => {
                        let mut state = state.borrow_mut();
                        if !state.has_results() {
                            false
                        } else {
                            state.selection.move_down();
                            refresh_results_view(&results_grid, &scroller, &state);
                            true
                        }
                    }
                    Key::Return | Key::KP_Enter => {
                        let state = state.borrow();
                        if let Some(item) = state.selected_item() {
                            if let Some(display) = gdk::Display::default() {
                                match key_action_for_enter(shift_pressed, alt_pressed, '\n') {
                                    KeyAction::CopyUrlAndClose => {
                                        if let Err(error) = copy_text(&display, &item.gif_url) {
                                            status_label.set_label(&format!("Copy failed: {error}"));
                                        } else {
                                            notify_copy("Copied GIF URL");
                                            status_label.set_label("Copied GIF URL.");
                                            window.close();
                                        }
                                    }
                                    KeyAction::CopyImageAndClose => {
                                        if let Err(error) =
                                            runtime.block_on(copy_gif_bytes(&display, &item.gif_url))
                                        {
                                            status_label.set_label(&format!("Copy failed: {error}"));
                                        } else {
                                            notify_copy("Copied GIF image");
                                            status_label.set_label("Copied GIF image data.");
                                            window.close();
                                        }
                                    }
                                    KeyAction::None => return glib::Propagation::Proceed,
                                }
                                true
                            } else {
                                status_label.set_label("Clipboard unavailable.");
                                true
                            }
                        } else {
                            false
                        }
                    }
                    Key::Escape => {
                        window.close();
                        true
                    }
                    _ => false,
                };

                if handled {
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            });
            ui.window.add_controller(key_controller);
        }

        ui.window.present();
    });

    app.run();
    Ok(())
}

fn notify_copy(summary: &str) {
    let _ = Command::new("notify-send")
        .arg("gifwalker")
        .arg(summary)
        .status();
}

fn spawn_search(
    generation: u64,
    query: String,
    api_key: String,
    giphy_client: GiphyClient,
    search_tx: mpsc::Sender<SearchMessage>,
) {
    std::thread::spawn(move || {
        let runtime = match tokio::runtime::Runtime::new() {
            Ok(runtime) => runtime,
            Err(error) => {
                let _ = search_tx.send(SearchMessage::Error(error.to_string()));
                return;
            }
        };

        match runtime.block_on(giphy_client.search(&query, &api_key)) {
            Ok(items) => {
                let thumbnails = items
                    .iter()
                    .map(|item| runtime.block_on(download_image_bytes(&item.thumbnail_url)).ok())
                    .collect();
                let _ = search_tx.send(SearchMessage::Success(SearchPayload {
                    generation,
                    items,
                    thumbnails,
                }));
            }
            Err(error) => {
                let _ = search_tx.send(SearchMessage::Error(error.to_string()));
            }
        }
    });
}

fn apply_css() {
    let provider = CssProvider::new();
    provider.load_from_data(include_str!("../assets/app.css"));

    let palette = load_walker_palette();
    let dynamic_css = format!(
        ".panel {{ background-color: {}; border-color: {}; color: {}; }} label {{ color: {}; }} .result-card-selected {{ border-color: {}; box-shadow: 0 0 0 1px {}; }}",
        palette.base,
        palette.border,
        palette.text,
        palette.text,
        palette.selected_text,
        palette.selected_text,
    );
    let dynamic_provider = CssProvider::new();
    dynamic_provider.load_from_data(&dynamic_css);

    if let Some(display) = gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        gtk4::style_context_add_provider_for_display(
            &display,
            &dynamic_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION + 1,
        );
    }
}

fn set_status_from_config(status_label: &Label, config: &AppConfig) {
    if config
        .giphy_api_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_none()
    {
        status_label.set_label("No Giphy API key set. Open /settings or Settings to save it.");
    } else {
        status_label.set_label("Start typing to search Giphy.");
    }
}

fn show_settings_dialog(
    window: &gtk4::ApplicationWindow,
    state: Rc<RefCell<AppState>>,
    store: ConfigStore,
    status_label: &Label,
) {
    let existing = state.borrow().config.giphy_api_key.clone();
    let status_label = status_label.clone();

    open_settings_dialog(window, existing.as_deref(), move |api_key| {
        let mut new_config = state.borrow().config.clone();
        new_config.giphy_api_key = if api_key.trim().is_empty() {
            None
        } else {
            Some(api_key.trim().to_string())
        };

        if let Err(error) = store.save(&new_config) {
            status_label.set_label(&format!("Could not save settings: {error}"));
            return;
        }

        state.borrow_mut().config = new_config.clone();
        set_status_from_config(&status_label, &new_config);
    });
}

fn refresh_results_view(grid: &Grid, scroller: &ScrolledWindow, state: &AppState) {
    while let Some(child) = grid.first_child() {
        grid.remove(&child);
    }

    for (index, item) in state.items.iter().enumerate() {
        let frame = Frame::new(None);
        frame.add_css_class("result-card");
        frame.set_hexpand(true);
        if index == state.selection.index() {
            frame.add_css_class("result-card-selected");
        }

        let card = gtk4::Box::new(gtk4::Orientation::Vertical, 8);

        if index == state.selection.index() {
            let video_file = File::for_uri(&item.preview_url);
            let video = Video::for_file(Some(&video_file));
            video.set_autoplay(true);
            video.set_loop(true);
            video.set_hexpand(true);
            video.set_size_request(220, 160);
            if let Some(stream) = video.media_stream() {
                stream.set_muted(true);
                stream.play();
            }
            card.append(&video);
        } else if let Some(texture) = state.thumbnails.get(index).and_then(|texture| texture.as_ref()) {
            let picture = Picture::for_paintable(texture);
            picture.set_width_request(220);
            picture.set_height_request(160);
            picture.set_can_shrink(true);
            card.append(&picture);
        }

        let title = if item.title.trim().is_empty() {
            "Untitled GIF"
        } else {
            item.title.as_str()
        };

        let label = Label::new(Some(title));
        label.set_wrap(true);
        label.set_xalign(0.0);
        label.set_max_width_chars(24);
        card.append(&label);

        frame.set_child(Some(&card));
        grid.attach(
            &frame,
            (index % GIF_GRID_COLUMNS) as i32,
            (index / GIF_GRID_COLUMNS) as i32,
            1,
            1,
        );
    }

    ensure_selection_visible(scroller, state.selection.index());
}

fn ensure_selection_visible(scroller: &ScrolledWindow, index: usize) {
    let adjustment = scroller.vadjustment();
    let page_size = adjustment.page_size();
    if page_size <= 0.0 {
        return;
    }

    let row = index / GIF_GRID_COLUMNS;
    let target = scroll_value_for_row(row, 220.0, 12.0, page_size, adjustment.value());
    if (target - adjustment.value()).abs() > f64::EPSILON {
        adjustment.set_value(target.min(adjustment.upper() - page_size).max(adjustment.lower()));
    }
}

fn texture_from_bytes(bytes: &[u8]) -> Result<gdk::Texture> {
    let loader = PixbufLoader::new();
    loader.write(bytes)?;
    loader.close()?;
    let pixbuf = loader
        .pixbuf()
        .ok_or_else(|| anyhow!("could not decode image bytes"))?;

    Ok(gdk::Texture::for_pixbuf(&pixbuf))
}

async fn download_image_bytes(url: &str) -> Result<Vec<u8>> {
    Ok(reqwest::get(url)
        .await?
        .error_for_status()?
        .bytes()
        .await?
        .to_vec())
}
