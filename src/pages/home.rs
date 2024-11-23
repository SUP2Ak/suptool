// use slint::Weak;
// use slint::ComponentHandle;
// use crate::slint_generated::{MainWindow, HomeLogic};

// pub fn init(window: &Weak<MainWindow>) {
//     let window_weak = window.clone();
    
//     // Exemple d'initialisation d'un gestionnaire d'événements
//     if let Some(window) = window_weak.upgrade() {
//         window.global::<HomeLogic>().on_home_button_clicked(move || {
//             println!("Bouton de la page d'accueil cliqué!");
//         });
//     }
// }