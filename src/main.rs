use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, CssProvider, Label, Orientation, glib,
};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

use serde::Deserialize;
use std::cell::RefCell;
use std::fs;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;
use std::thread;
use std::time::Duration;

const APP_ID: &str = "org.yggdrasil.Desktop";
const PANEL_HEIGHT: i32 = 34;

#[derive(Debug, Clone, Deserialize)]
struct HyprWorkspace {
    id: i32,
    name: String,
    #[serde(default)]
    windows: i32,
}

#[derive(Debug, Clone, Deserialize)]
struct HyprActiveWorkspace {
    id: i32,
    name: String,
}

#[derive(Clone)]
struct PanelState {
    workspace_box: GtkBox,
    active_workspace: Rc<RefCell<i32>>,
}

fn main() {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_panel);
    app.run();
}

fn build_panel(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Yggdrasil")
        .default_height(PANEL_HEIGHT)
        .build();

    /*
     * Convert the GTK window into a Wayland layer-shell surface.
     *
     * This makes Yggdrasil a real desktop panel instead of a normal
     * application window.
     */
    window.init_layer_shell();
    window.set_namespace(Some("yggdrasil-panel"));
    window.set_layer(Layer::Top);

    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);

    /*
     * Reserve the panel's height so maximized windows do not occupy
     * the panel's area.
     */
    window.set_exclusive_zone(PANEL_HEIGHT);

    let root = GtkBox::new(Orientation::Horizontal, 0);
    root.add_css_class("panel");

    let branding = create_branding();
    let workspace_box = GtkBox::new(Orientation::Horizontal, 4);
    workspace_box.add_css_class("workspaces");

    let clock = Label::new(None);
    clock.add_css_class("clock");

    let battery = Label::new(None);
    battery.add_css_class("battery");

    root.append(&branding);
    root.append(&workspace_box);

    /*
     * Spacer pushes the system information to the right.
     */
    let spacer = GtkBox::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);

    root.append(&spacer);
    root.append(&clock);
    root.append(&battery);

    window.set_child(Some(&root));

    load_css();

    let active_workspace = Rc::new(RefCell::new(0));

    let state = PanelState {
        workspace_box: workspace_box.clone(),
        active_workspace: active_workspace.clone(),
    };

    /*
     * Initial state.
     */
    refresh_workspaces(&state);
    update_clock(&clock);
    update_battery(&battery);

    /*
     * Clock and battery are intentionally low-frequency operations.
     * Workspace state uses Hyprland's event socket instead.
     */
    {
        let clock = clock.clone();

        glib::timeout_add_seconds_local(1, move || {
            update_clock(&clock);
            glib::ControlFlow::Continue
        });
    }

    {
        let battery = battery.clone();

        glib::timeout_add_seconds_local(10, move || {
            update_battery(&battery);
            glib::ControlFlow::Continue
        });
    }

    /*
     * Listen to Hyprland workspace events in a background thread.
     */
    start_hyprland_event_listener(state.workspace_box.clone(), state.active_workspace.clone());

    window.present();
}

fn create_branding() -> GtkBox {
    let container = GtkBox::new(Orientation::Horizontal, 0);
    container.add_css_class("branding");

    let label = Label::new(Some("Yggdrasil"));
    label.add_css_class("brand");

    container.append(&label);

    container
}

