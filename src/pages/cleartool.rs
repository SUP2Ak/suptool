/**
 * List à faire encore:
 * - Clean l'interface correctement, avec les icones bien
 * - Faire en sorte que les groupes soient sauvegardés dans un fichier json ou autre (groupes personnalisés)
 * - Faire en sorte que les applications détectées soient sauvegardés dans un fichier json ou autre (applications personnalisées et du coups quand l'application start verifie juste si elle existe toujours a ce path)
 * - Sur tout la partie personnalisation mieux faire les boites de dialogues, chemins etc...
 * - Clean le code, enlever les println! etc...
 * - Deplacer la fonction format_size dans un fichier utils
 * - Passer une grosse partie du code (logique pur back) dans un fichier backend comme pour le everysup
 */

use crate::slint_generated::{DetectedApp, MainWindow, CleanGroup, AppLogic};
use crate::utils::format_size;
use slint::{ComponentHandle, Weak, SharedString, ModelRc, VecModel};
use rfd::FileDialog;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::fs;
use std::thread;
use std::sync::Arc;
use std::path::PathBuf;

#[derive(Clone)]
struct CleaningGroup {
    name: String,
    enabled: bool,
    paths: Vec<PathBuf>,
    size: u64,
}

#[derive(Clone)]
struct DetectedAppInternal {
    name: String,
    paths: Vec<PathBuf>,
    size: u64,
    cleanable: bool,
}

#[derive(Clone)]
struct ClearTool {
    groups: Arc<Mutex<HashMap<String, CleaningGroup>>>,
    detected_apps: Arc<Mutex<HashMap<String, DetectedAppInternal>>>,
}

