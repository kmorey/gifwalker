use gtk4::prelude::*;
use gtk4::{Button, Dialog, Orientation, PasswordEntry, Window};

pub fn open_settings_dialog<F>(parent: &impl IsA<Window>, existing_key: Option<&str>, on_save: F)
where
    F: Fn(String) + 'static,
{
    let dialog = Dialog::builder()
        .transient_for(parent)
        .modal(true)
        .title("Settings")
        .build();

    let content = dialog.content_area();
    content.set_orientation(Orientation::Vertical);
    content.set_spacing(8);

    let entry = PasswordEntry::new();
    entry.set_placeholder_text(Some("Giphy API key"));
    if let Some(value) = existing_key {
        entry.set_text(value);
    }

    let save_button = Button::with_label("Save");

    content.append(&entry);
    content.append(&save_button);

    let dialog_clone = dialog.clone();
    save_button.connect_clicked(move |_| {
        on_save(entry.text().to_string());
        dialog_clone.close();
    });

    dialog.present();
}
