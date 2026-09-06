use gtk4::CssProvider;

pub fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_path("style.css");

    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("Could not connect to a display"),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
