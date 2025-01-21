use num_enum::IntoPrimitive;

#[derive(Debug, PartialEq, Eq, Clone, Copy, IntoPrimitive)]
#[repr(u8)]
pub enum Key {
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
    Down,
    Up,
}
