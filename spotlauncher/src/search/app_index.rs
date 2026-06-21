use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use gtk4::gio::AppInfo;
use gtk4::gio::prelude::AppInfoExt;
use gtk4::glib;

pub struct AppIndex {
    pub app_index: Rc<RefCell<HashSet<AppInfo>>>,
}

impl AppIndex {
    pub fn new() -> Self {
        Self {
            app_index: Rc::new(RefCell::new(AppInfo::all().into_iter().collect())),
        }
    }

    /// Call once after construction. Returns a `SourceId` you can use
    /// to cancel the timer later with `source_id.remove()`.
    pub fn start_background_refresh(&self, interval_secs: u32) -> glib::SourceId {
        let app_index = Rc::clone(&self.app_index);
        glib::timeout_add_seconds_local(interval_secs, move || {
            let new_apps: HashSet<AppInfo> = AppInfo::all().into_iter().collect();
            let mut apps = app_index.borrow_mut();
            if *apps != new_apps {
                *apps = new_apps;
            }
            glib::ControlFlow::Continue 
        })
    }

    pub fn search(&self, query: &str) -> Vec<AppInfo> {
        if query.is_empty() {
            return vec![];
        }

        let query = query.to_lowercase();

        let mut arr: Vec<AppInfo> = self.app_index
            .borrow()
            .iter()
            .cloned()
            .filter(|app: &AppInfo| app.display_name().to_lowercase().contains(&query))
            .take(10)
            .collect();

        arr.sort_by(|a: &AppInfo,b: &AppInfo| a.name().to_lowercase().cmp(&b.name().to_lowercase()));

        arr
    }
}
