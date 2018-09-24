#[allow(non_snake_case)]
#[derive(Debug)]
pub struct Timer {
    NR11: u8,
}

impl Timer {
    pub fn new() -> Self {
        Timer { NR11: 0 }
    }

    pub fn write(&mut self, address: u16, value: u8) -> bool {
        match address {
            0xFF07 => {
                // Frequency variable

            }
            _ => return false,
        }
        return true;
    }

    pub fn read(&self, address: u16) -> Option<u8> {
        match address {
            0xFF07 => Some(self.NR11),
            _ => None,
        }
    }
}
