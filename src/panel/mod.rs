pub mod layout;
pub mod window;

use gtk4::Application;
use gtk4::prelude::*;

use layout::build_layout;
use window::create_window;

pub fn build_panel(app: &Application) {
    let window = create_window(app);

    build_layout(&window);

    window.present();
}
