use std::path::Path;

use lofty::file::TaggedFileExt;
use lofty::prelude::{Accessor, ItemKey};

pub struct TrackMeta {
    pub artist: String,
    pub title: String,
    pub album: String,
}

pub fn read_track_meta(path: &Path) -> TrackMeta {
    let fallback_title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown Title")
        .to_string();

    let mut meta = TrackMeta {
        artist: "Unknown Artist".to_string(),
        title: fallback_title.clone(),
        album: "Unknown Album".to_string(),
    };

    // Lofty 0.23: read_from_path(path) (no bool arg)
    if let Ok(tagged) = lofty::read_from_path(path) {
        // primary_tag/first_tag live behind TaggedFileExt
        let tag_opt = tagged.primary_tag().or_else(|| tagged.first_tag());

        if let Some(tag) = tag_opt {
            if let Some(v) = tag.artist() {
                let v = v.trim();
                if !v.is_empty() {
                    meta.artist = v.to_string();
                }
            }
            if let Some(v) = tag.title() {
                let v = v.trim();
                if !v.is_empty() {
                    meta.title = v.to_string();
                }
            }
            if let Some(v) = tag.album() {
                let v = v.trim();
                if !v.is_empty() {
                    meta.album = v.to_string();
                }
            }

            if meta.title == "Unknown Title" || meta.title == fallback_title {
                if let Some(v) = tag.get_string(ItemKey::TrackTitle) {
                    let v = v.trim();
                    if !v.is_empty() {
                        meta.title = v.to_string();
                    }
                }
            }

            if meta.artist == "Unknown Artist" {
                if let Some(v) = tag.get_string(ItemKey::TrackArtist) {
                    let v = v.trim();
                    if !v.is_empty() {
                        meta.artist = v.to_string();
                    }
                }
            }

            if meta.album == "Unknown Album" {
                if let Some(v) = tag.get_string(ItemKey::AlbumTitle) {
                    let v = v.trim();
                    if !v.is_empty() {
                        meta.album = v.to_string();
                    }
                }
            }
        }
    }

    meta
}
