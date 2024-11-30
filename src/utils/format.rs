use slint::SharedString;
use chrono::DateTime;
use std::time::SystemTime;

pub fn format_size(size: u64) -> SharedString {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let size = size as f64;
    let formatted = if size >= GB {
        format!("{:.2} GB", size / GB)
    } else if size >= MB {
        format!("{:.2} MB", size / MB)
    } else if size >= KB {
        format!("{:.2} KB", size / KB)
    } else {
        format!("{} B", size)
    };

    SharedString::from(formatted)
}

pub fn format_time(time: SystemTime) -> String {
    let datetime = DateTime::<chrono::Local>::from(time);
    datetime.format("%Y-%m-%d %H:%M").to_string()
}