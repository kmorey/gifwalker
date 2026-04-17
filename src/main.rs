fn main() {
    if let Err(error) = gifwalker::app::run() {
        eprintln!("{error}");
    }
}
