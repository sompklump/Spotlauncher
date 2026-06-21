use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::gio::{self, AppInfo};
use gtk4::{Box, CssProvider, Image, Label, ListBox, Separator, glib};
use gtk4::{PolicyType, ScrolledWindow, prelude::*};

use crate::search::file_index::AppFile;

#[derive(Clone)]
pub struct ResultsList {
    pub app_list: ListBox,
    pub file_list: ListBox,
    pub separator: Separator,
    pub app_scroll: ScrolledWindow,
    pub file_scroll: ScrolledWindow,
    pub window: gtk4::ApplicationWindow,
    pub app_list_active: Rc<Cell<bool>>,
    app_results: Rc<RefCell<Vec<AppInfo>>>, // interior mutable
    file_results: Rc<RefCell<Vec<AppFile>>>,
}

impl ResultsList {
    pub fn new(root: &gtk4::Box, window: &gtk4::ApplicationWindow) -> Self {
        Self::apply_css();
        // Apps list
        let app_list: ListBox = ListBox::new();
        app_list.set_selection_mode(gtk4::SelectionMode::Single);
        app_list.set_visible(false);
        app_list.set_focusable(true);

        // Files list
        let file_list: ListBox = ListBox::new();
        file_list.set_selection_mode(gtk4::SelectionMode::Single);
        file_list.set_visible(false);
        file_list.set_focusable(true);
        // file_list.

        // App scroll
        let app_scroll: ScrolledWindow = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .vscrollbar_policy(PolicyType::Automatic)
            .propagate_natural_height(true)
            .build();

        app_scroll.set_visible(false);
        app_scroll.set_focusable(false);

        let file_scroll: ScrolledWindow = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .vscrollbar_policy(PolicyType::Automatic)
            .propagate_natural_height(true)
            .build();

        file_scroll.set_visible(false);
        file_scroll.set_focusable(false);

        // Load CSS
        let css: gtk4::CssProvider = gtk4::CssProvider::new();
        css.load_from_data("scrolledwindow { min-height: 0; max-height: .6rem; }");
        gtk4::style_context_add_provider_for_display(
            &gtk4::gdk::Display::default().unwrap(),
            &css,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        // Divider
        let separator = Separator::new(gtk4::Orientation::Horizontal);
        separator.set_margin_top(8);
        separator.set_margin_bottom(8);
        separator.add_css_class("dim-separator");
        separator.set_visible(false);

        app_scroll.set_child(Some(&app_list));
        file_scroll.set_child(Some(&file_list));

        root.append(&app_scroll);
        root.append(&separator);
        root.append(&file_scroll);

        Self {
            app_list,
            file_list,
            separator,
            app_scroll,
            file_scroll,
            window: window.clone(),
            app_results: Rc::new(RefCell::new(vec![])),
            file_results: Rc::new(RefCell::new(vec![])),
            app_list_active: Rc::new(Cell::new(true)),
        }
    }

    pub fn update(&self) {
        self.app_list_active.set(true);
        let show_sep =
            !self.app_results.borrow().is_empty() && !self.file_results.borrow().is_empty();
        self.separator.set_visible(show_sep);
        // self.refresh_list_selection();
    }

    pub fn update_apps(&self, items: &[AppInfo]) {
        *self.app_results.borrow_mut() = items.to_vec();

        while let Some(row) = self.app_list.first_child() {
            self.app_list.remove(&row);
        }

        for item in items {
            let hbox = Box::new(gtk4::Orientation::Horizontal, 8);

            let label: Label = Label::new(Some(&item.name()));
            let icon = if let Some(icon) = item.icon() {
                Image::from_gicon(&icon)
            } else {
                Image::new()
            };

            label.set_xalign(0.0);

            icon.set_pixel_size(24);

            hbox.set_focusable(false);

            hbox.append(&icon);
            hbox.append(&label);

            let row = gtk4::ListBoxRow::new();
            row.set_focusable(true);
            row.set_child(Some(&hbox));
            self.app_list.append(&row);
        }

        if items.is_empty() {
            self.app_scroll.set_visible(false);
            self.app_scroll.set_size_request(-1, 0);
            self.app_list.set_visible(false);
        } else {
            let height: i32 = (items.len() as i32 * 40).min(200);
            self.app_scroll.set_size_request(-1, height);
            self.app_scroll.set_visible(true);
            self.app_list.set_visible(true);
        }

        self.update();

        self.window.set_default_size(self.window.width(), -1);
        self.window.queue_resize();
    }

    pub fn update_files(&self, items: &[AppFile]) {
        *self.file_results.borrow_mut() = items.to_vec();

        while let Some(row) = self.file_list.first_child() {
            self.file_list.remove(&row);
        }

        for item in items {
            let hbox = Box::new(gtk4::Orientation::Horizontal, 8);
            let vbox = Box::new(gtk4::Orientation::Vertical, 2);

            let label = Label::new(Some(&item.name));
            label.set_xalign(0.0);

            let icon = Image::from_gicon(&item.icon);
            icon.set_pixel_size(24);

            let path_lbl = Label::new(Some(&item.path.to_str().unwrap().trim()));
            path_lbl.add_css_class("path");

            vbox.set_focusable(false);
            vbox.append(&label);
            vbox.append(&path_lbl);

            hbox.set_focusable(false);

            hbox.append(&icon);
            hbox.append(&vbox);

            let row = gtk4::ListBoxRow::new();
            row.set_focusable(true);
            row.set_child(Some(&hbox));
            self.file_list.append(&row);
        }

        if items.is_empty() {
            self.file_scroll.set_visible(false);
            self.file_scroll.set_size_request(-1, 0);
            self.file_list.set_visible(false);
        } else {
            let height: i32 = (items.len() as i32 * 40).min(200);
            self.file_scroll.set_size_request(-1, height);
            self.file_scroll.set_visible(true);
            self.file_list.set_visible(true);
        }

        self.update();

        self.window.set_default_size(self.window.width(), -1);
        self.window.queue_resize();
    }

    pub fn connect_selection(&self) {
        let app_results: Rc<RefCell<Vec<AppInfo>>> = self.app_results.clone();
        let file_results = self.file_results.clone();

        let app_window: gtk4::ApplicationWindow = self.window.clone();
        let file_window: gtk4::ApplicationWindow = self.window.clone();

        self.app_list
            .connect_row_activated(move |_, row: &gtk4::ListBoxRow| {
                let index: usize = row.index() as usize;
                let results: std::cell::Ref<'_, Vec<AppInfo>> = app_results.borrow();
                if let Some(app) = results.get(index) {
                    app.launch(&[], gtk4::gio::AppLaunchContext::NONE).ok();
                    app_window.set_visible(false);
                }
            });

        self.file_list
            .connect_row_activated(move |_, row: &gtk4::ListBoxRow| {
                let index: usize = row.index() as usize;
                let results = file_results.borrow();
                if let Some(file) = results.get(index) {
                    let uri = gio::File::for_path(&file.path).uri();
                    gio::AppInfo::launch_default_for_uri(&uri, gio::AppLaunchContext::NONE).ok();
                    file_window.set_visible(false);
                }
            });
    }

    pub fn switch_lists(&self) {
        let either_has_selection =
            self.app_list.selected_row().is_some() || self.file_list.selected_row().is_some();

        if either_has_selection {
            self.app_list_active.set(!self.app_list_active.get());
        }

        self.refresh_list_selection();
    }

    fn refresh_list_selection(&self) {
        let (list, old_list) = if self.app_list_active.get() {
            (&self.app_list, &self.file_list)
        } else {
            (&self.file_list, &self.app_list)
        };

        old_list.unselect_all();
        list.grab_focus();

        if list.selected_row().is_none() {
            if let Some(row) = list.row_at_index(0) {
                list.select_row(Some(&row));
            }
        }

        // Defer focus grab to after current event is fully processed
        let row = list.selected_row().clone();
        glib::idle_add_local_once(move || {
            if let Some(r) = row {
                r.grab_focus();
            }
        });
    }

    fn apply_css() {
        let css: CssProvider = CssProvider::new();
        css.load_from_data(
            "
            .path { font-size: 11px; }
    .dim-separator {
        background-color: #444444;
        min-height: 1px;
    }
    listbox.inactive row:selected {
    background-color: alpha(@selected_bg_color, 0.3);
    color: @text_color;
}
",
        );
        gtk4::style_context_add_provider_for_display(
            &gtk4::gdk::Display::default().unwrap(),
            &css,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
