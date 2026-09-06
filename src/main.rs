use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow};

const APP_ID: &str = "com.yggdrasil.desktop";

fn main() {
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);

    app.run();
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Yggdrasil")
        .default_width(800)
        .default_height(500)
        .build();

    window.present();
}
