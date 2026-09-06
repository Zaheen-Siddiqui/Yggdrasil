use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation};

pub fn create_branding() -> GtkBox {
    let container = GtkBox::new(Orientation::Horizontal, 0);
    container.add_css_class("branding");

    let label = Label::new(Some("Yggdrasil"));
    label.add_css_class("brand");

    container.append(&label);

    container
}
