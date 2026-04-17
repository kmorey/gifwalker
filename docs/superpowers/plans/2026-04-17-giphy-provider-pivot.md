# Giphy Provider Pivot Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the Tenor-only backend with a small provider layer and a Giphy implementation while keeping the existing overlay UI and copy actions unchanged.

**Architecture:** Introduce a provider-facing module for Giphy response mapping and request building, then update app/config/settings code to consume that provider instead of the Tenor client directly. Keep the UI stable and limit surface-area changes to backend-specific naming, tests, and documentation.

**Tech Stack:** Rust, GTK4, Reqwest, Tokio, Serde, TOML

---

### Task 1: Rename Config And Settings To Giphy

**Files:**
- Modify: `src/config.rs`
- Modify: `src/app.rs`
- Modify: `src/ui/settings.rs`
- Test: `tests/config.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn persists_giphy_api_key_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(dir.path().join("config.toml"));

    store
        .save(&AppConfig {
            giphy_api_key: Some("giphy-key".into()),
        })
        .unwrap();

    let loaded = store.load().unwrap();
    assert_eq!(loaded.giphy_api_key.as_deref(), Some("giphy-key"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test config persists_giphy_api_key_round_trip -q`
Expected: FAIL because `giphy_api_key` does not exist yet.

- [ ] **Step 3: Write minimal implementation**

```rust
pub struct AppConfig {
    pub giphy_api_key: Option<String>,
}
```

Update app/settings code to read and display `Giphy API key` instead of `Tenor API key`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test config -q`
Expected: PASS

### Task 2: Add Giphy Request Builder And Response Mapping

**Files:**
- Create: `src/provider.rs`
- Create: `src/giphy.rs`
- Modify: `src/lib.rs`
- Test: `tests/giphy.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn builds_giphy_search_url_with_required_parameters() {
    let url = build_search_url("dancing cat", "demo-key").unwrap();

    assert!(url.contains("api_key=demo-key"));
    assert!(url.contains("q=dancing+cat"));
    assert!(url.contains("limit=24"));
}

#[test]
fn maps_giphy_images_into_gif_items() {
    // sample response containing fixed_width_small_still, original_still, original, url, title
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test giphy -q`
Expected: FAIL because the Giphy module does not exist.

- [ ] **Step 3: Write minimal implementation**

```rust
pub struct GiphyClient { /* reqwest client */ }

impl GiphyClient {
    pub async fn search(&self, query: &str, api_key: &str) -> anyhow::Result<Vec<GifItem>> { /* ... */ }
}
```

Map Giphy image fields to:
- `thumbnail_url`: `images.fixed_width_small_still.url` or `images.original_still.url`
- `preview_url`: `images.original.url`
- `gif_url`: `images.original.url`
- `page_url`: `url`
- `title`: `title`

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test giphy -q`
Expected: PASS

### Task 3: Switch App Wiring From Tenor To Giphy

**Files:**
- Modify: `src/app.rs`
- Modify: `README.md`
- Delete: `src/tenor.rs`
- Delete: `tests/tenor.rs`

- [ ] **Step 1: Write the failing test**

Use the existing compile/test suite as the red step after updating imports in tests to the new Giphy module.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test giphy -q && cargo check -q`
Expected: FAIL until app imports and search strings are updated.

- [ ] **Step 3: Write minimal implementation**

Replace:
- `TenorClient` -> `GiphyClient`
- `tenor_api_key` -> `giphy_api_key`
- setup/status strings mentioning Tenor -> Giphy

- [ ] **Step 4: Run verification**

Run: `cargo test -q && cargo check -q`
Expected: PASS
