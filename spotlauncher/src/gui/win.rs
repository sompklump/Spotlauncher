use gtk4::{Application, ApplicationWindow, Orientation, prelude::GtkWindowExt};

#[cfg(target_os = "linux")]
use gtk4_layer_shell::{Edge, Layer, LayerShell};

pub struct AppWindow {
    pub window: ApplicationWindow,
    pub root: gtk4::Box,
}

impl AppWindow {
    pub fn new(app: &Application) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .default_width(600)
            .decorated(false)
            .build();

        #[cfg(target_os = "linux")]
        Self::setup_layer_shell(&window);

        #[cfg(not(target_os = "linux"))]
        Self::setup_floating(&window);

        window.set_default_size(600, -1);

        let root: gtk4::Box = gtk4::Box::new(Orientation::Vertical, 0);
        window.set_child(Some(&root));

        Self { window, root }
    }

    #[cfg(target_os = "linux")]
    fn setup_layer_shell(window: &ApplicationWindow) {
        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Bottom, true);
        window.set_anchor(Edge::Left, true);
        window.set_anchor(Edge::Right, true);
        window.set_exclusive_zone(-1);

        let screen_width = 1920; // TODO: get dynamically
        let win_width = 600;
        let margin_h = (screen_width - win_width) / 2;
        window.set_margin(Edge::Left, margin_h);
        window.set_margin(Edge::Right, margin_h);
        window.set_margin(Edge::Top, 200);
        window.set_margin(Edge::Bottom, 0);
    }

    #[cfg(not(target_os = "linux"))]
    fn setup_floating(window: &ApplicationWindow) {
        // Basic centered floating window for macOS
        window.set_resizable(false);
        // GTK will center it by default
    }
}
