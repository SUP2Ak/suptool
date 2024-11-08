use std::path::Path;
use std::sync::Arc;
use walkdir::WalkDir;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use parking_lot::Mutex as ParkingMutex;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::WebviewWindow;
use tauri::Emitter;
use std::time::Instant;

/**
 * Represents a file details.
 */
#[derive(Serialize, Deserialize)]
struct FileDetails {
    path: String,
    extension: String,
    size: u64,
    last_modified: DateTime<Utc>,
    created: DateTime<Utc>,
}

/**
 * Represents the indexing status.
 */
#[derive(Clone, Serialize, Debug)]
struct IndexingStatus {
    total_files: usize,
    current_drive: String,
    is_complete: bool,
}

/**
 * Represents a file index.
 */
#[derive(Clone, Serialize)]
pub struct FileIndex {
    path: String,
    extension: String,
    drive: String,
    size: u64,
    last_modified: DateTime<Utc>,
    created: DateTime<Utc>,
}

/**
 * Represents a file searcher.
 */
#[derive(Clone)]
pub struct FileSearcher {
    index: Arc<ParkingMutex<Vec<FileIndex>>>,
    is_indexing: Arc<AtomicBool>,
    window: Option<Arc<WebviewWindow>>,
    start_time: Instant,
}

/**
 * Represents a file searcher.
 * Is like a class in other languages? but need a struct (pub as public AND no prefix as private)
 * And in is same in struct and impl, no prefix is private, pub is public
 */
impl FileSearcher {
    /**
     * Creates a new FileSearcher instance.
     * 
     * @param window: Option<WebviewWindow> - The optional window to emit status to.
     * @return FileSearcher - The new FileSearcher instance.
     */
    pub fn new(window: Option<WebviewWindow>) -> Self {
        let searcher: FileSearcher = FileSearcher {
            index: Arc::new(ParkingMutex::new(Vec::new())),
            is_indexing: Arc::new(AtomicBool::new(false)),
            window: window.map(Arc::new),
            start_time: Instant::now(),
        };

        // Wait for the window to be available before starting the indexing
        if searcher.window.is_some() {
            let searcher_clone: FileSearcher = searcher.clone();
            std::thread::spawn(move || {
                searcher_clone.build_index();
            });
        } else {
            println!("WARNING: FileSearcher initialized without a window?!");
            println!("Awaiting window to start indexing...");
        }

        searcher // In Rust, the last expression in a function is the return value wtf? x)
    }

    /**
     * Returns whether the indexing is in progress.
     * 
     * @return bool - Whether the indexing is in progress.
     */
    pub fn is_indexing(&self) -> bool {
        self.is_indexing.load(Ordering::SeqCst)
    }

    /**
     * Emits the status to the front end.
     * 
     * @param drive: &str - The current drive being indexed.
     * @param files_count: usize - The number of files indexed.
     * @param is_complete: bool - Whether the indexing is complete.
     */
    fn emit_status(&self, drive: &str, files_count: usize, is_complete: bool) {
        if files_count == 0 { return; }
        if let Some(window) = &self.window {
            let status: IndexingStatus = IndexingStatus {
                total_files: files_count,
                current_drive: drive.to_string(),
                is_complete,
            };
            
            println!("Emitting status to the front end: {:?}", &status);
            match window.emit("indexing-status", &status) {
                Ok(_) => println!("Status emitted successfully"),
                Err(e) => println!("Error emitting status: {:?}", e),
            }
            let elapsed_time = self.start_time.elapsed();
            println!("Time to emit status: {:?}, {:?} s, {:?} ms", elapsed_time, elapsed_time.as_secs(), elapsed_time.as_millis());
            if elapsed_time.as_secs() > 60 {
                println!(
                    "Time to emit status: {:?} min and {:?} s",
                    elapsed_time.as_secs() / 60,
                    elapsed_time.as_secs() % 60
                );
            } else if elapsed_time.as_secs() >= 1 {
                println!("Time to emit status: {:?} s", elapsed_time.as_secs());
            } else if elapsed_time.as_millis() >= 1 {
                println!("Time to emit status: {:?} ms", elapsed_time.as_millis());
            } else {
                println!("Time to emit status: {:?} µs", elapsed_time.as_micros());
            }
        } else {
            println!("Not emitting status because no window is available?!");
        }
    }

