use gifwalker::giphy::{build_search_url, SearchResponse};

#[test]
fn builds_search_url_with_required_parameters() {
    let url = build_search_url("dancing cat", "demo-key").unwrap();

    assert!(url.contains("q=dancing+cat"));
    assert!(url.contains("api_key=demo-key"));
    assert!(url.contains("limit=24"));
}

#[test]
fn maps_giphy_results_into_gif_items() {
    let raw = r#"
    {
      "data": [
        {
          "id": "abc123",
          "title": "Dancing cat",
          "url": "https://giphy.com/gifs/dancing-cat-abc123",
          "images": {
            "fixed_width_still": { "url": "https://media.example/abc123-still.gif" },
            "preview": { "mp4": "https://media.example/abc123-preview.mp4" },
            "original": { "url": "https://media.example/abc123.gif", "mp4": "https://media.example/abc123.mp4" }
          }
        }
      ]
    }
    "#;

    let parsed: SearchResponse = serde_json::from_str(raw).unwrap();
    let items = parsed.into_gif_items();

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].id, "abc123");
    assert_eq!(items[0].title, "Dancing cat");
    assert_eq!(
        items[0].thumbnail_url,
        "https://media.example/abc123-still.gif"
    );
    assert_eq!(
        items[0].preview_url,
        "https://media.example/abc123-preview.mp4"
    );
    assert_eq!(items[0].gif_url, "https://media.example/abc123.gif");
    assert_eq!(
        items[0].page_url,
        "https://giphy.com/gifs/dancing-cat-abc123"
    );
}

#[test]
fn falls_back_to_original_still_when_small_still_missing() {
    let raw = r#"
    {
      "data": [
        {
          "id": "def456",
          "title": "Happy dog",
          "url": "https://giphy.com/gifs/happy-dog-def456",
          "images": {
            "original_still": { "url": "https://media.example/def456-still.gif" },
            "original": { "url": "https://media.example/def456.gif", "mp4": "https://media.example/def456.mp4" }
          }
        }
      ]
    }
    "#;

    let parsed: SearchResponse = serde_json::from_str(raw).unwrap();
    let items = parsed.into_gif_items();

    assert_eq!(
        items[0].thumbnail_url,
        "https://media.example/def456-still.gif"
    );
    assert_eq!(items[0].preview_url, "https://media.example/def456.mp4");
}
