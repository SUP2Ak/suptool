mod file_search; // file_search.rs is like import local module

use file_search::FileSearcher;
use std::sync::Arc;
use tauri::State;
use std::sync::Mutex;
use tauri::Manager;

struct SearchState(Arc<Mutex<FileSearcher>>);

#[tauri::command]
async fn get_all_index_entry(state: State<'_, SearchState>) -> Result<Vec<file_search::FileIndex>, String> {
    let searcher = state.0.lock().unwrap();
    Ok(searcher.get_all_indexed_files())
}

#[tauri::command]
async fn is_indexing_complete(state: State<'_, SearchState>) -> Result<bool, String> {
    let searcher = state.0.lock().unwrap();
    Ok(!searcher.is_indexing())
}

#[tauri::command]
async fn start_indexing(state: State<'_, SearchState>) -> Result<(), String> {
    let searcher = state.0.lock().unwrap();
    searcher.build_index();
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let searcher = Arc::new(Mutex::new(FileSearcher::new(None)));
    let searcher_clone= searcher.clone();

    // Create the app
    // move is like clone but for variables with "|app|" wtf is this shitty syntax? x)
    tauri::Builder::default()
        .manage(SearchState(searcher))
        .setup(move |app| {
            let webview = app.get_webview_window("main").unwrap();
            // webview.eval("console.log('hello from Rust')")?;
            println!("webview is ready: {}", webview.label());
            let mut fs = searcher_clone.lock().unwrap();
            *fs = FileSearcher::new(Some(webview));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_all_index_entry,
            start_indexing,
            is_indexing_complete
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
