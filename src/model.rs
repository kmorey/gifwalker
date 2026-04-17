#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GifItem {
    pub id: String,
    pub title: String,
    pub thumbnail_url: String,
    pub preview_url: String,
    pub gif_url: String,
    pub page_url: String,
}
