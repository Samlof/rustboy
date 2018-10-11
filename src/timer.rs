use super::CPU_SPEED;
use crate::utils::check_bit;

const DIV_PER_FRAME: u64 = 16384;
const DIV_CLOCK_PER_CPU: u64 = CPU_SPEED / DIV_PER_FRAME;

pub struct Timer {
    main: u64,
    sub: u64,
    cl_div: u64,

    div: u8,
    tima: u8,
    tma: u8,
    tac: u8,

    div_counter: u64,
    tima_counter: u64,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            main: 0,
            sub: 0,
            cl_div: 0,

            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,

            div_counter: 0,
            tima_counter: 0,
        }
    }

    pub fn write(&mut self, address: u16, value: u8) -> bool {
        match address {
            0xFF04 => {
                self.div = 0;
            }
            0xFF05 => {
                self.tima = value;
            }
            0xFF06 => {
                self.tma = value;
            }
            0xFF07 => {
                self.tac = value;
            }
            _ => return false,
        }
        return true;
    }

    pub fn read(&self, address: u16) -> Option<u8> {
        match address {
            0xFF04 => Some(self.div),
            0xFF05 => Some(self.tima),
            0xFF06 => Some(self.tma),
            0xFF07 => Some(self.tac),
            _ => None,
        }
    }

    pub fn update(&mut self) -> bool {
        self.sub += 1;

        if self.sub >= 16 {
            self.main += 1;
            self.sub -= 16;

            // Handle div
            self.cl_div += 1;
            if self.cl_div == 16 {
                self.cl_div = 0;
                self.div = self.div.wrapping_add(1);
            }
        }
        if !self.timer_enabled() {
            return false;
        }

        // Handle tima
        if self.main >= self.timer_clock() {
            self.main = 0;
            if self.tima == 0xFF {
                self.tima = self.tma;
                return true;
            }
            self.tima += 1;
        }

        false
    }

    fn timer_enabled(&self) -> bool {
        check_bit(self.tac, 2)
    }

    fn timer_clock(&self) -> u64 {
        match self.tac & 0b11 {
            0 => 64,
            1 => 1,
            2 => 4,
            3 => 16,
            _ => unreachable!(),
        }
    }
}
