use gtk4::gio;
use std::path::PathBuf;
use std::process::Command;

#[derive(Clone, Debug)]
pub struct AppFile {
    pub path: PathBuf,
    pub name: String,
    pub icon: gio::Icon,
}

impl AppFile {
    fn from_path_str(path_str: &str) -> Self {
        let path = PathBuf::from(path_str);

        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path_str.to_string());

        let icon = Self::resolve_icon(&path);

        Self { path, name, icon }
    }

    fn resolve_icon(path: &PathBuf) -> gio::Icon {
        // GIO can guess content type from the filename alone (no disk read).
        // e.g. "foo.pdf" → "application/pdf" → a themed icon
        let (content_type, _uncertain) = gio::functions::content_type_guess(path.to_str(), None);

        gio::functions::content_type_get_icon(&content_type)
    }
}

pub struct FileIndex;

impl FileIndex {
    pub fn new() -> Self {
        Self
    }

    pub fn search(&self, query: &str) -> Vec<AppFile> {
        if query.is_empty() {
            return vec![];
        }

        let output = Command::new("plocate")
            .arg("--limit")
            .arg("10")
            .arg(query) // plocate does the filtering — no need to do it ourselves
            .output()
            .unwrap_or_else(|e| panic!("plocate failed: {e}"));

        String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(AppFile::from_path_str)
            .collect()
    }
}
