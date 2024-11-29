/**
 * List à faire encore:
 * - Clean le code, enlever les println! etc...
 * - Sans doute changer cette sections ailleurs car la c'est l'updatateur
 */

use slint::{ComponentHandle, Weak};
use crate::slint_generated::{MainWindow, AppLogic};
use crate::updater::Updater;
use std::sync::Arc;

pub fn init(window: &Weak<MainWindow>, updater: Arc<Updater>) {
    let window_weak = window.clone();

    if let Some(window) = window_weak.upgrade() {
        window.set_current_version(env!("CARGO_PKG_VERSION").into());
        
        let updater_clone = Arc::clone(&updater);
        let window_weak = window.as_weak();
        window.global::<AppLogic>().on_check_for_updates(move || {
            match updater_clone.check_for_updates() {
                Ok(true) => {
                    if let Some(window) = window_weak.upgrade() {
                        window.set_update_available(true);
                    }
                }
                Ok(false) => {
                    if let Some(window) = window_weak.upgrade() {
                        window.set_update_available(false);
                    }
                }
                Err(e) => {
                    eprintln!("Erreur lors de la vérification des mises à jour: {}", e);
                    if let Some(window) = window_weak.upgrade() {
                        window.set_update_available(false);
                    }
                }
            }
        });

        let updater_clone = Arc::clone(&updater);
        window.global::<AppLogic>().on_install_update(move || {
            if let Err(e) = updater_clone.download_and_install_update() {
                eprintln!("Erreur lors de l'installation de la mise à jour: {}", e);
            }
        });
    }
}