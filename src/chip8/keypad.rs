use std::sync::{Condvar, Mutex};

use num_enum::FromPrimitive;

#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
#[repr(u8)]
pub enum Key {
    #[num_enum(default)]
    Key0 = 0x0,
    Key1 = 0x1,
    Key2 = 0x2,
    Key3 = 0x3,
    Key4 = 0x4,
    Key5 = 0x5,
    Key6 = 0x6,
    Key7 = 0x7,
    Key8 = 0x8,
    Key9 = 0x9,
    KeyA = 0xA,
    KeyB = 0xB,
    KeyC = 0xC,
    KeyD = 0xD,
    KeyE = 0xE,
    KeyF = 0xF,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum KeyState {
    Up,
    Down,
}

#[derive(Debug)]
pub struct KeyPad {
    key_pressed: Mutex<Option<Key>>,
    is_key_pressed: Condvar,
}

impl KeyPad {
    pub fn new() -> KeyPad {
        KeyPad {
            key_pressed: Mutex::new(None),
            is_key_pressed: Condvar::new(),
        }
    }

    pub fn press(&mut self, key: Key) {
        self.key_pressed.lock().unwrap().replace(key);
        self.is_key_pressed.notify_all();
    }

    pub fn release(&mut self) {
        self.key_pressed.lock().unwrap().take();
    }

    pub fn is_pressed(&self, key: Key) -> bool {
        self.key_pressed.lock().unwrap().is_some_and(|k| k == key)
    }

    pub fn has_any_key_pressed(&self) -> bool {
        self.key_pressed.lock().unwrap().is_some()
    }

    /// If a key is already pressed, immediately returns
    /// the key. Otherwise, blocks until a key is pressed.
    pub fn wait_for_key_press(&self) -> Key {
        loop {
            let guard = self.key_pressed.lock().unwrap();
            match guard.as_ref() {
                Some(key) => return *key,
                None => {
                    let _key = self.is_key_pressed.wait(guard).unwrap();
                }
            }
        }
    }
}
