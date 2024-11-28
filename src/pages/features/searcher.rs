use std::sync::Arc;
use parking_lot::RwLock;
use crate::everything::{EverythingIndex, search::{SearchEngine, SearchResult}};
use crate::everything::index::block::SearchBlock;
// use crate::pages::everysup::MAX_DISPLAY_RESULTS;
use std::path::Path;
use std::io;
// use rayon::prelude::*;
use crate::everything::index::entry::FileEntry;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
// use std::sync::mpsc::{channel, Sender};
use std::thread;
// use std::cmp::Ordering as CmpOrdering;
use crossbeam::channel::bounded;
// use num_cpus;
// use std::time::SystemTime;

#[derive(Clone)]
pub struct FileSearcher {
    index: Arc<RwLock<EverythingIndex>>,
    engine: Arc<SearchEngine>,
}

impl FileSearcher {
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            index: Arc::new(RwLock::new(EverythingIndex::new())),
            engine: Arc::new(SearchEngine::new()),
        })
    }

    pub fn build_index(&self) {
        println!("=== Construction de l'index ===");
        let paths = Self::get_system_paths();
        let index = self.index.clone();
        let start_time = std::time::Instant::now();
        let total_files = Arc::new(AtomicUsize::new(0));

        // Utiliser un HashSet pour suivre les chemins uniques pendant l'indexation
        let seen_paths = Arc::new(parking_lot::Mutex::new(std::collections::HashSet::new()));
        
        let (tx, rx) = bounded::<(FileEntry, PathBuf)>(1_000_000);
        let (block_tx, block_rx) = bounded::<(SearchBlock, Vec<PathBuf>)>(100_000);
        let block_tx = Arc::new(block_tx);

        // Scanner les fichiers en parallèle
        let scan_threads: Vec<_> = paths.into_iter().map(|path| {
            let tx = tx.clone();
            let total = total_files.clone();
            let seen = seen_paths.clone();
            thread::spawn(move || {
                scan_directory_recursive(&path, &tx, &total, &seen);
            })
        }).collect();

        // Construire les blocs
        let block_builder = thread::spawn({
            let rx = rx;
            let block_tx = block_tx.clone();
            move || {
                let mut current_block = SearchBlock::new();
                while let Ok((entry, path)) = rx.recv() {
                    let _ = path.file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default();
                        
                    if current_block.entries.len() >= 10_000 {  // Taille de bloc fixe
                        let paths = std::mem::take(&mut current_block.paths);
                        let block = std::mem::replace(&mut current_block, SearchBlock::new());
                        let _ = block_tx.send((block, paths));
                        current_block.add_entry(entry, &path);
                        current_block.paths.push(path);
                    } else {
                        current_block.add_entry(entry, &path);
                        current_block.paths.push(path);
                    }
                }
                
                if !current_block.entries.is_empty() {
                    let paths = std::mem::take(&mut current_block.paths);
                    let _ = block_tx.send((current_block, paths));
                }
            }
        });

        // Construire l'index final
        let index_builder = thread::spawn({
            let index = index.clone();
            move || {
                let mut index_guard = index.write();
                while let Ok((mut block, paths)) = block_rx.recv() {
                    for (i, path) in paths.iter().enumerate() {
                        let path_id = index_guard.path_table.add_path(path);
                        if let Some(entry) = block.entries.get_mut(i) {
                            entry.set_path_id(path_id);
                        }
                    }
                    index_guard.blocks.push(block);
                }
            }
        });

        // Attendre la fin
        for handle in scan_threads {
            handle.join().unwrap();
        }
        drop(tx);
        block_builder.join().unwrap();
        drop(block_tx);
        index_builder.join().unwrap();

        // Debug: afficher l'état final
        {
            let index = self.index.read();
            println!("\n=== État de l'index ===");
            println!("Nombre total de blocs: {}", index.blocks.len());
            
            if let Some(first_block) = index.blocks.first() {
                println!("\nPremier bloc:");
                println!("Nombre d'entrées: {}", first_block.entries.len());
                
                if let Some(first_entry) = first_block.entries.first() {
                    if let Some(path) = index.path_table.get_path(first_entry.path_id) {
                        println!("\n=== Premier fichier indexé ===");
                        println!("Chemin complet: {}", path.display());
                        println!("Nom du fichier: {}", path.file_name().unwrap_or_default().to_string_lossy());
                        println!("Taille: {} bytes", first_entry.decompress_size());
                        println!("Est un dossier: {}", first_entry.is_dir());
                        println!("Date de modification: {:?}", first_entry.decompress_date());
                        println!("Hash du nom: {:#x}", first_entry.name_hash);
                        println!("ID dans la table des chemins: {}", first_entry.path_id);
                        println!("=== Fin du premier fichier ===\n");
                    }
                }
            }
        }

        let final_total = total_files.load(Ordering::Relaxed);
        let elapsed = start_time.elapsed().as_secs_f64();
        println!("Total indexé : {} fichiers en {:.1}s ({:.0} fichiers/sec)", 
            final_total, elapsed, final_total as f64 / elapsed);
    }

    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        let index = self.index.read();
        self.engine.search(&index, query)
    }

    fn get_system_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();
        
        for drive in 'A'..='Z' {
            let path = PathBuf::from(format!("{}:\\", drive));
            if path.exists() {
                println!("=== Disque trouvé : {} ===", path.display());
                paths.push(path);
            }
        }

        println!("=== {} disques trouvés au total ===", paths.len());
        paths
    }
}

fn scan_directory_recursive(
    dir: &Path, 
    tx: &crossbeam::channel::Sender<(FileEntry, PathBuf)>,
    total: &Arc<AtomicUsize>,
    seen_paths: &Arc<parking_lot::Mutex<std::collections::HashSet<String>>>
) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if let Ok(metadata) = entry.metadata() {
                let file_entry = FileEntry::from_metadata(&metadata, &path);
                let _ = tx.send((file_entry, path.clone()));
                total.fetch_add(1, Ordering::Relaxed);
                
                if metadata.is_dir() {
                    scan_directory_recursive(&path, tx, total, seen_paths);
                }
            }
        }
    }
}