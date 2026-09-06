use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Clone, Deserialize)]
pub struct HyprWorkspace {
    pub id: i32,
    name: String,
    #[serde(default)]
    windows: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HyprActiveWorkspace {
    pub id: i32,
    name: String,
}

pub fn get_workspaces() -> Vec<HyprWorkspace> {
    let output = Command::new("hyprctl").args(["-j", "workspaces"]).output();

    let Ok(output) = output else {
        return Vec::new();
    };

    serde_json::from_slice(&output.stdout).unwrap_or_default()
}

pub fn get_active_workspace() -> Option<HyprActiveWorkspace> {
    let output = Command::new("hyprctl")
        .args(["-j", "activeworkspace"])
        .output()
        .ok()?;

    serde_json::from_slice(&output.stdout).ok()
}

pub fn switch_workspace(id: i32) {
    let _ = Command::new("hyprctl")
        .args(["dispatch", "workspace", &id.to_string()])
        .spawn();
}
