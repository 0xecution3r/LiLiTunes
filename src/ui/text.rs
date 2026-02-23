use adw::prelude::*;

use crate::ui::UiHandles;
pub use crate::sync::device_ids::IPOD_USB_ID;
// ===== Dialog Text Constants =====
//
// All human-readable dialog text lives here.
// dialogs.rs should never contain hardcoded words.
pub const ADD_FOLDER_DIALOG_TITLE: &str = "Select Music Folder";
pub const ADD_FOLDER_CANCEL_BUTTON: &str = "Cancel";
pub const ADD_FOLDER_OPEN_BUTTON: &str = "Open";
pub const ADD_FOLDER_SELECTED_LABEL: &str = "Selected!";
pub const ADD_FOLDER_DEFAULT_LABEL: &str = "Add Folder";

// ===== Import / Scanning UI Text =====
//
// Used when the Add Folder operation is running.
pub const ADD_FOLDER_SCANNING_LABEL: &str = "Scanning...";

// ===== App & Error Text =====
//
// These keep main.rs free from hardcoded words.
pub const APP_ID: &str = "com.ghost.lilitunes";
pub const DB_INIT_FAILED: &str = "DB init failed";

pub const IMPORT_ERROR_PREFIX: &str = "Import error: ";
pub const IMPORT_WORKER_DISCONNECTED: &str = "Import error: worker disconnected";

pub const ERR_READ_DIR_FAILED: &str = "read_dir failed: ";
pub const ERR_DIR_ENTRY_FAILED: &str = "dir entry failed: ";

// ===== GStreamer Play =====
//
// Text for Gstreamer
pub const NOTHING_PLAYING_LABEL: &str = "Nothing playing";
pub const GST_INIT_FAILED: &str = "Failed to init GStreamer";
pub const PLAY_LABEL: &str = "Play";
pub const PAUSE_LABEL: &str = "Pause";

// Used to build "Artist — Title" consistently without hardcoding it in logic
pub const TRACK_DISPLAY_SEPARATOR: &str = " — ";

// ===== File Type Rules =====
//
// Not shown to user, but still "words" (string literals), so we centralize them.
pub const AUDIO_EXTS: [&str; 5] = ["mp3", "flac", "ogg", "m4a", "wav"];

// ===== iPod Sync Text / Constants =====
pub const SYNC_IPOD_DEFAULT_LABEL: &str = "Sync iPod";
pub const SYNC_IPOD_WORKING_LABEL: &str = "Syncing...";

pub const SYNC_DONE_LABEL: &str = "iPod sync complete";
pub const SYNC_ERR_DISCONNECTED_LABEL: &str = "ERROR sync channel disconnected";

// Helper executable + args (still “text.rs” per your rule)
pub const SYNC_HELPER_PATH: &str = "src/sync/lilitunes-sync-c";
pub const SYNC_ARG_MOUNT: &str = "--mount";
pub const SYNC_ARG_SQLITE: &str = "--sqlite";

// Default paths (we’ll make these configurable later)
pub const SYNC_IPOD_MEDIA_PATH: &str = "/home/some0nee/ipod/var/mobile/Media";
pub const SYNC_IFUSE_MOUNTPOINT: &str = "/home/some0nee/ipod";
pub const SYNC_DEFAULT_DB_PATH: &str = "library.db";

// Error prefixes (so we never hardcode strings in sync module)
pub const SYNC_ERR_START: &str = "ERROR failed to start sync helper: ";
pub const SYNC_ERR_WAIT: &str = "ERROR failed waiting for helper: ";
pub const SYNC_ERR_EXIT: &str = "ERROR helper exited ";

pub const IPOD_STATUS_CONNECTED: &str = "iPod detected";
pub const IPOD_STATUS_DISCONNECTED: &str = "iPod not detected";

// ===== legacy env prefix =====
// Matches your setup script: export PREFIX="$HOME/ios-legacy"
pub const IOS_LEGACY_HOME_DIRNAME: &str = "ios-legacy";

// If you want repo-local prefix instead, you can swap later:
// pub const IOS_LEGACY_PREFIX_FALLBACK: &str = "src/deps/ios-legacy";

pub const LEGACY_ENV_PATH_KEY: &str = "PATH";
pub const LEGACY_ENV_PKG_CONFIG_KEY: &str = "PKG_CONFIG_PATH";
pub const LEGACY_ENV_LD_LIBRARY_KEY: &str = "LD_LIBRARY_PATH";

// ===== usbmuxd legacy instructions (iPod touch 1st gen) =====
pub const USBMUXD_FIX_TITLE: &str = "iPod not visible to legacy tools";
pub const USBMUXD_FIX_BODY: &str = "Your iPod is detected over USB, but legacy idevice tools cannot see it.\n\
For iPod touch 1st gen you usually need to restart usbmuxd in legacy mode.\n\n\
Run these commands in a terminal:\n\n\
sudo systemctl stop usbmuxd\n\
sudo systemctl stop usbmuxd.socket 2>/dev/null\n\
sudo usbmuxd -p -U usbmux\n";

pub const USBMUXD_FIX_OK: &str = "OK";

// This module owns ALL human-readable text/icons/titles.
// app_ui.rs stays "shapes only".
pub fn apply_text(ui: &UiHandles) {
    // Window title
    ui.window.set_title(Some("LiliTunes"));

    // Header title + subtitle
    let title = adw::WindowTitle::builder()
        .title("LiliTunes")
        .subtitle("Library")
        .build();
    ui.header.set_title_widget(Some(&title));

    // Header buttons
    ui.add_folder_button.set_label(ADD_FOLDER_DEFAULT_LABEL);
    ui.sync_ipod_button.set_label("Sync iPod");

    // Sidebar labels
    ui.sidebar_library_label.set_label("Library");
    ui.sidebar_playlists_label.set_label("Playlists");
    ui.sidebar_devices_label.set_label("Devices");

    // Player buttons (icons + play label)
    ui.prev_button.set_label("⏮");
    ui.play_button.set_label("Play");
    ui.next_button.set_label("⏭");

    // Player text
    ui.now_playing.set_label("Nothing playing");
}
