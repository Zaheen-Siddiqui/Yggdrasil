use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

const APP_ID: &str = "org.yggdrasil.Desktop";
const PANEL_HEIGHT: i32 = 34;

pub fn create_window(app: &Application) -> ApplicationWindow {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Yggdrasil")
        .default_height(PANEL_HEIGHT)
        .build();

    // Initialize the GTK window as a Wayland layer-shell surface.
    window.init_layer_shell();
    window.set_namespace(Some("yggdrasil-panel"));
    window.set_layer(Layer::Top);

    // Anchor the panel to the top edge and stretch it across the screen.
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);

    // Reserve the panel's height so maximized windows do not overlap it.
    window.set_exclusive_zone(PANEL_HEIGHT);

    window
}
