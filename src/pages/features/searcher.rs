use std::time::SystemTime;
use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use rayon::prelude::*;
use std::time::Instant;
use std::path::Path;
use dashmap::DashMap;
use ignore::WalkBuilder;
use num_cpus;
use once_cell::sync::Lazy;
use std::collections::HashSet;

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

    pub fn build_index(&self) {
        // On lance l'indexation dans un thread séparé pour ne pas bloquer l'UI
        std::thread::spawn(|| {
            let start_time = Instant::now();
            println!("🔄 Démarrage de l'indexation...");

            let drives = Self::get_drives();
            println!("💾 Disques détectés: {:?}", drives);

            // Scan ultra rapide en parallèle
            drives.into_par_iter().for_each(|drive| {
                println!("📂 Indexation du disque: {}", drive);
                
                WalkBuilder::new(&drive)
                    .hidden(false)
                    .git_ignore(false)
                    .git_global(false)
                    .git_exclude(false)
                    .threads(num_cpus::get())
                    .build_parallel()
                    .run(|| {
                        // Buffer local pour chaque thread
                        let mut local_results = Vec::with_capacity(10_000);
                        let mut local_word_map = HashMap::with_capacity(1_000);
                        
                        Box::new(move |entry| {
                            let entry = match entry {
                                Ok(entry) => entry,
                                Err(_) => return ignore::WalkState::Continue,
                            };

                            if let Ok(metadata) = entry.metadata() {
                                let count = FILE_COUNT.fetch_add(1, Ordering::Relaxed);
                                if count % 100_000 == 0 {
                                    println!("⏳ {} fichiers trouvés...", count);
                                }

                                let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
                                let name = entry.file_name().to_string_lossy().into_owned();

                                // Indexer les mots localement
                                for word in name.to_lowercase()
                                    .split(|c: char| !c.is_alphanumeric())
                                    .filter(|w| !w.is_empty())
                                {
                                    local_word_map.entry(word.to_string())
                                        .or_insert_with(Vec::new)
                                        .push(id);
                                }

                                local_results.push((id, SearchResult {
                                    id,
                                    name,
                                    path: entry.path().to_string_lossy().into_owned(),
                                    size: metadata.len(),
                                    is_dir: metadata.is_dir(),
                                    modified: metadata.modified().unwrap_or(SystemTime::now()),
                                }));

                                // Fusionner périodiquement dans la mémoire globale
                                if local_results.len() >= 10_000 {
                                    for (id, result) in local_results.drain(..) {
                                        FILES.insert(id, result);
                                    }

                                    for (word, ids) in local_word_map.drain() {
                                        WORDS.entry(word)
                                            .and_modify(|e: &mut Vec<u64>| e.extend(&ids))
                                            .or_insert_with(|| ids);
                                    }
                                }
                            }
                            ignore::WalkState::Continue
                        })
                    });
            });

            let duration = start_time.elapsed();
            println!("✅ Indexation terminée!");
            println!("=== Statistiques d'indexation ===");
            println!("⏱️  Temps total: {:.2} secondes", duration.as_secs_f64());
            println!("📑 Nombre de fichiers indexés: {}", FILE_COUNT.load(Ordering::Relaxed));
            println!("📊 Moyenne: {:.2} fichiers/seconde", 
                FILE_COUNT.load(Ordering::Relaxed) as f64 / duration.as_secs_f64());
        });
    }

    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        // Recherche ultra rapide directement en mémoire
        let words: Vec<String> = query.to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        if words.is_empty() {
            return Vec::new();
        }

        // Récupérer les IDs des fichiers qui matchent
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

    // Fonction utilitaire pour obtenir la liste des disques
    fn get_drives() -> Vec<String> {
        #[cfg(windows)]
        {
            // Pour Windows, vérifier les lettres de lecteur A-Z
            (b'C'..=b'Z')  // Commencer à C: pour éviter A: et B: (lecteurs obsolètes)
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