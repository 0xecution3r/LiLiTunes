// src/sync/mod.rs
//
// This module wires the "Sync iPod" button and runs the sync helper.
// No UI text is hardcoded here; all user-facing words come from ui/text.rs.

use gtk::prelude::*;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::ui::{self, UiHandles};

pub fn wire_sync_button(ui: &UiHandles) {
    // Clone widgets we need inside callbacks
    let button = ui.sync_ipod_button.clone();
    let now_playing = ui.now_playing.clone();

    ui.sync_ipod_button.connect_clicked(move |_| {
        // Disable button while syncing
        button.set_sensitive(false);
        button.set_label(ui::text::SYNC_IPOD_WORKING_LABEL);

        // Channel for worker -> UI updates
        let (tx, rx) = mpsc::channel::<SyncMsg>();

        // Worker thread: run helper + stream output
        thread::spawn(move || {
            // NOTE: we’ll move these into settings later; for now they are constants in text.rs.
            let mountpoint = ui::text::SYNC_IFUSE_MOUNTPOINT;
            let media_path = ui::text::SYNC_IPOD_MEDIA_PATH;            
            let db_path = ui::text::SYNC_DEFAULT_DB_PATH;
            
            if let Err(e) = ensure_ipod_mounted(mountpoint) {
                let _ = tx.send(SyncMsg::Error(e));
                return;
            }           

            let mut child = match Command::new(ui::text::SYNC_HELPER_PATH)
                .arg(ui::text::SYNC_ARG_MOUNT)
                .arg(media_path) // <-- pass Media path to libgpod helper
                .arg(ui::text::SYNC_ARG_SQLITE)
                .arg(db_path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
            {
                Ok(c) => c,
                Err(e) => {
                    let _ = tx.send(SyncMsg::Error(format!("{}{}", ui::text::SYNC_ERR_START, e)));
                    return;
                }
            };

            // Forward stdout lines
            if let Some(stdout) = child.stdout.take() {
                use std::io::{BufRead, BufReader};
                for line in BufReader::new(stdout).lines().flatten() {
                    let _ = tx.send(SyncMsg::Progress(line));
                }
            }

            // Wait for exit
            match child.wait() {
                Ok(status) if status.success() => {
                    let _ = tx.send(SyncMsg::Done);
                }
                Ok(status) => {
                    let _ = tx.send(SyncMsg::Error(format!(
                        "{}{}",
                        ui::text::SYNC_ERR_EXIT,
                        status
                    )));
                }
                Err(e) => {
                    let _ = tx.send(SyncMsg::Error(format!("{}{}", ui::text::SYNC_ERR_WAIT, e)));
                }
            }
        });

        // UI polling loop: keep GTK responsive
        let button_ui = button.clone();
        let now_ui = now_playing.clone();

        glib::timeout_add_local(Duration::from_millis(50), move || {
            for _ in 0..50 {
                match rx.try_recv() {
                    Ok(SyncMsg::Progress(line)) => {
                        // Show progress in the UI (we can improve formatting later)
                        now_ui.set_label(&line);
                    }
                    Ok(SyncMsg::Done) => {
                        now_ui.set_label(ui::text::SYNC_DONE_LABEL);
                        button_ui.set_label(ui::text::SYNC_IPOD_DEFAULT_LABEL);
                        button_ui.set_sensitive(true);
                        return glib::ControlFlow::Break;
                    }
                    Ok(SyncMsg::Error(err)) => {
                        now_ui.set_label(&err);
                        button_ui.set_label(ui::text::SYNC_IPOD_DEFAULT_LABEL);
                        button_ui.set_sensitive(true);
                        return glib::ControlFlow::Break;
                    }
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        now_ui.set_label(ui::text::SYNC_ERR_DISCONNECTED_LABEL);
                        button_ui.set_label(ui::text::SYNC_IPOD_DEFAULT_LABEL);
                        button_ui.set_sensitive(true);
                        return glib::ControlFlow::Break;
                    }
                }
            }

            glib::ControlFlow::Continue
        });
    });
}

//let ui_for_dialog = ui.clone(); // if UiHandles isn't Clone, then pass needed window handle instead
//let window = ui.window.clone();

