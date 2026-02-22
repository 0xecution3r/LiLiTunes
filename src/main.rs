// This brings in helpful GTK/Adwaita traits and methods
// so we can build and control the application.
use adw::prelude::*;
use gtk::prelude::*;

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

// Internal modules:
// - core: database/backend logic
// - ui: user interface building + text + dialogs
mod core;
mod playback;
mod sync;
mod ui;
// Import the database handle type so we can open the DB.
use core::db::Library;
use core::metadata::read_track_meta;

fn main() {
    // Create the application object (your app "instance").
    // The application_id is a unique ID for desktop environments.
    let app = adw::Application::builder()
        .application_id(ui::text::APP_ID)
        .build();

    // When the app launches, GTK calls build_ui().
    app.connect_activate(build_ui);

    // Start the GTK event loop (handles clicks, drawing, window events).
    app.run();
}

// This function runs when the app activates.
// It builds the window, applies text labels, and wires dialog behaviors.
fn build_ui(app: &adw::Application) {
    // Open/create the database.
    // If it fails, stop immediately with a clear message.
    let library = Library::new().expect(ui::text::DB_INIT_FAILED);

    // Build the UI widgets/layout (shapes only, no words if you're using text.rs).
    // This returns handles to important widgets (buttons, labels, lists, etc.).
    let ui = ui::build_ui(app, &library);

    // Apply all human-readable text (button labels, window title, sidebar text, etc.)
    // from a single centralized place.
    ui::text::apply_text(&ui);
    sync::wire_sync_button(&ui);
    ui::track_list::refresh(&ui.track_list, &library);
    sync::start_device_watch(&ui);

    let track_cache =
        std::rc::Rc::new(std::cell::RefCell::new(Vec::<playback::TrackRowData>::new()));
    let current_index = std::rc::Rc::new(std::cell::Cell::new(None));
    let is_playing = std::rc::Rc::new(std::cell::Cell::new(false));
    let user_seeking = std::rc::Rc::new(std::cell::Cell::new(false));
    let pending_seek_seconds = std::rc::Rc::new(std::cell::Cell::new(0.0));

    let pending_seek_for_change = pending_seek_seconds.clone();
    ui.seek_scale
        .connect_change_value(move |_scale, _scroll, value| {
            pending_seek_for_change.set(value);
            glib::Propagation::Proceed
        });

    fill_track_cache(&library, &track_cache);
    let controller = std::rc::Rc::new(playback::PlaybackController::new(
        track_cache.clone(),
        current_index.clone(),
        is_playing.clone(),
    ));
    // Gesture controller that detects mouse/touch press + release on the slider
    let click = gtk::GestureClick::new();

    let user_seeking_press = user_seeking.clone();
    click.connect_pressed(move |_, _, _, _| {
        user_seeking_press.set(true);
    });

    let controller_for_release = controller.clone();
    let pending_seek_for_release = pending_seek_seconds.clone();
    let user_seeking_release = user_seeking.clone();

    click.connect_released(move |_, _, _, _| {
        user_seeking_release.set(false);

        let target = pending_seek_for_release.get();
        controller_for_release.as_ref().seek_to_seconds(target);
    });

    // Attach the controller to the seek slider widget
    ui.seek_scale.add_controller(click);

    let controller_for_tick = controller.clone();
    let seek_scale_tick = ui.seek_scale.clone();
    let user_seeking_tick = user_seeking.clone();

    glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
        if user_seeking_tick.get() {
            return glib::ControlFlow::Continue;
        }

        if let Some(pos) = controller_for_tick.as_ref().position_seconds() {
            if pos.is_finite() && pos >= 0.0 {
                seek_scale_tick.set_value(pos);
            }
        }

        glib::ControlFlow::Continue
    });

    let play_btn = ui.play_button.clone();
    let now_lbl = ui.now_playing.clone();
    let controller_for_play = controller.clone();

    ui.play_button
        .connect_clicked(move |_| match controller_for_play.toggle_play_pause() {
            playback::ToggleResult::NoTrack => {
                now_lbl.set_label(ui::text::NOTHING_PLAYING_LABEL);
                play_btn.set_label(ui::text::PLAY_LABEL);
            }
            playback::ToggleResult::NowPlaying(_idx) => {
                play_btn.set_label(ui::text::PAUSE_LABEL);
            }
            playback::ToggleResult::NowPaused(_idx) => {
                play_btn.set_label(ui::text::PLAY_LABEL);
            }
        });

    let play_btn_row = ui.play_button.clone();
    let now_lbl_row = ui.now_playing.clone();
    let controller_for_rows = controller.clone();
    let library_for_rows = library.clone();
    let track_cache_for_rows = track_cache.clone();
    let seek_scale_row = ui.seek_scale.clone();

    ui.track_list.connect_row_activated(move |_list, row| {
        let idx = row.index();
        if idx < 0 {
            return;
        }
        let idx = idx as usize;

        // Make sure cache matches DB order right now
        fill_track_cache(&library_for_rows, &track_cache_for_rows);

        if let Some(display) = controller_for_rows.play_from_index(idx) {
            now_lbl_row.set_label(&display);
            play_btn_row.set_label(ui::text::PAUSE_LABEL);
            if let Some(dur) = controller_for_rows.duration_seconds() {
                if dur.is_finite() && dur > 0.0 {
                    seek_scale_row.set_range(0.0, dur);
                }
            }
        }
    });

    // Next Button Logic
    let next_btn = ui.next_button.clone();
    let play_btn_next = ui.play_button.clone();
    let now_lbl_next = ui.now_playing.clone();

    let controller_for_next = controller.clone();
    let library_for_next = library.clone();
    let track_cache_for_next = track_cache.clone();
    let seek_scale_next = ui.seek_scale.clone();

    ui.next_button.connect_clicked(move |_| {
        // Ensure cache matches DB order
        fill_track_cache(&library_for_next, &track_cache_for_next);

        if let Some(display) = controller_for_next.next_track() {
            now_lbl_next.set_label(&display);
            play_btn_next.set_label(ui::text::PAUSE_LABEL);
            if let Some(dur) = controller_for_next.duration_seconds() {
                if dur.is_finite() && dur > 0.0 {
                    seek_scale_next.set_range(0.0, dur);
                }
            }
        } else {
            now_lbl_next.set_label(ui::text::NOTHING_PLAYING_LABEL);
            play_btn_next.set_label(ui::text::PLAY_LABEL);
        }
    });

    // Previous Button Logic
    let prev_btn = ui.prev_button.clone();
    let play_btn_prev = ui.play_button.clone();
    let now_lbl_prev = ui.now_playing.clone();

    let controller_for_prev = controller.clone();
    let library_for_prev = library.clone();
    let track_cache_for_prev = track_cache.clone();
    let seek_scale_prev = ui.seek_scale.clone();

    ui.prev_button.connect_clicked(move |_| {
        // Ensure cache matches DB order
        fill_track_cache(&library_for_prev, &track_cache_for_prev);

        if let Some(display) = controller_for_prev.prev_track() {
            now_lbl_prev.set_label(&display);
            play_btn_prev.set_label(ui::text::PAUSE_LABEL);
            if let Some(dur) = controller_for_prev.duration_seconds() {
                if dur.is_finite() && dur > 0.0 {
                    seek_scale_prev.set_range(0.0, dur);
                }
            }
        } else {
            now_lbl_prev.set_label(ui::text::NOTHING_PLAYING_LABEL);
            play_btn_prev.set_label(ui::text::PLAY_LABEL);
        }
    });

    let controller_for_seek = controller.clone();

    let pending_seek_seconds = std::rc::Rc::new(std::cell::Cell::new(0.0));

    let pending_seek_for_change = pending_seek_seconds.clone();
    ui.seek_scale
        .connect_change_value(move |_scale, _scroll, value| {
            pending_seek_for_change.set(value);
            glib::Propagation::Proceed
        });
    // Wire up dialog behaviors (popups).
    // Right now this hooks "Add Folder" button to the folder chooser dialog.
    //
    // We grab the widgets we need inside the callback.
    // These clones let the callback own them safely.
    let add_button = ui.add_folder_button.clone();
    let track_list = ui.track_list.clone();

    // Wire the dialog so it returns the selected folder path.
    // When a folder is chosen, we start importing it into the DB.
    ui::dialogs::wire_add_folder(&ui, move |folder_path: PathBuf| {
        // Give the user immediate feedback that work is starting.
        add_button.set_sensitive(false);
        add_button.set_label(ui::text::ADD_FOLDER_SCANNING_LABEL);

        // Run import in a background thread so the UI stays responsive.
        import_folder_async(
            folder_path,
            library.clone(),
            add_button.clone(),
            track_list.clone(),
        );
    });
}

