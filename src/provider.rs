use anyhow::Result;
use std::future::Future;
use std::pin::Pin;

use crate::model::GifItem;

pub trait GifSearchProvider: Clone + Send + Sync + 'static {
    fn search<'a>(
        &'a self,
        query: &'a str,
        api_key: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<GifItem>>> + Send + 'a>>;
}
