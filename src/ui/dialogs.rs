// src/ui/dialogs.rs

use gtk::prelude::*;
use std::path::PathBuf;
use std::rc::Rc;

use crate::ui::UiHandles;
use crate::ui::text;

pub fn wire_add_folder<F>(ui: &UiHandles, on_selected: F)
where
    F: Fn(PathBuf) + 'static,
{
    let window = ui.window.clone();

    // Wrap the callback in Rc so we can reuse it across multiple clicks.
    let on_selected = Rc::new(on_selected);

    ui.add_folder_button.connect_clicked(move |_| {
        let dialog = gtk::FileChooserDialog::new(
            Some(text::ADD_FOLDER_DIALOG_TITLE),
            Some(&window),
            gtk::FileChooserAction::SelectFolder,
            &[
                (text::ADD_FOLDER_CANCEL_BUTTON, gtk::ResponseType::Cancel),
                (text::ADD_FOLDER_OPEN_BUTTON, gtk::ResponseType::Accept),
            ],
        );

        dialog.set_modal(true);

        // Clone the callback for this particular dialog instance.
        let on_selected = Rc::clone(&on_selected);

        dialog.connect_response(move |d, response| {
            let selected = if response == gtk::ResponseType::Accept {
                d.file().and_then(|f| f.path())
            } else {
                None
            };

            d.close();

            if let Some(path) = selected {
                on_selected(path);
            }
        });

        dialog.show();
    });
}
pub fn show_usbmuxd_fix_dialog(window: &adw::ApplicationWindow) {
    let dialog = gtk::MessageDialog::new(
        Some(window),
        gtk::DialogFlags::MODAL,
        gtk::MessageType::Info,
        gtk::ButtonsType::None,
        crate::ui::text::USBMUXD_FIX_BODY,
    );

    dialog.set_title(Some(crate::ui::text::USBMUXD_FIX_TITLE));
    dialog.add_button(crate::ui::text::USBMUXD_FIX_OK, gtk::ResponseType::Ok);

    dialog.connect_response(|d, _| d.close());
    dialog.show();
}
