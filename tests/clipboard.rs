use gifwalker::clipboard::{gif_wl_copy_args, text_wl_copy_args};

#[test]
fn text_clipboard_uses_plain_text_mime() {
    assert_eq!(
        text_wl_copy_args(),
        vec!["--type", "text/plain;charset=utf-8"]
    );
}

#[test]
fn gif_clipboard_uses_gif_mime() {
    assert_eq!(gif_wl_copy_args(), vec!["--type", "image/gif"]);
}
