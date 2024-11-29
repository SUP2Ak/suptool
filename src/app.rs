#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::pages;
use crate::slint_generated::MainWindow;
use crate::updater::Updater;
use std::error::Error;
use slint::ComponentHandle;
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOW;
use windows::core::HSTRING;

pub fn run() -> Result<(), Box<dyn Error>> {
    if cfg!(windows) && !is_elevated::is_elevated() {
        println!("=== Redémarrage en mode administrateur ===");
        let current_exe = std::env::current_exe()?;
        
        unsafe {
            let _ = ShellExecuteW(
                None,
                &HSTRING::from("runas"),
                &HSTRING::from(current_exe.to_str().unwrap()),
                None,
                None,
                SW_SHOW,
            );
        }
        std::process::exit(0);
    }
    
    let ui = MainWindow::new()?;
    let updater = Updater::new(&ui.as_weak());
    if let Some(window) = ui.as_weak().upgrade() {
        window.set_current_version(env!("CARGO_PKG_VERSION").into());
    }

    pages::settings::init(&ui.as_weak());
    pages::everysup::init(&ui.as_weak());
    pages::about::init(&ui.as_weak(), updater.into());
    pages::cleartool::init(&ui.as_weak());

    ui.run()?;
    Ok(())
}