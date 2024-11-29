/**
 * List à faire encore:
 * - Clean le code, enlever les println! etc...
 * - Gerer l'annulation de l'indexation
 * - filtre de recherche etc..., icon, ouverture a la racine du path..
 */

use crate::utils::get_drives;
use serde::{Serialize, Deserialize};
use rayon::prelude::*;
use dashmap::DashMap;
use ignore::WalkBuilder;
use num_cpus;
use once_cell::sync::Lazy;
use std::sync::Arc;
use std::time::Instant;
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

static FILES: Lazy<DashMap<u64, SearchResult>> = Lazy::new(|| DashMap::with_capacity(500_000));
static NAME_INDEX: Lazy<DashMap<String, Vec<u64>>> = Lazy::new(|| DashMap::with_capacity(25_000));
static PATH_INDEX: Lazy<DashMap<String, Vec<u64>>> = Lazy::new(|| DashMap::with_capacity(25_000));
static FILE_COUNT: AtomicU64 = AtomicU64::new(0);
static NEXT_ID: AtomicU64 = AtomicU64::new(0);

pub struct FileSearcher {}

impl FileSearcher {
    pub fn new() -> Self {
        Self {}
    }

    pub fn clear_index(&self) {
        FILES.clear();
        NAME_INDEX.clear();
        PATH_INDEX.clear();
        FILE_COUNT.store(0, Ordering::Relaxed);
        NEXT_ID.store(0, Ordering::Relaxed);
        println!("🧹 Index vidé");
    }

    pub fn build_index<F>(&self, should_cancel: F)
    where F: Fn() -> bool + Send + Sync + 'static {
        self.clear_index();
        
        let start_time = Instant::now();
        println!("🔄 Démarrage de l'indexation...");

        let drives = get_drives();
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
                            let name = entry.file_name().to_string_lossy();
                            let path = entry.path().to_string_lossy();

                            FILES.insert(id, SearchResult {
                                id,
                                name: name.to_string(),
                                path: path.to_string(),
                                size: metadata.len(),
                                is_dir: metadata.is_dir(),
                                modified: metadata.modified().unwrap_or(SystemTime::now()),
                            });

                            let name_lower = name.to_lowercase();
                            for word in name_lower
                                .split(|c: char| !c.is_alphanumeric())
                                .filter(|s| !s.is_empty() && s.len() > 2) {
                                
                                NAME_INDEX.entry(word.to_string())
                                    .or_insert_with(|| Vec::with_capacity(50))
                                    .push(id);
                            }

                            if let Some(parent) = entry.path().parent() {
                                let last_segment = parent.file_name()
                                    .map(|s| s.to_string_lossy().to_lowercase());
                                
                                if let Some(segment) = last_segment {
                                    PATH_INDEX.entry(segment.to_string())
                                        .or_insert_with(|| Vec::with_capacity(50))
                                        .push(id);
                                }
                            }

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
        let query = query.to_lowercase();
        
        // Recherche dans l'index des noms (plus rapide)
        let name_matches: Vec<u64> = NAME_INDEX.iter()
            .filter(|entry| entry.key().contains(&query))
            .flat_map(|entry| entry.value().clone())
            .take(1000)
            .collect();

        // Si pas assez de résultats, chercher dans les chemins
        let path_matches: Vec<u64> = if name_matches.len() < 1000 {
            PATH_INDEX.iter()
                .filter(|entry| entry.key().contains(&query))
                .flat_map(|entry| entry.value().clone())
                .take(1000 - name_matches.len())
                .collect()
        } else {
            Vec::new()
        };

        // Combiner et convertir les résultats
        let mut results: Vec<SearchResult> = name_matches.into_iter()
            .chain(path_matches)
            .filter_map(|id| FILES.get(&id))
            .map(|entry| entry.value().clone())
            .collect();

        results.truncate(100);
        results
    }
}