use gtk::prelude::*;

use crate::core::db::Library;

// Rebuild the visible track list from the database.
//
// - Remove all rows currently shown
// - Ask the database for all tracks
// - Add one row per track back into the list
pub fn refresh(track_list: &gtk::ListBox, library: &Library) {
    // Clear existing rows from the list box
    while let Some(child) = track_list.first_child() {
        track_list.remove(&child);
    }

    // Pull all tracks from the DB and display them
    if let Ok(tracks) = library.all_tracks() {
        for t in tracks {
            // Create a row container
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
            row.set_margin_start(12);
            row.set_margin_end(12);
            row.set_margin_top(6);
            row.set_margin_bottom(6);

            // Build display string like: "Artist — Title"
            let display = format!(
                "{}{}{}",
                t.artist,
                crate::ui::text::TRACK_DISPLAY_SEPARATOR,
                t.title
            );

            // Left label: artist/title
            let main = gtk::Label::new(Some(&display));
            main.set_xalign(0.0);
            main.set_hexpand(true);

            // Right label: album
            let album = gtk::Label::new(Some(&t.album));
            album.set_xalign(1.0);

            // Add labels into row, then row into list
            row.append(&main);
            row.append(&album);
            track_list.append(&row);
        }
    }
}