fn load_css() {
    let provider = CssProvider::new();

    provider.load_from_data(
        r#"
        * {
            font-family: Sans;
        }

        window {
            background: transparent;
        }

        .panel {
            min-height: 34px;
            padding-left: 12px;
            padding-right: 12px;
            background: rgba(18, 18, 24, 0.96);
            color: #eeeeee;
        }

        .branding {
            min-width: 110px;
        }

        .brand {
            font-weight: 700;
            font-size: 14px;
        }

        .workspaces {
            margin-left: 10px;
        }

        .workspace {
            min-width: 28px;
            min-height: 26px;
            padding: 0;
            border-radius: 5px;
            background: transparent;
            color: #b8b8c2;
        }

        .workspace:hover {
            background: rgba(255, 255, 255, 0.08);
        }

        .workspace.active {
            background: rgba(255, 255, 255, 0.16);
            color: #ffffff;
        }

        .clock {
            min-width: 58px;
            margin-right: 12px;
            font-size: 13px;
        }

        .battery {
            min-width: 55px;
            font-size: 13px;
        }
        "#,
    );

    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("No GDK display available"),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn update_clock(label: &Label) {
    let now = glib::DateTime::now_local();

    match now {
        Ok(now) => {
            let text = now
                .format("%H:%M")
                .map(|value| value.to_string())
                .unwrap_or_else(|_| "--:--".to_string());

            label.set_text(&text);
        }
        Err(_) => label.set_text("--:--"),
    }
}

fn update_battery(label: &Label) {
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

fn refresh_workspaces(state: &PanelState) {
    let workspaces = get_workspaces();
    let active = get_active_workspace();

    if let Some(active) = active {
        *state.active_workspace.borrow_mut() = active.id;
    }

    while let Some(child) = state.workspace_box.first_child() {
        state.workspace_box.remove(&child);
    }

    /*
     * Always expose the first few workspaces even when they are empty.
     * This keeps the initial panel similar to:
     *
     * Yggdrasil | 1 2 3 | ...
     */
    let mut ids = vec![1, 2, 3];

    for workspace in workspaces {
        if workspace.id > 0 && !ids.contains(&workspace.id) {
            ids.push(workspace.id);
        }
    }

    ids.sort_unstable();

    for id in ids {
        let workspace = Button::with_label(&id.to_string());

        workspace.add_css_class("workspace");

        if *state.active_workspace.borrow() == id {
            workspace.add_css_class("active");
        }

        let state_clone = state.clone();

        workspace.connect_clicked(move |_| {
            switch_workspace(id);
            *state_clone.active_workspace.borrow_mut() = id;
            refresh_workspaces(&state_clone);
        });

        state.workspace_box.append(&workspace);
    }
}

fn get_workspaces() -> Vec<HyprWorkspace> {
    let output = Command::new("hyprctl").args(["-j", "workspaces"]).output();

    let Ok(output) = output else {
        return Vec::new();
    };

    serde_json::from_slice(&output.stdout).unwrap_or_default()
}

fn get_active_workspace() -> Option<HyprActiveWorkspace> {
    let output = Command::new("hyprctl")
        .args(["-j", "activeworkspace"])
        .output()
        .ok()?;

    serde_json::from_slice(&output.stdout).ok()
}

fn switch_workspace(id: i32) {
    let _ = Command::new("hyprctl")
        .args(["dispatch", "workspace", &id.to_string()])
        .spawn();
}

fn start_hyprland_event_listener(workspace_box: GtkBox, active_workspace: Rc<RefCell<i32>>) {
    let (sender, receiver) = std::sync::mpsc::channel::<()>();

    /*
     * Background thread:
     *
     * This thread only communicates using plain Rust data.
     * No GTK objects cross the thread boundary.
     */
    thread::spawn(move || {
        let Some(socket_path) = hyprland_event_socket() else {
            return;
        };

        loop {
            match UnixStream::connect(&socket_path) {
                Ok(stream) => {
                    read_hyprland_events(stream, &sender);
                }

                Err(_) => {
                    thread::sleep(Duration::from_secs(2));
                }
            }
        }
    });

    /*
     * GTK main thread:
     *
     * Poll the channel periodically and update GTK widgets here.
     */
    glib::timeout_add_local(Duration::from_millis(100), move || {
        while receiver.try_recv().is_ok() {
            let state = PanelState {
                workspace_box: workspace_box.clone(),
                active_workspace: active_workspace.clone(),
            };

            refresh_workspaces(&state);
        }

        glib::ControlFlow::Continue
    });
}

fn hyprland_event_socket() -> Option<PathBuf> {
    let runtime_dir = std::env::var_os("XDG_RUNTIME_DIR")?;
    let signature = std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE")?;

    Some(
        PathBuf::from(runtime_dir)
            .join("hypr")
            .join(signature)
            .join(".socket2.sock"),
    )
}

fn read_hyprland_events(stream: UnixStream, sender: &std::sync::mpsc::Sender<()>) {
    use std::io::{BufRead, BufReader};

    let reader = BufReader::new(stream);

    for line in reader.lines().map_while(Result::ok) {
        /*
         * Hyprland events use formats such as:
         *
         * workspace>>2
         * workspacev2>>2,2
         * focusedmonv2>>DP-1,2
         */
        let event = line.split(">>").next().unwrap_or("");

        match event {
            "workspace" | "workspacev2" | "focusedmon" | "focusedmonv2" | "createworkspace"
            | "createworkspacev2" | "destroyworkspace" | "destroyworkspacev2"
            | "renameworkspace" => {
                /*
                 * Send only a simple event notification.
                 *
                 * No GTK objects are passed through the thread.
                 */
                let _ = sender.send(());
            }

            _ => {}
        }
    }
}