pub fn init(window: &Weak<MainWindow>) {
    let cleartool = Arc::new(ClearTool::new());
    
    if let Some(window) = window.upgrade() {
        let cleartool_clone = cleartool.clone();
        let window_weak = window.as_weak();
        
        thread::spawn(move || {
            cleartool_clone.detect_apps();
            cleartool_clone.refresh_sizes();
            
            slint::invoke_from_event_loop(move || {
                if let Some(window) = window_weak.upgrade() {
                    update_ui(&window.global::<AppLogic>(), &cleartool_clone);
                }
            }).unwrap();
        });

        let logic = window.global::<AppLogic>();
        logic.on_refresh_sizes({
            let cleartool = cleartool.clone();
            let window_weak = window.as_weak();
            move || {
                let cleartool = cleartool.clone();
                let window_weak = window_weak.clone();
                
                thread::spawn(move || {
                    cleartool.refresh_sizes();
                    
                    slint::invoke_from_event_loop(move || {
                        if let Some(window) = window_weak.upgrade() {
                            update_ui(&window.global::<AppLogic>(), &cleartool);
                        }
                    }).unwrap();
                });
            }
        });

        let cleartool_clone = cleartool.clone();
        logic.on_start_cleaning({
            let window_weak = window.as_weak();
            move || {
                let cleartool = cleartool_clone.clone();
                let window_weak = window_weak.clone();
                
                thread::spawn(move || {
                    let cleaned_size = cleartool.clean_selected();
                    println!("🧹 Nettoyage terminé : {} libérés", format_size(cleaned_size));
                    
                    cleartool.refresh_sizes();
                    
                    slint::invoke_from_event_loop(move || {
                        if let Some(window) = window_weak.upgrade() {
                            update_ui(&window.global::<AppLogic>(), &cleartool);
                        }
                    }).unwrap();
                });
            }
        });

        let cleartool_clone = cleartool.clone();
        logic.on_add_custom_group({
            let window_weak = window.as_weak();
            move |name| {
                let cleartool = cleartool_clone.clone();
                let window_weak = window_weak.clone();
                
                let mut groups = cleartool.groups.lock();
                groups.insert(name.to_string(), CleaningGroup {
                    name: name.to_string(),
                    enabled: true,
                    paths: Vec::new(),
                    size: 0,
                });
                drop(groups);
                
                slint::invoke_from_event_loop(move || {
                    if let Some(window) = window_weak.upgrade() {
                        update_ui(&window.global::<AppLogic>(), &cleartool);
                    }
                }).unwrap();
            }
        });

        let cleartool_clone = cleartool.clone();
        logic.on_toggle_group({
            let window_weak = window.as_weak();
            move |index, enabled| {
                let cleartool = cleartool_clone.clone();
                let window_weak = window_weak.clone();
                
                let mut groups = cleartool.groups.lock();
                if let Some(group) = groups.values_mut().nth(index as usize) {
                    group.enabled = enabled;
                }
                drop(groups);
                
                slint::invoke_from_event_loop(move || {
                    if let Some(window) = window_weak.upgrade() {
                        update_ui(&window.global::<AppLogic>(), &cleartool);
                    }
                }).unwrap();
            }
        });

        let cleartool_clone = cleartool.clone();
        logic.on_add_path_to_group({
            let window_weak = window.as_weak();
            move |group_index, path| {
                let cleartool = cleartool_clone.clone();
                let window_weak = window_weak.clone();
                
                let mut groups = cleartool.groups.lock();
                if let Some(group) = groups.values_mut().nth(group_index as usize) {
                    group.paths.push(PathBuf::from(path.as_str()));
                }
                drop(groups);
                
                slint::invoke_from_event_loop(move || {
                    if let Some(window) = window_weak.upgrade() {
                        update_ui(&window.global::<AppLogic>(), &cleartool);
                    }
                }).unwrap();
            }
        });

        window.global::<AppLogic>().on_browse_for_path({
            move || -> SharedString {
                if let Some(paths) = FileDialog::new()
                    .set_title("Sélectionner des dossiers")
                    .set_directory("/")
                    .pick_folders()
                {
                    let paths_str = paths.iter()
                        .map(|p| p.to_string_lossy().to_string())
                        .collect::<Vec<String>>()
                        .join(";");
                    SharedString::from(paths_str)
                } else {
                    SharedString::default()
                }
            }
        });

        // Dans la fonction init
        let cleartool_clone = cleartool.clone();
        logic.on_toggle_app({
            let window_weak = window.as_weak();
            move |index, enabled| {
                let mut apps = cleartool_clone.detected_apps.lock();
                
                // On récupère le nom de l'app à partir de l'index
                let app_name = apps.keys()
                    .nth(index as usize)
                    .map(|k| k.clone());
                
                if let Some(name) = app_name {
                    if let Some(app) = apps.get_mut(&name) {
                        app.cleanable = enabled;
                    }
                }
                drop(apps);
                
                if let Some(window) = window_weak.upgrade() {
                    update_ui(&window.global::<AppLogic>(), &cleartool_clone);
                }
            }
        });
    }
}

impl ClearTool {
    fn new() -> Self {
        let mut groups = HashMap::new();
        
        // Groupe Windows Temp par défaut
        groups.insert("Windows Temp".to_string(), CleaningGroup {
            name: "Windows Temp".to_string(),
            enabled: true,
            paths: vec![
                PathBuf::from(std::env::var("windir").unwrap_or("C:\\Windows".to_string())).join("Temp"),
                PathBuf::from(std::env::var("TEMP").unwrap_or_else(|_| {
                    PathBuf::from(std::env::var("USERPROFILE").unwrap_or("C:\\Users\\Default".to_string()))
                        .join("AppData\\Local\\Temp")
                        .to_string_lossy()
                        .to_string()
                })),
            ],
            size: 0,
        });

        Self {
            groups: Arc::new(Mutex::new(groups)),
            detected_apps: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn calculate_dir_size(&self, path: &PathBuf) -> u64 {
        let mut total_size = 0;

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if let Ok(metadata) = fs::metadata(&path) {
                    if metadata.is_file() {
                        total_size += metadata.len();
                    } else if metadata.is_dir() {
                        total_size += self.calculate_dir_size(&path);
                    }
                }
            }
        }

        total_size
    }

