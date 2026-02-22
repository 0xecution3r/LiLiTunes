use adw::prelude::*;
use gtk::prelude::*;

use crate::core::db::Library;

// UI widget handles only.
// IMPORTANT: This file contains NO human-readable strings.
// All words/icons/titles are applied in ui/text.rs.
pub struct UiHandles {
    pub window: adw::ApplicationWindow,
    pub header: adw::HeaderBar,

    // Header buttons (labels set elsewhere)
    pub add_folder_button: gtk::Button,
    pub sync_ipod_button: gtk::Button,

    // Sidebar "rows" (text set elsewhere)
    pub sidebar: gtk::ListBox,
    pub sidebar_library_label: gtk::Label,
    pub sidebar_playlists_label: gtk::Label,
    pub sidebar_devices_label: gtk::Label,

    // Track list
    pub track_list: gtk::ListBox,

    // Player controls (labels set elsewhere)
    pub prev_button: gtk::Button,
    pub play_button: gtk::Button,
    pub next_button: gtk::Button,
    pub seek_scale: gtk::Scale,
    pub now_playing: gtk::Label,
}

pub fn build_ui(app: &adw::Application, _library: &Library) -> UiHandles {
    // Main window (title set elsewhere)
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .default_width(1200)
        .default_height(800)
        .build();

    // Header bar (title widget set elsewhere)
    let header = adw::HeaderBar::new();

    // Header buttons (no labels here)
    let add_folder_button = gtk::Button::new();
    let sync_ipod_button = gtk::Button::new();

    header.pack_start(&add_folder_button);
    header.pack_end(&sync_ipod_button);

    // Sidebar list (no text here)
    let sidebar = gtk::ListBox::new();
    sidebar.set_width_request(200);

    // Sidebar labels created with empty text; words applied elsewhere
    let sidebar_library_label = gtk::Label::new(None);
    let sidebar_playlists_label = gtk::Label::new(None);
    let sidebar_devices_label = gtk::Label::new(None);

    sidebar.append(&sidebar_library_label);
    sidebar.append(&sidebar_playlists_label);
    sidebar.append(&sidebar_devices_label);

    // Track list + scroller
    let track_list = gtk::ListBox::new();
    track_list.set_vexpand(true);

    let scroll = gtk::ScrolledWindow::builder()
        .child(&track_list)
        .vexpand(true)
        .hexpand(true)
        .build();

    // Player bar container
    let player_bar = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    player_bar.set_margin_top(8);
    player_bar.set_margin_bottom(8);
    player_bar.set_margin_start(12);
    player_bar.set_margin_end(12);

    // Player buttons with no labels here
    let prev_button = gtk::Button::new();
    let play_button = gtk::Button::new();
    let next_button = gtk::Button::new();

    // Seek slider (range is numeric; no words)
    let seek_adjustment = gtk::Adjustment::new(0.0, 0.0, 100.0, 1.0, 5.0, 0.0);
    let seek_scale = gtk::Scale::new(gtk::Orientation::Horizontal, Some(&seek_adjustment));
    seek_scale.set_draw_value(false);
    seek_scale.set_hexpand(true);

    // Now playing label (empty text here)
    let now_playing = gtk::Label::new(None);
    now_playing.set_xalign(0.0);

    player_bar.append(&prev_button);
    player_bar.append(&play_button);
    player_bar.append(&next_button);
    player_bar.append(&seek_scale);
    player_bar.append(&now_playing);

    // Overall layout
    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.append(&header);

    let main_area = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    main_area.append(&sidebar);
    main_area.append(&scroll);

    content.append(&main_area);
    content.append(&player_bar);

    window.set_content(Some(&content));
    window.present();

    UiHandles {
        window,
        header,
        add_folder_button,
        sync_ipod_button,
        sidebar,
        sidebar_library_label,
        sidebar_playlists_label,
        sidebar_devices_label,
        track_list,
        prev_button,
        play_button,
        next_button,
        seek_scale,
        now_playing,
    }
}
