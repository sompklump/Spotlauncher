mod gui;
mod keyboard;
mod search;

use gtk4::gio::AppInfo;
use gtk4::gio::prelude::{ApplicationExt, ApplicationExtManual};
use gtk4::glib::object::ObjectExt;
use gtk4::prelude::{EditableExt, EventControllerExt, GtkWindowExt, WidgetExt};
use gtk4::{gdk, glib};
use gui::results::ResultsList;
use gui::search::SearchBar;
use gui::win::AppWindow;
use search::app_index::AppIndex;

use keyboard::keyboard_handler::Key as KBKey;
use keyboard::keyboard_handler::KeyboardHandler;

use crate::search::file_index::FileIndex;

fn main() {
    let app: gtk4::Application =
        gtk4::Application::new(Some("com.spotlauncher"), Default::default());

    app.connect_activate(|app: &gtk4::Application| {
        let win: AppWindow = AppWindow::new(app);
        let search: SearchBar = SearchBar::new(&win.root);
        let results: ResultsList = ResultsList::new(&win.root, &win.window);

        results.connect_selection();

        let app_index: AppIndex = AppIndex::new();
        let _refresh_handle = app_index.start_background_refresh(10);

        let file_index = FileIndex::new();

        win.window.set_default_size(500, -1);
        win.window.queue_resize();

        win.window.connect_notify_local(
            Some("is-active"),
            |window: &gtk4::ApplicationWindow, _| {
                if !window.is_active() {
                    window.hide();
                }
            },
        );

        let mut keyboard_handler: KeyboardHandler = KeyboardHandler::new();

        keyboard_handler.bind(vec![KBKey::Space, KBKey::Alt], {
            let window: gtk4::ApplicationWindow = win.window.clone();
            let search_bar: gtk4::SearchEntry = search.entry.clone();
            move || {
                if !window.is_visible() {
                    let entry = search_bar.clone();
                    window.set_visible(true);

                    glib::timeout_add_local_once(std::time::Duration::from_millis(50), move || {
                        entry.grab_focus();
                        entry.select_region(0, -1);
                    });
                }
            }
        });

        keyboard_handler.bind(vec![KBKey::Escape], {
            let window: gtk4::ApplicationWindow = win.window.clone();
            move || {
                window.set_visible(false);
            }
        });

        keyboard_handler.listen();

        let gtk_key_results = results.clone();

        search
            .entry
            .connect_search_changed(move |entry: &gtk4::SearchEntry| {
                let query: String = entry.text().to_string();

                let app_results: Vec<AppInfo> = app_index.search(&query);
                let file_results = file_index.search(&query);

                results.update_files(&file_results);
                results.update_apps(&app_results);
            });

        search.entry.grab_focus();

        bind_gtk(&win.window, gtk_key_results, search);

        win.window.present();
    });

    app.run();
}

pub fn bind_gtk(window: &gtk4::ApplicationWindow, results: ResultsList, search: SearchBar) {
    let key_controller = gtk4::EventControllerKey::new();
    key_controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
    key_controller.connect_key_pressed(move |_, key, _, modifier| {
        // println!("FOCUS: {}", search.entry.has_focus());
        match key {
            gdk::Key::Tab => {
                results.switch_lists();
                glib::Propagation::Stop
            }
            gdk::Key::a if modifier.contains(gdk::ModifierType::CONTROL_MASK) => {
                search.entry.grab_focus();
                search.entry.select_region(0, -1);
                glib::Propagation::Stop
            }
            gdk::Key::a | gdk::Key::A => {
                search.entry.grab_focus();
                glib::Propagation::Proceed
            }
            gdk::Key::Return => {
                let app_list_active: bool = results.app_list_active.get();

                let list: gtk4::ListBox = if app_list_active {
                    results.app_list.clone()
                } else {
                    results.file_list.clone()
                };

                if list.selected_row().is_some() {
                    list.emit_activate_cursor_row();
                }

                glib::Propagation::Stop
            }
            _ => glib::Propagation::Proceed,
        }
    });
    window.add_controller(key_controller);
}