    /**
     * Builds the file index.
     * 
     * TODO: Add a progress bar to the front end or refresh estimate time?
     * 
     */
    pub fn build_index(&self) {
        if self.is_indexing.load(Ordering::SeqCst) {
            // println!("Indexation déjà en cours");
            self.emit_status("", 0, false);
            return;
        }

        self.is_indexing.store(true, Ordering::SeqCst);
        // println!("Démarrage de l'indexation...");
        
        let drives: Vec<String> = self.get_all_drives().unwrap_or_default();
        // println!("Disques trouvés : {:?}", drives);
        
        let mut all_files: Vec<FileIndex> = Vec::new();
        
        for drive in &drives {
            // println!("Indexation du disque : {}", drive);
            self.emit_status(drive, all_files.len(), false);
            
            let drive_files: Vec<FileIndex> = WalkDir::new(drive)
                .follow_links(false)
                .into_iter()
                .filter_entry(|e| !self.is_system_directory(e.path()))
                .filter_map(|e| e.ok())
                .filter_map(|entry| self.index_entry(entry))
                .collect();

            all_files.extend(drive_files);
            self.emit_status(drive, all_files.len(), false);
        }

        // println!("Indexation terminée : {} fichiers trouvés", all_files.len());
        *self.index.lock() = all_files;
        self.is_indexing.store(false, Ordering::SeqCst);
        self.emit_status("", self.index.lock().len(), true);
    }

    /**
     * Checks if the path is a system directory.
     * 
     * TODO: Add more system directories to check?
     * 
     * @param path: &Path - The path to check.
     * @return bool - Whether the path is a system directory.
     */
    fn is_system_directory(&self, path: &Path) -> bool {
        let path_str: String = path.to_string_lossy().to_lowercase();
        path_str.contains("windows") || 
        path_str.contains("$recycle.bin") || 
        path_str.contains("system volume information") ||
        path_str.contains("program files") ||
        path_str.contains("appdata")
    }

    /**
     * Indexes an entry.
     * 
     * @param entry: walkdir::DirEntry - The entry to index.
     * @return Option<FileIndex> - The indexed file details.
     */
    fn index_entry(&self, entry: walkdir::DirEntry) -> Option<FileIndex> {
        // Check if the entry is a file and not a directory
        // TODO: Check if it's a directory and return the files inside as sub-entries maybe ?
        if !entry.path().is_file() {
            return None;
        }

        let path: &Path = entry.path();
        let metadata: &std::fs::Metadata = &entry.metadata().ok()?;
        let drive: char = path.to_string_lossy().chars().next().unwrap();
        //println!("metadata: {:?}", metadata);
        
        Some(FileIndex {
            path: path.to_string_lossy().into_owned(),
            extension: path.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_string(),
            size: metadata.len(),
            last_modified: DateTime::from(metadata.modified().ok()?),
            created: DateTime::from(metadata.created().ok()?),
            drive: drive.to_string(),
        })
    }

    /**
     * Returns all drives.
     * 
     * @return Result<Vec<String>, std::io::Error> - The list of drives.
     */
    fn get_all_drives(&self) -> Result<Vec<String>, std::io::Error> {
        #[cfg(windows)]
        {
            use windows::Win32::Storage::FileSystem::{GetLogicalDrives, GetDriveTypeW};
            
            let drives = unsafe { GetLogicalDrives() };
            let mut result = Vec::new();
            
            for i in 0..26 { // 26 letters in the alphabet that weirdly enough are used as drive letters :issou:
                if (drives & (1 << i)) != 0 {
                    let drive_letter = (b'A' + i as u8) as char;
                    let drive_path = format!("{}:\\", drive_letter);
                    let wide_path: Vec<u16> = drive_path.encode_utf16().chain(std::iter::once(0)).collect();
                    let ptr = windows::core::PCWSTR(wide_path.as_ptr());
                    
                    if unsafe { GetDriveTypeW(ptr) } == 3 {
                        result.push(drive_path);
                    }
                }
            }
            Ok(result)
        }

        // Handle non-Windows systems, return the root drive
        // But it's not used yet
        #[cfg(not(windows))]
        {
            Ok(vec!["/".to_string()])
        }
    }

    /**
     * Returns all indexed files.
     * 
     * @return Vec<FileIndex> - The list of indexed files.
     */
    pub fn get_all_indexed_files(&self) -> Vec<FileIndex> {
        // Wait for the indexing to be complete before returning the files
        // Maybe better option todo that.. but my first time with rust x)
        if self.is_indexing.load(Ordering::SeqCst) {
            return Vec::new();
        }
        let index = self.index.lock();
        println!("Récupération de {} fichiers indexés", index.len());
        index.iter().cloned().collect()
    }
}