// Import all audio files from a folder into the DB without freezing the UI.
//
// - Worker thread scans files + writes DB rows
// - Main thread waits for completion
fn import_folder_async(
    folder: PathBuf,
    library: Library,
    add_button: gtk::Button,
    track_list: gtk::ListBox,
) {
    let (tx, rx) = mpsc::channel::<Result<usize, String>>();

    // Clone library so we can use one copy in the worker thread
    // and keep the original for refreshing the UI after.
    let lib_for_worker = library.clone();

    thread::spawn(move || {
        let mut count: usize = 0;

        let mut on_file = |file_path: PathBuf| {
            let meta = read_track_meta(&file_path);

            if let Some(path_str) = file_path.to_str() {
                let _ =
                    lib_for_worker.upsert_track(path_str, &meta.artist, &meta.title, &meta.album);
                count += 1;
            }
        };

        let result = walk_folder(&folder, &mut on_file);

        match result {
            Ok(()) => {
                let _ = tx.send(Ok(count));
            }
            Err(e) => {
                let _ = tx.send(Err(e));
            }
        }
    });

    glib::timeout_add_local(Duration::from_millis(50), move || match rx.try_recv() {
        Ok(Ok(_n)) => {
            add_button.set_label(ui::text::ADD_FOLDER_DEFAULT_LABEL);
            add_button.set_sensitive(true);

            ui::track_list::refresh(&track_list, &library);
            glib::ControlFlow::Break
        }
        Ok(Err(err)) => {
            add_button.set_label(ui::text::ADD_FOLDER_DEFAULT_LABEL);
            add_button.set_sensitive(true);

            eprintln!("{}{}", ui::text::IMPORT_ERROR_PREFIX, err);

            glib::ControlFlow::Break
        }
        Err(mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
        Err(mpsc::TryRecvError::Disconnected) => {
            add_button.set_label(ui::text::ADD_FOLDER_DEFAULT_LABEL);
            add_button.set_sensitive(true);

            eprintln!("{}", ui::text::IMPORT_WORKER_DISCONNECTED);

            glib::ControlFlow::Break
        }
    });
}

fn fill_track_cache(
    library: &core::db::Library,
    track_cache: &std::rc::Rc<std::cell::RefCell<Vec<playback::TrackRowData>>>,
) {
    let mut cache = track_cache.borrow_mut();
    cache.clear();

    if let Ok(tracks) = library.all_tracks() {
        for t in tracks {
            let uri = gio::File::for_path(&t.path).uri().to_string();
            let display = format!(
                "{}{}{}",
                t.artist,
                ui::text::TRACK_DISPLAY_SEPARATOR,
                t.title
            );

            cache.push(playback::TrackRowData { uri, display });
        }
    }
}

// Walk through folders recursively and call `on_file` for each audio file.
//
// - It goes through every folder/subfolder
// - For each file, if it's audio, it calls your callback
fn walk_folder<F>(root: &Path, on_file: &mut F) -> Result<(), String>
where
    F: FnMut(PathBuf),
{
    let entries =
        std::fs::read_dir(root).map_err(|e| format!("{}{}", ui::text::ERR_READ_DIR_FAILED, e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("{}{}", ui::text::ERR_DIR_ENTRY_FAILED, e))?;

        let path = entry.path();

        if path.is_dir() {
            // Reuse the SAME callback reference in recursion
            walk_folder(&path, on_file)?;
            continue;
        }

        if is_audio_file(&path) {
            on_file(path);
        }
    }

    Ok(())
}
fn set_seek_range_to_track(ui: &ui::UiHandles, controller: &playback::PlaybackController) {
    let Some(dur) = controller.duration_seconds() else {
        return;
    };

    if dur.is_finite() && dur > 0.0 {
        ui.seek_scale.set_range(0.0, dur);
    }
}

// Decide if a file is an audio file based on extension.
fn is_audio_file(p: &Path) -> bool {
    let Some(ext) = p.extension().and_then(|e| e.to_str()) else {
        return false;
    };

    let ext = ext.to_lowercase();
    ui::text::AUDIO_EXTS.iter().any(|&e| e == ext.as_str())
}
