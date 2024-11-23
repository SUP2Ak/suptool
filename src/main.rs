use std::env;
use std::io::Write;

mod app;
mod pages;
mod widgets;
mod slint_generated;
mod everything;

fn main() {
    #[cfg(debug_assertions)]
    env::set_var("RUST_BACKTRACE", "1");

    #[cfg(all(windows, debug_assertions))]
    {
        use windows::Win32::System::Console::{AllocConsole, GetConsoleWindow};
        use windows::Win32::Foundation::HWND;
        
        unsafe {
            if GetConsoleWindow() == HWND(0) {
                AllocConsole().expect("Impossible de créer la console");
            }
        }
    }

    if let Err(e) = app::run() {
        eprintln!("Erreur lors du lancement de l'application: {}", e);
        print!("Appuyez sur Entrée pour continuer...");
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut String::new()).unwrap();
    }
}