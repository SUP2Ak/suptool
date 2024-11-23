#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::pages;
use crate::slint_generated::MainWindow;
use std::error::Error;
use slint::ComponentHandle;
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOW;
use windows::core::{PCWSTR, HSTRING};

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
    pages::settings::init(&ui.as_weak());
    pages::everysup::init(&ui.as_weak());
    ui.run()?;
    Ok(())
}