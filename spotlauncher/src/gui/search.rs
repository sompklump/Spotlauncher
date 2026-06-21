use gtk4::{
    SearchEntry,
    prelude::{BoxExt, WidgetExt},
};

pub struct SearchBar {
    pub entry: SearchEntry,
}
impl SearchBar {
    pub fn new(root: &gtk4::Box) -> Self {
        let entry = SearchEntry::new();
        entry.set_focusable(true);
        entry.set_placeholder_text(Some("Search apps & files..."));
        // entry.set_key_capture_widget(&Some(window));
        root.append(&entry); // appends to the shared container
        Self { entry }
    }
}
