use std::time::Duration;

use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Box as GtkBox, Label, Orientation};

use crate::components::battery::update_battery;
use crate::components::branding::create_branding;
use crate::components::clock::update_clock;
use crate::components::workspaces::{PanelState, refresh_workspaces};
use crate::hyprland::events::start_hyprland_event_listener;
use crate::theme::load_css;

pub fn build_layout(window: &ApplicationWindow) {
    let root = GtkBox::new(Orientation::Horizontal, 0);
    root.add_css_class("panel");

    let branding = create_branding();

    let workspace_box = GtkBox::new(Orientation::Horizontal, 4);
    workspace_box.add_css_class("workspaces");

    let clock = Label::new(None);
    clock.add_css_class("clock");

    let battery = Label::new(None);
    battery.add_css_class("battery");

    let spacer = GtkBox::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);

    root.append(&branding);
    root.append(&workspace_box);
    root.append(&spacer);
    root.append(&clock);
    root.append(&battery);

    window.set_child(Some(&root));

    load_css();

    let active_workspace = std::rc::Rc::new(std::cell::RefCell::new(0));

    let state = PanelState {
        workspace_box: workspace_box.clone(),
        active_workspace: active_workspace.clone(),
    };

    refresh_workspaces(&state);
    update_clock(&clock);
    update_battery(&battery);

    {
        let clock = clock.clone();

        gtk4::glib::timeout_add_seconds_local(1, move || {
            update_clock(&clock);
            gtk4::glib::ControlFlow::Continue
        });
    }

    {
        let battery = battery.clone();

        gtk4::glib::timeout_add_seconds_local(10, move || {
            update_battery(&battery);
            gtk4::glib::ControlFlow::Continue
        });
    }

    let (sender, receiver) = std::sync::mpsc::channel::<()>();

    start_hyprland_event_listener(sender);

    gtk4::glib::timeout_add_local(Duration::from_millis(100), move || {
        while receiver.try_recv().is_ok() {
            let state = PanelState {
                workspace_box: workspace_box.clone(),
                active_workspace: active_workspace.clone(),
            };

            refresh_workspaces(&state);
        }

        gtk4::glib::ControlFlow::Continue
    });
}
