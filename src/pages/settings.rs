use slint::{ComponentHandle, Weak};
use crate::slint_generated::{MainWindow, MainWindowLogic};
use crate::widgets::show_notification;

pub fn init(window: &Weak<MainWindow>) {
    let window_weak = window.clone();
    
    if let Some(window) = window_weak.upgrade() {
        window.global::<MainWindowLogic>().on_settings_changed(move |setting, id| {
            println!("Paramètre modifié: {}", setting);
            show_notification(
                &window_weak,
                &format!("setting_{}", id),
                "Paramètre modifié",
                &setting,
                "info"
            );
        });
    }
}