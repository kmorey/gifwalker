# gifwalker

Walker-styled GTK overlay for searching Giphy GIFs on Omarchy.

## Usage

1. Run `cargo run`.
2. Click **Settings** or type `/settings`, then save your Giphy API key.
3. Enter a search query and wait briefly for results to load.
4. Use the arrow keys to move through the result grid.
5. Press `Enter` to copy GIF image data or `Shift+Enter` to copy the direct GIF URL.

## Config Path

Config is stored at `~/.config/gifwalker/config.toml`.

## Notes

- Theme colors are read from `~/.config/omarchy/current/theme/walker.css` when available, with safe defaults otherwise.
- Grid tiles use downloaded thumbnails in a 4-column browser layout.
- Hyprland placement is handled with a user rule matching `com.kevin.gifwalker`.
