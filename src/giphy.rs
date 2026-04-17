use std::pin::Pin;

use anyhow::Result;
use reqwest::Url;
use serde::Deserialize;

use crate::model::GifItem;
use crate::provider::GifSearchProvider;

const GIPHY_SEARCH_ENDPOINT: &str = "https://api.giphy.com/v1/gifs/search";

pub fn build_search_url(query: &str, api_key: &str) -> Result<String> {
    let mut url = Url::parse(GIPHY_SEARCH_ENDPOINT)?;
    url.query_pairs_mut()
        .append_pair("api_key", api_key)
        .append_pair("q", query)
        .append_pair("limit", "24");

    Ok(url.to_string())
}

#[derive(Clone, Debug)]
pub struct GiphyClient {
    client: reqwest::Client,
}

impl GiphyClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn search(&self, query: &str, api_key: &str) -> Result<Vec<GifItem>> {
        let url = build_search_url(query, api_key)?;
        let response = self
            .client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json::<SearchResponse>()
            .await?;

        Ok(response.into_gif_items())
    }
}

impl GifSearchProvider for GiphyClient {
    fn search<'a>(
        &'a self,
        query: &'a str,
        api_key: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<GifItem>>> + Send + 'a>> {
        Box::pin(async move { self.search(query, api_key).await })
    }
}

#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    #[serde(default)]
    pub data: Vec<GiphyItem>,
}

#[derive(Debug, Deserialize)]
pub struct GiphyItem {
    pub id: String,
    #[serde(default)]
    pub title: String,
    pub url: String,
    pub images: Images,
}

#[derive(Debug, Deserialize)]
pub struct Images {
    pub fixed_width_still: Option<ImageVariant>,
    pub fixed_width_small_still: Option<ImageVariant>,
    pub original_still: Option<ImageVariant>,
    pub preview: Option<VideoVariant>,
    pub original: ImageVariant,
}

#[derive(Debug, Deserialize)]
pub struct ImageVariant {
    pub url: String,
    pub mp4: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct VideoVariant {
    pub mp4: String,
}

impl SearchResponse {
    pub fn into_gif_items(self) -> Vec<GifItem> {
        self.data
            .into_iter()
            .map(|item| {
                let thumbnail_url = item
                    .images
                    .fixed_width_still
                    .as_ref()
                    .or(item.images.fixed_width_small_still.as_ref())
                    .or(item.images.original_still.as_ref())
                    .map(|image| image.url.clone())
                    .unwrap_or_else(|| item.images.original.url.clone());

                let preview_url = item
                    .images
                    .preview
                    .as_ref()
                    .map(|video| video.mp4.clone())
                    .or(item.images.original.mp4.clone())
                    .unwrap_or_else(|| item.images.original.url.clone());

                GifItem {
                    id: item.id,
                    title: item.title,
                    thumbnail_url,
                    preview_url,
                    gif_url: item.images.original.url,
                    page_url: item.url,
                }
            })
            .collect()
    }
}
