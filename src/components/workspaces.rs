use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button};
use std::cell::RefCell;
use std::rc::Rc;

use crate::hyprland::ipc::{get_active_workspace, get_workspaces, switch_workspace};

#[derive(Clone)]
pub struct PanelState {
    pub workspace_box: GtkBox,
    pub active_workspace: Rc<RefCell<i32>>,
}

pub fn refresh_workspaces(state: &PanelState) {
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
