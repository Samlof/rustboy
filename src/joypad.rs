use super::utils::check_bit;

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
    keys: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad { keys: 0xFF }
    }
    pub fn read(&self, address: u16) -> Option<u8> {
        match address {
            0xFF00 => Some(self.keys),
            _ => None,
        }
    }

    // Returns true if the write was handled. False otherwise
    pub fn write(&mut self, address: u16, value: u8) -> bool {
        match address {
            0xFF00 => {
                // First clear the upper 4 bits
                self.keys &= 0x0F;
                // Then write them
                self.keys |= value & 0xF0
            }
            _ => return false,
        }
        true
    }

    pub fn update_button(&mut self, btn: Button, pressed: bool) {
        let bit = get_button_bit(self.button_mode(), btn);
        if let Some(bit) = bit {
            if pressed {
                // Set the pressed button's bit to 0
                self.keys &= !(1 << bit);
            } else {
                // Set the released button's bit to 1
                self.keys |= (1 << bit);
            }
        }
    }

    fn button_mode(&self) -> Mode {
        if !check_bit(self.keys, 5) {
            return Mode::Buttons;
        }
        if !check_bit(self.keys, 4) {
            return Mode::Directions;
        }
        Mode::None
    }
}

fn get_button_bit(mode: Mode, btn: Button) -> Option<u8> {
    match mode {
        Mode::Buttons => match btn {
            Button::Start => Some(3),
            Button::Select => Some(2),
            Button::B => Some(1),
            Button::A => Some(0),
            _ => None,
        },
        Mode::Directions => match btn {
            Button::Down => Some(3),
            Button::Up => Some(2),
            Button::Left => Some(1),
            Button::Right => Some(0),
            _ => None,
        },
        Mode::None => None,
    }
}
