use crate::slint_generated::{MainWindow, NotificationData};
use slint::{ComponentHandle, Weak, Timer};
use std::collections::VecDeque;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;

static NOTIFICATION_QUEUE: Lazy<Mutex<VecDeque<NotificationData>>> = Lazy::new(|| Mutex::new(VecDeque::new()));
static CURRENT_TOKEN: AtomicU64 = AtomicU64::new(0);
static ACTIVE_TOKENS: Lazy<Mutex<HashMap<String, u64>>> = Lazy::new(|| Mutex::new(HashMap::new()));

const MAX_VISIBLE_NOTIFICATIONS: usize = 3;
//const MAX_QUEUE_SIZE: usize = 10;

fn print_queue_state(queue: &VecDeque<NotificationData>) {
    println!("Taille de la queue: {}", queue.len());
    for (i, notif) in queue.iter().enumerate() {
        println!("{}. ID: {}, Type: {}, Title: {}", 
            i + 1, 
            notif.id, 
            notif.notification_type, 
            notif.title
        );
    }
}

pub fn show_notification(
    window: &Weak<MainWindow>,
    id: &str,
    title: &str,
    message: &str,
    notification_type: &str
) {
    println!("\n=== TENTATIVE D'AFFICHAGE DE NOTIFICATION ===");
    println!("ID: {}, Type: {}", id, notification_type);

    if let Some(window) = window.upgrade() {
        // Générer un nouveau jeton pour cette notification
        let token = CURRENT_TOKEN.fetch_add(1, Ordering::SeqCst);
        
        // Enregistrer le jeton pour cette notification
        if let Ok(mut tokens) = ACTIVE_TOKENS.lock() {
            tokens.insert(id.to_string(), token);
        }

        let notification = NotificationData {
            id: id.into(),
            title: title.into(),
            message: message.into(),
            notification_type: notification_type.into(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i32,
        };

        if let Ok(mut queue) = NOTIFICATION_QUEUE.lock() {
            println!("\n=== ÉTAT INITIAL DE LA QUEUE ===");
            print_queue_state(&queue);
            
            // Supprimer si existe déjà
            queue.retain(|n| n.id != id);
            
            // Ajouter la nouvelle notification
            queue.push_front(notification);
            println!("\n=== APRÈS AJOUT ===");
            print_queue_state(&queue);
            
            update_visible_notifications(&window, &queue);
        }

        // Auto-hide avec vérification du jeton
        let window_weak = window.as_weak();
        let id_owned = id.to_string();
        let type_owned = notification_type.to_string();
        Timer::single_shot(Duration::from_secs(5), move || {
            println!("\n=== VÉRIFICATION AUTO-HIDE ===");
            println!("ID: {}, Type: {}", id_owned, type_owned);
            
            // Vérifier si le jeton est toujours valide
            if let Ok(tokens) = ACTIVE_TOKENS.lock() {
                if let Some(&current_token) = tokens.get(&id_owned) {
                    if current_token == token {
                        if let Some(_) = window_weak.upgrade() {
                            if let Ok(queue) = NOTIFICATION_QUEUE.lock() {
                                let exists = queue.iter().any(|n| n.id == id_owned);
                                println!("Notification existe encore: {}", exists);
                                if exists {
                                    drop(queue);
                                    drop(tokens);
                                    hide_notification(&window_weak, &id_owned);
                                }
                            }
                        }
                    } else {
                        println!("Auto-hide annulé : jeton périmé");
                    }
                }
            }
        });
    }
}

pub fn hide_notification(window: &Weak<MainWindow>, id: &str) {
    println!("\n=== TENTATIVE DE SUPPRESSION DE NOTIFICATION ===");
    println!("ID: {}", id);

    // Supprimer le jeton
    if let Ok(mut tokens) = ACTIVE_TOKENS.lock() {
        tokens.remove(id);
    }

    if let Some(window) = window.upgrade() {
        if let Ok(mut queue) = NOTIFICATION_QUEUE.lock() {
            if !queue.iter().any(|n| n.id == id) {
                return;
            }

            queue.retain(|n| n.id != id);
            println!("\n=== APRÈS SUPPRESSION ===");
            print_queue_state(&queue);
            
            update_visible_notifications(&window, &queue);
        }
    }
}

fn update_visible_notifications(window: &MainWindow, queue: &VecDeque<NotificationData>) {
    let visible_notifications: Vec<NotificationData> = queue
        .iter()
        .take(MAX_VISIBLE_NOTIFICATIONS)
        .cloned()
        .collect();

    window.set_notifications(visible_notifications.as_slice().into());
}