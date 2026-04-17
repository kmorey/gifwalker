use gifwalker::app::clear_pending_source;
use gtk4::glib;

#[test]
fn clearing_a_fired_timeout_source_is_safe() {
    let context = glib::MainContext::default();
    let mut pending = Some(glib::timeout_add_local_once(
        std::time::Duration::from_millis(0),
        || {},
    ));

    while context.pending() {
        context.iteration(false);
    }

    clear_pending_source(&mut pending);

    assert!(pending.is_none());
}
