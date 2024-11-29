/**
 * List à faire encore:
 * - Clean le code, enlever les println! etc...
 * - Gerer l'annulation de l'indexation
 * - filtre de recherche etc..., icon, ouverture a la racine du path..
 */

use serde::{Serialize, Deserialize};
use rayon::prelude::*;
use dashmap::DashMap;
use ignore::WalkBuilder;
use num_cpus;
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;
use std::path::Path;
use std::time::SystemTime;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: u64,
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
    pub modified: SystemTime,
}

static FILES: Lazy<DashMap<u64, SearchResult>> = Lazy::new(|| DashMap::with_capacity(2_500_000));
static WORDS: Lazy<DashMap<String, Vec<u64>>> = Lazy::new(|| DashMap::with_capacity(100_000));
static FILE_COUNT: AtomicU64 = AtomicU64::new(0);
static NEXT_ID: AtomicU64 = AtomicU64::new(0);

pub struct FileSearcher {}

impl FileSearcher {
    pub fn new() -> Self {
        Self {}
    }

    pub fn clear_index(&self) {
        FILES.clear();
        WORDS.clear();
        FILE_COUNT.store(0, Ordering::Relaxed);
        NEXT_ID.store(0, Ordering::Relaxed);
        println!("🧹 Index vidé");
    }

    pub fn build_index<F>(&self, should_cancel: F)
    where F: Fn() -> bool + Send + Sync + 'static {
        self.clear_index();
        
        let start_time = Instant::now();
        println!("🔄 Démarrage de l'indexation...");

        let drives = Self::get_drives();
        println!("💾 Disques détectés: {:?}", drives);

        let should_cancel = Arc::new(should_cancel);

        drives.into_par_iter().for_each(|drive| {
            let should_cancel = should_cancel.clone();
            
            if (should_cancel)() {
                return;
            }

            println!("📂 Indexation du disque: {}", drive);
            
            WalkBuilder::new(&drive)
                .hidden(false)
                .git_ignore(false)
                .git_global(false)
                .git_exclude(false)
                .threads(num_cpus::get())
                .build_parallel()
                .run(|| {
                    let should_cancel = should_cancel.clone();
                    
                    Box::new(move |entry| {
                        let entry = match entry {
                            Ok(entry) => entry,
                            Err(_) => return ignore::WalkState::Continue,
                        };

                        // Vérifier AVANT chaque fichier
                        if (should_cancel)() {
                            return ignore::WalkState::Quit;
                        }

                        if let Ok(metadata) = entry.metadata() {
                            let count = FILE_COUNT.fetch_add(1, Ordering::Relaxed);
                            if count % 100_000 == 0 {
                                println!("⏳ {} fichiers trouvés...", count);
                            }

                            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
                            let name = entry.file_name().to_string_lossy().into_owned();

                            FILES.insert(id, SearchResult {
                                id,
                                name,
                                path: entry.path().to_string_lossy().into_owned(),
                                size: metadata.len(),
                                is_dir: metadata.is_dir(),
                                modified: metadata.modified().unwrap_or(SystemTime::now()),
                            });

                            // Vérifier APRÈS chaque insertion
                            if (should_cancel)() {
                                return ignore::WalkState::Quit;
                            }
                        }
                        
                        ignore::WalkState::Continue
                    })
                });
        });

        if (should_cancel)() {
            println!("⏹️ Indexation annulée");
            return;
        }

        let duration = start_time.elapsed();
        println!("✅ Indexation terminée!");
        println!("=== Statistiques d'indexation ===");
        println!("⏱️  Temps total: {:.2} secondes", duration.as_secs_f64());
        println!("📑 Nombre de fichiers indexés: {}", FILE_COUNT.load(Ordering::Relaxed));
        println!("📊 Moyenne: {:.2} fichiers/seconde", 
            FILE_COUNT.load(Ordering::Relaxed) as f64 / duration.as_secs_f64());
    }

    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        let words: Vec<String> = query.to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        if words.is_empty() {
            return Vec::new();
        }

        let mut matches = HashSet::new();
        let mut first = true;

        for word in &words {
            if let Some(ids) = WORDS.get(word) {
                if first {
                    matches.extend(ids.value());
                    first = false;
                } else {
                    matches.retain(|id| ids.value().contains(id));
                }
            } else if first {
                return Vec::new();
            }
        }

        // Convertir les IDs en résultats
        let mut results: Vec<_> = matches.iter()
            .filter_map(|id| FILES.get(id))
            .map(|r| r.value().clone())
            .collect();

        // Trier par date de modification
        results.sort_unstable_by(|a, b| b.modified.cmp(&a.modified));

        results
    }

    fn get_drives() -> Vec<String> {
        #[cfg(windows)]
        {
            (b'C'..=b'Z')  
                .filter_map(|c| {
                    let drive = format!("{}:\\", c as char);
                    if Path::new(&drive).exists() {
                        let path = Path::new(&drive);
                        if let Ok(metadata) = path.metadata() {
                            if metadata.is_dir() {
                                return Some(drive);
                            }
                        }
                    }
                    None
                })
                .collect()
        }

        #[cfg(not(windows))]
        {
            vec!["/".to_string()]
        }
    }
}