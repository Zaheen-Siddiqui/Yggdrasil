use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

pub fn start_hyprland_event_listener(sender: Sender<()>) {
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
}

fn read_hyprland_events(stream: UnixStream, sender: &Sender<()>) {
    let reader = BufReader::new(stream);

    for line in reader.lines().map_while(Result::ok) {
        let event = line.split(">>").next().unwrap_or("");

        match event {
            "workspace" | "workspacev2" | "focusedmon" | "focusedmonv2" | "createworkspace"
            | "createworkspacev2" | "destroyworkspace" | "destroyworkspacev2"
            | "renameworkspace" => {
                let _ = sender.send(());
            }

            _ => {}
        }
    }
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