    fn clean_directory(&self, path: &PathBuf) -> u64 {
        let mut cleaned_size = 0;

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if let Ok(metadata) = fs::metadata(&path) {
                    if metadata.is_file() {
                        cleaned_size += metadata.len();
                        let _ = fs::remove_file(&path);
                    } else if metadata.is_dir() {
                        cleaned_size += self.clean_directory(&path);
                        let _ = fs::remove_dir(&path);
                    }
                }
            }
        }

        cleaned_size
    }

    fn detect_apps(&self) {
        let mut apps = HashMap::new();
        
        if let Some(local_appdata) = std::env::var_os("LOCALAPPDATA") {
            let fivem_base = PathBuf::from(local_appdata).join("FiveM\\FiveM.app");
            let mut total_size = 0;
            let mut fivem_paths = Vec::new();
            
            let cache_paths = vec![
                (fivem_base.join("data\\cache"), "Cache"),
                (fivem_base.join("data\\server-cache"), "Server Cache"),
                (fivem_base.join("data\\server-cache-priv"), "Server Cache Priv"),
                (fivem_base.join("logs"), "Logs"),
                (fivem_base.join("crashes"), "Crashes"),
            ];

            for (path, _) in &cache_paths {
                if path.exists() {
                    let size = self.calculate_dir_size(path);
                    total_size += size;
                    fivem_paths.push(path.clone());
                }
            }

            if !fivem_paths.is_empty() {
                apps.insert("FiveM".to_string(), DetectedAppInternal {
                    name: "FiveM".to_string(),
                    paths: fivem_paths,
                    size: total_size,
                    cleanable: true,
                });
            }
        }

        *self.detected_apps.lock() = apps;
    }

    fn refresh_sizes(&self) {
        let mut groups = self.groups.lock();
        for group in groups.values_mut() {
            let mut total_size = 0;
            for path in &group.paths {
                if path.exists() {
                    total_size += self.calculate_dir_size(path);
                }
            }
            group.size = total_size;
        }
    }

    fn clean_selected(&self) -> u64 {
        let mut total_cleaned = 0;
        
        // Nettoyage des groupes
        let groups = self.groups.lock();
        for group in groups.values() {
            if !group.enabled {
                continue;
            }

            for path in &group.paths {
                if !path.exists() {
                    continue;
                }
                total_cleaned += self.clean_directory(path);
            }
        }
        drop(groups);

        let mut apps = self.detected_apps.lock();
        for app in apps.values_mut() {
            if app.cleanable {
                for path in &app.paths {
                    if path.exists() {
                        total_cleaned += self.clean_directory(path);
                    }
                }
                
                let mut new_size = 0;
                for path in &app.paths {
                    if path.exists() {
                        new_size += self.calculate_dir_size(path);
                    }
                }
                app.size = new_size;
            }
        }

        total_cleaned
    }
}

fn update_ui(logic: &AppLogic, cleartool: &ClearTool) {
    let groups = cleartool.groups.lock();
    let clean_groups: Vec<_> = groups.values()
        .map(|group| CleanGroup {
            name: SharedString::from(group.name.clone()),
            enabled: group.enabled,
            size: SharedString::from(format_size(group.size)),
            paths: ModelRc::new(VecModel::from(
                group.paths.iter()
                    .map(|p| SharedString::from(p.to_string_lossy().to_string()))
                    .collect::<Vec<_>>()
            )),
        })
        .collect();
    logic.set_cleaning_groups(ModelRc::new(VecModel::from(clean_groups)));

    let apps = cleartool.detected_apps.lock();
    let detected_apps: Vec<_> = apps.values()
        .map(|app| DetectedApp {
            name: SharedString::from(app.name.clone()),
            paths: ModelRc::new(VecModel::from(
                app.paths.iter()
                    .map(|p| SharedString::from(p.to_string_lossy().to_string()))
                    .collect::<Vec<_>>()
            )),
            size: SharedString::from(format_size(app.size)),
            cleanable: app.cleanable,
        })
        .collect();
    logic.set_detected_apps(ModelRc::new(VecModel::from(detected_apps)));
}