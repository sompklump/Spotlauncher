use gtk4::glib;
use std::thread;

type Callback = Box<dyn Fn() + 'static>;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    Space,
    Escape,
    Alt,
    Enter,
    Tab,
}

// Unified key type per platform
#[cfg(target_os = "linux")]
impl From<Key> for input_query::KeyCode {
    fn from(key: Key) -> Self {
        match key {
            Key::Space => input_query::KeyCode::KeySpace,
            Key::Escape => input_query::KeyCode::KeyEsc,
            Key::Alt => input_query::KeyCode::KeyLeftAlt,
            Key::Enter => input_query::KeyCode::KeyEnter,
            Key::Tab => input_query::KeyCode::KeyTab,
        }
    }
}

#[cfg(target_os = "macos")]
impl From<Key> for rdev::Key {
    fn from(key: Key) -> Self {
        match key {
            Key::Space => rdev::Key::Space,
            Key::Escape => rdev::Key::Escape,
            Key::Alt => rdev::Key::Alt,
            Key::Enter => rdev::Key::Return,
            Key::Tab => rdev::Key::Tab,
        }
    }
}

pub type PlatformKey = Key;

pub struct KeyBinding {
    keys: Vec<PlatformKey>,
    tx: async_channel::Sender<usize>,
}

pub struct KeyboardHandler {
    bindings: Vec<KeyBinding>,
    callbacks: Vec<Callback>,
    tx: async_channel::Sender<usize>,
    rx: async_channel::Receiver<usize>,
}

impl KeyboardHandler {
    pub fn new() -> Self {
        let (tx, rx) = async_channel::bounded(8);
        Self {
            bindings: vec![],
            callbacks: vec![],
            tx,
            rx,
        }
    }

    pub fn bind(&mut self, keys: Vec<PlatformKey>, action: impl Fn() + 'static) {
        self.callbacks.push(Box::new(action));
        self.bindings.push(KeyBinding {
            keys,
            tx: self.tx.clone(),
        });
    }

    pub fn listen(self) {
        let bindings = self.bindings;
        let rx = self.rx;
        let callbacks = self.callbacks;

        #[cfg(target_os = "linux")]
        thread::spawn(move || {
            let handler = input_query::InputHandler::new();
            loop {
                for (i, binding) in bindings.iter().enumerate() {
                    if binding.keys.iter().all(|k| handler.is_pressed((*k).into())) {
                        binding.tx.send_blocking(i).ok();
                        thread::sleep(std::time::Duration::from_millis(300));
                    }
                }
                thread::sleep(std::time::Duration::from_millis(100));
            }
        });

        #[cfg(target_os = "macos")]
        {
            use rdev::{Event, EventType, listen};
            use std::collections::HashSet;
            use std::sync::{Arc, Mutex};

            let pressed: Arc<Mutex<HashSet<rdev::Key>>> = Arc::new(Mutex::new(HashSet::new()));
            let pressed_clone = Arc::clone(&pressed);

            thread::spawn(move || {
                listen(move |event: Event| match event.event_type {
                    EventType::KeyPress(key) => {
                        let mut keys = pressed_clone.lock().unwrap();
                        keys.insert(key);
                        for (i, binding) in bindings.iter().enumerate() {
                            if binding.keys.iter().all(|k| keys.contains(&(*k).into())) {
                                binding.tx.send_blocking(i).ok();
                            }
                        }
                    }
                    EventType::KeyRelease(key) => {
                        pressed_clone.lock().unwrap().remove(&key);
                    }
                    _ => {}
                })
                .expect("Failed to start global key listener");
            });
        }

        glib::spawn_future_local(async move {
            while let Ok(index) = rx.recv().await {
                if let Some(cb) = callbacks.get(index) {
                    cb();
                }
            }
        });
    }
}
