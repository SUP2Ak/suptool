use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;
use slint::Weak;
use crate::slint_generated::MainWindow;
use crate::widgets::show_notification;
use ureq;
use semver::Version;
use std::sync::mpsc;
use slint::TimerMode;
use std::os::windows::process::CommandExt;

#[derive(Debug)]
pub enum UpdateError {
    NoRelease,
    NetworkError(String),
    ParseError(String),
    DownloadError(String),
}

impl std::fmt::Display for UpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoRelease => write!(f, "Aucune release trouvée"),
            Self::NetworkError(e) => write!(f, "Erreur réseau: {}", e),
            Self::ParseError(e) => write!(f, "Erreur de parsing: {}", e),
            Self::DownloadError(e) => write!(f, "Erreur de téléchargement: {}", e),
        }
    }
}

impl UpdateError {
    // Helper pour obtenir le type de notification et le titre selon l'erreur
    fn notification_details(&self) -> (&'static str, &'static str) {
        match self {
            Self::NoRelease => ("warning", "Aucune release"),
            Self::NetworkError(_) => ("error", "Erreur réseau"),
            Self::ParseError(_) => ("error", "Erreur de parsing"),
            Self::DownloadError(_) => ("error", "Erreur de téléchargement"),
        }
    }
}

impl std::error::Error for UpdateError {}

#[derive(Debug, Deserialize, Serialize)]
struct GithubRelease {
    tag_name: String,
    assets: Vec<GithubAsset>,
    body: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct AppMetadata {
    version: String,
    last_check: u64,
}

pub struct Updater {
    //metadata: Arc<RwLock<AppMetadata>>,
    version: String,
    window: Weak<MainWindow>,
    update_available: Arc<RwLock<Option<GithubRelease>>>,
}

impl Updater {
    pub fn new(window: &Weak<MainWindow>) -> Self {
        //let metadata = Self::load_or_create_metadata();
        let version = env!("CARGO_PKG_VERSION").to_string();
        
        Self {
            //metadata: Arc::new(RwLock::new(metadata)),
            version: version,
            window: window.clone(),
            update_available: Arc::new(RwLock::new(None)),
        }
    }

    // fn get_metadata_path() -> PathBuf {
    //     let local_app_data = dirs::data_local_dir()
    //         .unwrap_or_else(|| PathBuf::from("./"));
    //     local_app_data.join("suptool").join("metadata.json")
    // }

    // fn load_or_create_metadata() -> AppMetadata {
    //     let path = Self::get_metadata_path();
        
    //     if let Ok(content) = std::fs::read_to_string(&path) {
    //         if let Ok(metadata) = serde_json::from_str(&content) {
    //             return metadata;
    //         }
    //     }

    //     // Créer des métadonnées par défaut si le fichier n'existe pas
    //     let metadata = AppMetadata {
    //         version: env!("CARGO_PKG_VERSION").to_string(), // Version par défaut
    //         last_check: 0,
    //     };

    //     // Créer le dossier si nécessaire
    //     if let Some(parent) = path.parent() {
    //         let _ = std::fs::create_dir_all(parent);
    //     }

    //     // Sauvegarder les métadonnées
    //     let _ = std::fs::write(
    //         &path,
    //         serde_json::to_string_pretty(&metadata).unwrap_or_default(),
    //     );

    //     metadata
    // }

    pub fn check_for_updates(&self) -> Result<bool, UpdateError> {
        // Faire la requête avec gestion d'erreur
        let response = match ureq::get("https://api.github.com/repos/SUP2Ak/suptool/releases/latest")
            .set("User-Agent", "suptool")
            .call()
        {
            Ok(response) => response,
            Err(e) => {
                let error = match e {
                    ureq::Error::Status(404, _) => UpdateError::NoRelease,
                    ureq::Error::Status(code, response) => {
                        UpdateError::NetworkError(
                            format!("Status {}: {}", code, response.status_text())
                        )
                    },
                    e => UpdateError::NetworkError(e.to_string()),
                };
                
                if let Some(_) = self.window.upgrade() {
                    let (notification_type, title) = error.notification_details();
                    show_notification(
                        &self.window,
                        "update-error",
                        title,
                        &error.to_string(),
                        notification_type
                    );
                }
                return Err(error);
            }
        };

        // Parser le JSON avec gestion d'erreur
        let release: GithubRelease = match response.into_json() {
            Ok(release) => release,
            Err(e) => {
                let error = UpdateError::ParseError(e.to_string());
                if let Some(_) = self.window.upgrade() {
                    let (notification_type, title) = error.notification_details();
                    show_notification(
                        &self.window,
                        "parse-error",
                        title,
                        &error.to_string(),
                        notification_type
                    );
                }
                return Err(error);
            }
        };
            
        let latest_version = match Version::parse(release.tag_name.trim_start_matches('v')) {
            Ok(version) => version,
            Err(e) => {
                let error = UpdateError::ParseError(e.to_string());
                if let Some(_) = self.window.upgrade() {
                    let (notification_type, title) = error.notification_details();
                    show_notification(
                        &self.window,
                        "version-error",
                        title,
                        &error.to_string(),
                        notification_type
                    );
                }
                return Err(error);
            }
        };
        
        let current_version = match Version::parse(&self.version) {
            Ok(version) => version,
            Err(e) => {
                let error = UpdateError::ParseError(e.to_string());
                if let Some(_) = self.window.upgrade() {
                    let (notification_type, title) = error.notification_details();
                    show_notification(
                        &self.window,
                        "current-version-error",
                        title,
                        &error.to_string(),
                        notification_type
                    );
                }
                return Err(error);
            }
        };

        if latest_version > current_version {
            *self.update_available.write() = Some(release);
            
            if let Some(_) = self.window.upgrade() {
                show_notification(
                    &self.window,
                    "update-available",
                    "Mise à jour disponible",
                    &format!("Version {} disponible, votre version actuelle est : {}", 
                        latest_version, current_version),
                    "info"
                );
            }
            return Ok(true);
        } else {
            if let Some(_) = self.window.upgrade() {
                show_notification(
                    &self.window,
                    "no-update",
                    "Aucune mise à jour disponible",
                    &format!("Vous avez la dernière version disponible : {}", current_version),
                    "info"
                );
            }
        }

        Ok(false)
    }

