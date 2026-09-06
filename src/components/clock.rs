use gtk4::Label;
use gtk4::glib;
use gtk4::prelude::*;

pub fn update_clock(label: &Label) {
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
