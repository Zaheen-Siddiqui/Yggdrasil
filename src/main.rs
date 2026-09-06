mod components;
mod hyprland;
mod panel;
mod theme;

use gtk4::Application;
use gtk4::prelude::*;

const APP_ID: &str = "org.yggdrasil.Desktop";

fn main() {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(panel::build_panel);

    app.run();
}