    pub fn download_and_install_update(&self) -> Result<(), UpdateError> {
        let (progress_tx, progress_rx) = mpsc::channel::<f32>();
        let window_weak = self.window.clone();
        
        // Activer l'état de téléchargement
        if let Some(window) = self.window.upgrade() {
            window.invoke_set_is_downloading(true);
            window.invoke_set_download_progress(0.0);
        }

        // Timer pour mettre à jour la progression
        slint::Timer::default().start(
            TimerMode::Repeated,
            std::time::Duration::from_millis(100),
            move || {
                if let Some(window) = window_weak.upgrade() {
                    if let Ok(progress) = progress_rx.try_recv() {
                        window.invoke_set_download_progress(progress);
                    }
                }
            },
        );

        // Clone pour le thread
        let url = if let Some(release) = self.update_available.read().as_ref() {
            if let Some(asset) = release.assets.iter().find(|a| a.name.ends_with(".exe")) {
                asset.browser_download_url.clone()
            } else {
                return Err(UpdateError::DownloadError("Aucun installateur trouvé".into()));
            }
        } else {
            return Err(UpdateError::DownloadError("Aucune mise à jour disponible".into()));
        };

        let progress_tx = progress_tx.clone();
        
        std::thread::spawn(move || {
            println!("Début du téléchargement...");
            
            // Télécharger l'installateur
            let response = match ureq::get(&url).call() {
                Ok(response) => response,
                Err(e) => {
                    println!("Erreur de téléchargement: {}", e);
                    return;
                }
            };

            let total_size = response.header("content-length")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);

            let mut reader = response.into_reader();
            let mut bytes = Vec::new();
            let mut buffer = [0; 8192];
            let mut downloaded = 0;

            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        bytes.extend_from_slice(&buffer[..n]);
                        downloaded += n;
                        
                        if total_size > 0 {
                            let progress = downloaded as f32 / total_size as f32;
                            let _ = progress_tx.send(progress);
                            println!("Progression : {}%", (progress * 100.0) as i32);
                        }
                    }
                    Err(e) => {
                        println!("Erreur de lecture: {}", e);
                        return;
                    }
                }
            }

            // Sauvegarder et lancer l'installateur
            let temp_path = std::env::temp_dir().join("suptool_update.exe");
            if let Err(e) = std::fs::write(&temp_path, bytes) {
                println!("Erreur d'écriture: {}", e);
                return;
            }

            println!("Installation en cours...");

            // Créer le batch d'installation
            let install_cmd = format!(
                "start /wait /min \"\" \"{}\" /VERYSILENT /SUPPRESSMSGBOXES /NORESTART",
                temp_path.display()
            );

            let batch_path = std::env::temp_dir().join("update_suptool.bat");
            let batch_content = format!(
                "@echo off\n\
                >nul 2>&1 timeout /t 2 /nobreak\n\
                >nul 2>&1 taskkill /F /IM suptool.exe\n\
                {}\n\
                >nul 2>&1 start /min \"\" \"{}\"\n\
                >nul 2>&1 del \"%~f0\"\n\
                exit",
                install_cmd,
                std::env::current_exe()
                    .unwrap_or_default()
                    .display()
            );

            println!("Chemin du fichier batch : {}", batch_path.display());

            if let Err(e) = std::fs::write(&batch_path, batch_content) {
                println!("Erreur de création du batch: {}", e);
                return;
            }

            if let Err(e) = std::process::Command::new("cmd")
                .args(&["/C", &batch_path.to_string_lossy()])
                .creation_flags(0x08000000)
                .spawn()
            {
                println!("Erreur de lancement: {}", e);
                return;
            }

            std::thread::sleep(std::time::Duration::from_secs(2));
            std::process::exit(0);
        });

        Ok(())
    }
    // pub fn update_installed_version(&self, new_version: String) {
    //     let mut metadata = self.metadata.write();
    //     metadata.version = new_version;
        
    //     // Sauvegarder les métadonnées
    //     let path = Self::get_metadata_path();
    //     let _ = std::fs::write(
    //         &path,
    //         serde_json::to_string_pretty(&*metadata).unwrap_or_default(),
    //     );
    // }
}