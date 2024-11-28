mod query;
mod scorer;

use std::time::SystemTime;
// use std::path::PathBuf;
// use super::index::entry::FileEntry;
use crate::everything::EverythingIndex;
// use super::path::PathTable;
// use parking_lot::RwLock;
// use std::sync::Arc;
// use lru::LruCache;
// use std::num::NonZeroUsize;
use crate::pages::everysup::MAX_DISPLAY_RESULTS;
// use crate::everything::index::bitmap::CharBitmap;
// use rayon::prelude::*;
// use std::sync::atomic::{AtomicUsize, Ordering};

// pub use query::{SearchQuery, SearchFilter};
// use scorer::SearchScorer;

#[derive(Clone, Debug)]
pub struct SearchResult {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
    pub modified: SystemTime,
    // pub score: f32,
}

pub struct SearchEngine {

    //scorer: SearchScorer,
}

impl SearchEngine {
    pub fn new() -> Self {
        Self {
            //scorer: SearchScorer::new(),
        }
    }

    pub fn search(&self, index: &EverythingIndex, query: &str) -> Vec<SearchResult> {
        let query_lower = query.to_lowercase();
        let mut seen_paths = std::collections::HashSet::new();
        let mut results = Vec::new();

        for block in &index.blocks {
            for entry in block.entries.iter() {
                if let Some(path) = index.path_table.get_path(entry.path_id) {
                    let path_str = path.to_string_lossy().to_string();
                    let name = path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();

                    if !path_str.contains(":\\") {
                        println!("Chemin incomplet trouvé:");
                        println!("  - Nom: {}", name);
                        println!("  - Chemin stocké: {}", path_str);
                        println!("  - PathBuf: {:?}", path);
                        println!("  - ID: {}", entry.path_id);
                    }

                    if name.to_lowercase().contains(&query_lower) {
                        if seen_paths.contains(&path_str) {
                            continue;
                        }
                        seen_paths.insert(path_str.clone());

                        results.push(SearchResult {
                            name,
                            path: path_str,
                            size: entry.decompress_size(),
                            is_dir: entry.is_dir(),
                            modified: entry.decompress_date(),
                            // score: 1.0
                        });

                        if results.len() >= MAX_DISPLAY_RESULTS {
                            break;
                        }
                    }
                }
            }
        }

        results.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        println!("=== {} résultats trouvés pour '{}' ===", results.len(), query);
        results
    }
}
