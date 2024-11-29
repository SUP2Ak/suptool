/**
 * List à faire encore:
 * - Deplacer la fonction format_size dans un fichier utils
 * - Clean le code, enlever les println! etc...
 */

use crate::slint_generated::{MainWindow, AppLogic};
use crate::pages::features::FileSearcher;
use slint::{ComponentHandle, Weak};
use parking_lot::Mutex;
use chrono;
use std::time::{Duration, Instant, SystemTime};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size >= GB {
        format!("{:.1} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.1} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.1} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}

// fn init_searcher(window: &Weak<MainWindow>) -> Arc<FileSearcher> {
//     println!("=== Initialisation du moteur de recherche ===");
    
//     let searcher = Arc::new(FileSearcher::new());
//     let searcher_clone = searcher.clone();
//     let window_weak = window.clone();

//     thread::spawn(move || {
//         println!("=== Début de l'indexation des fichiers ===");
//         (*searcher_clone).build_index();
//         println!("=== Fin de l'indexation des fichiers ===");
        
//         slint::invoke_from_event_loop(move || {
//             if let Some(window) = window_weak.upgrade() {
//                 window.global::<AppLogic>().on_invoke_search_ready(|| {
//                     println!("=== Moteur de recherche prêt ===");
//                 });
//             }
//         }).unwrap();
//     });

//     searcher
// }

fn format_time(time: SystemTime) -> String {
    let datetime = chrono::DateTime::<chrono::Local>::from(time);
    datetime.format("%Y-%m-%d %H:%M").to_string()
}

pub fn init(window: &Weak<MainWindow>) {
    let window_weak = window.clone();
    let searcher = Arc::new(FileSearcher::new());
    let last_query = Arc::new(Mutex::new((String::new(), Instant::now())));
    let is_indexing = Arc::new(AtomicBool::new(false));
    
    if let Some(window) = window_weak.upgrade() {
        let searcher_clone = searcher.clone();
        let window_weak_clone = window.as_weak();
        let is_indexing_clone = is_indexing.clone();
        
        window.global::<AppLogic>().on_start_indexing(move || {
            let window_weak = window_weak_clone.clone();
            let searcher = searcher_clone.clone();
            let is_indexing = is_indexing_clone.clone();
            
            // Si déjà en cours d'indexation, on COUPE TOUT
            if is_indexing.load(Ordering::SeqCst) {
                println!("🛑 ARRÊT FORCÉ DE L'INDEXATION");
                is_indexing.store(false, Ordering::SeqCst);
                searcher.clear_index();
                if let Some(window) = window_weak.upgrade() {
                    window.set_is_indexing(false);
                }
                return;
            }
            
            // Nouvelle indexation
            is_indexing.store(true, Ordering::SeqCst);
            if let Some(window) = window_weak.upgrade() {
                window.set_is_indexing(true);
            }

            let is_indexing_thread = is_indexing.clone();
            thread::spawn(move || {
                searcher.build_index(move || {
                    if !is_indexing_thread.load(Ordering::SeqCst) {
                        println!("⚡ Interruption immédiate de l'indexation");
                        return true;
                    }
                    false
                });
                
                // Reset le flag à la fin quoi qu'il arrive
                is_indexing.store(false, Ordering::SeqCst);
                
                slint::invoke_from_event_loop(move || {
                    if let Some(window) = window_weak.upgrade() {
                        window.set_is_indexing(false);
                    }
                }).unwrap();
            });
        });

        let searcher_clone = searcher.clone();
        let last_query = last_query.clone();
        let window_weak = window.as_weak();
        
        window.global::<AppLogic>().on_everysup_changed(move |value| {
            let value_string = value.to_string();
            let now = Instant::now();
            
            {
                let (ref last_value, ref last_time) = *last_query.lock();
                if now.duration_since(*last_time) < Duration::from_millis(20) 
                   || value_string == *last_value {
                    return;
                }
            }
            
            *last_query.lock() = (value_string.clone(), now);
            
            if value_string.len() < 2 {
                if let Some(window) = window_weak.upgrade() {
                    window.set_everysup_files(slint::ModelRc::new(slint::VecModel::default()).into());
                }
                return;
            }

            let window_weak = window_weak.clone();
            let searcher = searcher_clone.clone();
            
            std::thread::spawn(move || {
                let results = searcher.search(&value_string);
                
                slint::invoke_from_event_loop(move || {
                    if let Some(window) = window_weak.upgrade() {
                        let model = std::rc::Rc::new(slint::VecModel::from(
                            results.into_iter().take(100).map(|result| {
                                // Conversion en modèle UI
                                let row = vec![
                                    result.name,
                                    result.path,
                                    format_size(result.size),
                                    if result.is_dir { "Folder".into() } else { "File".into() },
                                    format_time(result.modified),
                                ];
                                slint::ModelRc::new(slint::VecModel::from(
                                    row.into_iter().map(|s| {
                                        let mut item = slint::StandardListViewItem::default();
                                        item.text = s.into();
                                        item
                                    }).collect::<Vec<_>>()
                                ))
                            }).collect::<Vec<_>>()
                        ));
                        window.set_everysup_files(model.into());
                    }
                }).unwrap();
            });
        });
    }
}