pub fn start_device_watch(ui: &UiHandles) {
    let sync_btn = ui.sync_ipod_button.clone();
    let now_lbl = ui.now_playing.clone();
    let window = ui.window.clone();

    // Show the "fix usbmuxd" dialog only once per run
    let dialog_shown = Rc::new(Cell::new(false));
    let dialog_shown_tick = dialog_shown.clone();

    // Start disabled until we see the device
    sync_btn.set_sensitive(false);

    glib::timeout_add_local(std::time::Duration::from_millis(1000), move || {
        let usb_present = usb_present_lsusb();
        let legacy_visible = if usb_present {
            legacy_device_visible()
        } else {
            false
        };

        if legacy_visible {
            sync_btn.set_sensitive(true);
            now_lbl.set_label(crate::ui::text::IPOD_STATUS_CONNECTED);
        } else {
            sync_btn.set_sensitive(false);

            if usb_present {
                now_lbl.set_label(crate::ui::text::IPOD_STATUS_DISCONNECTED);

                // If USB is there but idevice tools can't see it, show instructions once
                if !dialog_shown_tick.get() {
                    dialog_shown_tick.set(true);
                    crate::ui::dialogs::show_usbmuxd_fix_dialog(&window);
                }
            } else {
                now_lbl.set_label(crate::ui::text::IPOD_STATUS_DISCONNECTED);
            }
        }

        glib::ControlFlow::Continue
    });
}

fn ipod_present() -> bool {
    // Run: lsusb
    let out = Command::new("lsusb").output();
    let Ok(out) = out else {
        return false;
    };
    if !out.status.success() {
        return false;
    }

    let s = String::from_utf8_lossy(&out.stdout);
    s.to_lowercase().contains(crate::ui::text::IPOD_USB_ID)
}

fn legacy_prefix_from_repo() -> Option<PathBuf> {
    // When running `cargo run`, current_dir is usually the repo root.
    let cwd = std::env::current_dir().ok()?;
    let p = cwd.join("src").join("deps").join("ios-legacy");
    if p.exists() { Some(p) } else { None }
}

fn legacy_cmd(program_name: &str) -> Option<Command> {
    let prefix = legacy_prefix_from_repo()?;

    let bin = prefix.join("bin");
    let lib = prefix.join("lib");
    let lib64 = prefix.join("lib64");

    let mut cmd = Command::new(bin.join(program_name));

    // Mirror your shell script:
    cmd.env("PREFIX", &prefix);
    cmd.env(
        "PATH",
        format!(
            "{}:{}",
            bin.display(),
            std::env::var("PATH").unwrap_or_default()
        ),
    );

    let old_ld = std::env::var("LD_LIBRARY_PATH").unwrap_or_default();
    cmd.env(
        "LD_LIBRARY_PATH",
        format!("{}:{}:{}", lib.display(), lib64.display(), old_ld),
    );

    let old_pc = std::env::var("PKG_CONFIG_PATH").unwrap_or_default();
    cmd.env(
        "PKG_CONFIG_PATH",
        format!(
            "{}:{}:{}",
            prefix.join("lib/pkgconfig").display(),
            prefix.join("share/pkgconfig").display(),
            old_pc
        ),
    );

    Some(cmd)
}

fn ensure_ipod_mounted(mount_path: &str) -> Result<(), String> {
    // 1) Make sure mount dir exists
let mp = std::path::Path::new(mount_path);

if mp.exists() {
    // Exists: must be a directory (mountpoint)
    if !mp.is_dir() {
        return Err(format!(
            "Mount path exists but is not a directory: {}",
            mount_path
        ));
    }
} else {
    // Does not exist: create it
    std::fs::create_dir_all(mp)
        .map_err(|e| format!("create_dir_all({}): {}", mount_path, e))?;
}

    // 2) If it already looks mounted (iPod Touch media has iTunes_Control), do nothing
let mp = std::path::PathBuf::from(mount_path);

// If already mounted, Media path exists and contains iTunes_Control
let media = mp.join("var/mobile/Media");
let itunes_control = media.join("iTunes_Control");
if itunes_control.exists() {
    return Ok(());
}

    // 3) Run legacy ifuse: deps/ios-legacy/bin/ifuse --root <mountpoint>
    let mut cmd = legacy_cmd("ifuse").ok_or("legacy_cmd(ifuse) failed")?;

    let out = cmd
        .arg("--root")
        .arg(mount_path)
        .output()
        .map_err(|e| format!("ifuse spawn failed: {}", e))?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(format!("ifuse failed: {} {}", out.status, stderr.trim()));
    }

    Ok(())
}

fn usb_present_lsusb() -> bool {
    let out = Command::new("lsusb").output();
    let Ok(out) = out else {
        return false;
    };
    if !out.status.success() {
        return false;
    }

    let s = String::from_utf8_lossy(&out.stdout);
    s.to_lowercase().contains(crate::ui::text::IPOD_USB_ID)
}

fn legacy_device_visible() -> bool {
    // deps/ios-legacy/bin/idevice_id -l
    let mut cmd = match legacy_cmd("idevice_id") {
        Some(c) => c,
        None => return false,
    };

    let out = cmd.arg("-l").output();
    let Ok(out) = out else {
        return false;
    };
    if !out.status.success() {
        return false;
    }

    let s = String::from_utf8_lossy(&out.stdout);
    !s.trim().is_empty()
}

enum SyncMsg {
    Progress(String),
    Done,
    Error(String),
}
