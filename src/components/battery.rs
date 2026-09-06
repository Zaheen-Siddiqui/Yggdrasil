use gtk4::Label;
use gtk4::prelude::*;
use std::fs;
use std::path::PathBuf;

pub fn update_battery(label: &Label) {
    let battery_path = find_battery();

    let Some(path) = battery_path else {
        label.set_text("🔋 N/A");
        return;
    };

    let capacity = fs::read_to_string(path.join("capacity"))
        .ok()
        .and_then(|value| value.trim().parse::<u8>().ok())
        .unwrap_or(0);

    let status = fs::read_to_string(path.join("status")).unwrap_or_else(|_| "Unknown".to_string());

    let icon = match status.trim() {
        "Charging" => "⚡",
        "Full" => "🔋",
        _ if capacity <= 20 => "🪫",
        _ => "🔋",
    };

    label.set_text(&format!("{icon} {capacity}%"));
}

fn find_battery() -> Option<PathBuf> {
    let entries = fs::read_dir("/sys/class/power_supply").ok()?;

    for entry in entries.flatten() {
        let path = entry.path();

        let type_path = path.join("type");

        if let Ok(power_type) = fs::read_to_string(type_path) {
            if power_type.trim() == "Battery" {
                return Some(path);
            }
        }
    }

    None
}
