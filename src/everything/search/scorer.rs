// use std::path::Path;
// use super::query::SearchQuery;
// use crate::everything::index::entry::FileEntry;

// pub struct SearchScorer;

// impl SearchScorer {
//     pub fn new() -> Self {
//         Self
//     }

//     pub fn calculate_score(&self, query: &str, entry: &FileEntry, path: &Path) -> f32 {
//         let name = path.file_name()
//             .and_then(|n| n.to_str())
//             .unwrap_or_default();
            
//         let name_lower = name.to_lowercase();
//         let path_lower = path.to_string_lossy().to_lowercase();
//         let query_lower = query.to_lowercase();

//         let name_score = if let Some(pos) = name_lower.find(&query_lower) {
//             let position_multiplier = 1.0 / (1.0 + pos as f32);
//             let length_ratio = query_lower.len() as f32 / name_lower.len() as f32;
//             3.0 * position_multiplier * length_ratio
//         } else {
//             0.0
//         };

//         let path_score = if let Some(_) = path_lower.find(&query_lower) {
//             1.0 / path_lower.len() as f32
//         } else {
//             0.0
//         };

//         let exact_match_score = if name_lower == query_lower {
//             5.0
//         } else if name_lower.starts_with(&query_lower) {
//             3.0
//         } else {
//             0.0
//         };

//         name_score + path_score + exact_match_score
//     }
// }
