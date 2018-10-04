use super::utils::check_bit;
use minifb::{Key, Window};

enum Mode {
    Buttons,
    Directions,
    None,
}

pub enum Button {
    Down,
    Up,
    Left,
    Right,
    Start,
    Select,
    B,
    A,
}

pub struct Joypad {
    register: u8,
    keys: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad {
            register: 0,
            keys: 0,
        }
    }
    pub fn read(&self, address: u16) -> Option<u8> {
        match address {
            0xFF00 => Some(self.register),
            _ => None,
        }
    }

    // Returns true if the write was handled. False otherwise
    pub fn write(&mut self, address: u16, value: u8) -> bool {
        match address {
            0xFF00 => {
                // First clear the upper 4 bits
                self.register &= 0x0F;
                // Then write them
                self.register |= value & 0xF0;
                // Update the key values
                self.update_register();
            }
            _ => return false,
        }
        true
    }

    pub fn update(&mut self, window: &Window) -> bool {
        let mut interrupt = false;

        self.update_button(Button::A, window.is_key_down(Key::Z));
        self.update_button(Button::B, window.is_key_down(Key::X));
        self.update_button(Button::Select, window.is_key_down(Key::C));
        self.update_button(Button::Start, window.is_key_down(Key::Space));
        self.update_button(Button::Up, window.is_key_down(Key::Up));
        self.update_button(Button::Down, window.is_key_down(Key::Down));
        self.update_button(Button::Right, window.is_key_down(Key::Right));
        self.update_button(Button::Left, window.is_key_down(Key::Left));

        // TODO: handle interrupt stuff
        false
    }

    pub fn update_button(&mut self, btn: Button, pressed: bool) -> bool {
        let bit = get_button_bit(btn);
        if pressed {
            let old_value = self.keys;
            // Change the bit for down button to 1
            self.keys |= 1 << bit;
            // Check for interrupt
            if check_bit(old_value, bit) {
                return true;
            }
        } else {
            // Button is up, so change the bit to 0
            self.keys &= !(1 << bit);
        }
        false
    }
    fn update_register(&mut self) {
        // Update direction keys
        if !check_bit(self.register, 4) {
            for i in 0..=3 {
                let pressed = check_bit(self.keys, i);
                if pressed {
                    self.register &= !(1 << i);
                } else {
                    self.register |= 1 << i;
                }
            }
        }
        // Update buttons
        if !check_bit(self.register, 5) {
            for i in 0..=3 {
                let pressed = check_bit(self.keys, 4 + i);
                if pressed {
                    self.register &= !(1 << i);
                } else {
                    self.register |= 1 << i;
                }
            }
        }
    }
}

fn get_button_bit(btn: Button) -> u8 {
    match btn {
        Button::Down => 0,
        Button::Up => 1,
        Button::Left => 2,
        Button::Right => 3,

        Button::Start => 4,
        Button::Select => 5,
        Button::B => 6,
        Button::A => 7,
    }
}
