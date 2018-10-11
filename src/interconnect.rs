use super::cartridge::*;
use super::joypad::*;
use super::ppu::*;
use super::sound_subsystem::*;
use super::timer::*;
use super::utils::check_bit;
use crate::memory_map::*;
use enum_primitive_derive::*;
use num_traits::{FromPrimitive, ToPrimitive};

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy, Primitive)]
// The value is interrupt priority
pub enum Interrupt {
    VBLANK = 0,
    LCDStatus = 1,
    TimerOverflow = 2,
    SerialTransfer = 3,
    Joypad = 4,
}

pub struct Interconnect {
    boot: Vec<u8>,
    cartridge: Cartridge,

    pub internal_ram2: Box<[u8]>,
    internal_ram: Box<[u8]>,

    pub ppu: Ppu,
    sound: SoundSubsystem,
    timer: Timer,
    joypad: Joypad,

    interrupt_flag: u8,
    interrupt_enable: u8,

    booting: bool,
}

impl Interconnect {
    pub fn new(boot: Vec<u8>, mut cartridge: Cartridge) -> Self {
        Interconnect {
            cartridge,
            boot,
            internal_ram2: vec![0; INTERNAL_RAM2_LENGTH as usize].into_boxed_slice(),
            internal_ram: vec![0; INTERNAL_RAM_LENGTH as usize].into_boxed_slice(),
            ppu: Ppu::new(),
            sound: SoundSubsystem::new(),
            timer: Timer::new(),
            joypad: Joypad::new(),
            interrupt_flag: 0,
            interrupt_enable: 0,
            booting: true,
        }
    }

    pub fn write_mem(&mut self, address: u16, value: u8) {
        if self.cartridge.write_mem(address, value) {
            return;
        }
        match address {
            0xFF50 => {
                // Stop boot mode
                self.booting = false;
            }
            VRAM_START..VRAM_END => self.ppu.write_vram(address, value),
            IO_PORTS_START..IO_PORTS_END => self.io_port_write(address, value),
            INTERNAL_RAM_START..INTERNAL_RAM_END => {
                self.internal_ram[(address - INTERNAL_RAM_START) as usize] = value;
            }
            ECHO_RAM_START..ECHO_RAM_END => {
                self.internal_ram[(address - ECHO_RAM_START) as usize] = value;
            }
            INTERNAL_RAM2_START..INTERNAL_RAM2_END => {
                self.internal_ram2[(address - INTERNAL_RAM2_START) as usize] = value;
            }
            SPRITE_MEM_START..SPRITE_MEM_END => {
                if value > 0 {
                    panic!("Sprite area: 0x{:04x}, value: 0x{:02x}", address, value);
                }
                self.ppu.write_sprite_mem(address, value);
            }
            INTERRUPT_REGISTER => self.interrupt_enable = value,
            0xFEA0...0xFEFF => println!(
                "Write to not usable area: 0x{:04x}, value: 0x{:02x}",
                address, value
            ),
            IO_PORTS_END...0xFF7F => println!(
                "Write to unknown area: 0x{:04x}, value: 0x{:02x}",
                address, value
            ),
            _ => panic!(
                "Interconnect: Can't write memory address: 0x{:04x}, value: 0x{:02x}",
                address, value
            ),
        }
    }

    pub fn read_mem(&self, address: u16) -> u8 {
        if self.booting && address <= 0xFF {
            return self.boot[address as usize];
        }
        if let Some(value) = self.cartridge.read_mem(address) {
            return value;
        }
        // Find out where the address points
        match address {
            VRAM_START..VRAM_END => self.ppu.read_vram(address),
            IO_PORTS_START..IO_PORTS_END => self.io_port_read(address),
            INTERNAL_RAM_START..INTERNAL_RAM_END => {
                self.internal_ram[(address - INTERNAL_RAM_START) as usize]
            }
            ECHO_RAM_START..ECHO_RAM_END => self.internal_ram[(address - ECHO_RAM_START) as usize],
            INTERNAL_RAM2_START..INTERNAL_RAM2_END => {
                self.internal_ram2[(address - INTERNAL_RAM2_START) as usize]
            }
            SPRITE_MEM_START..SPRITE_MEM_END => self.ppu.read_sprite_mem(address),
            INTERRUPT_REGISTER => self.interrupt_enable,
            0xFEA0...0xFEFF => {
                println!("Read to not usable area: 0x{:04x}", address);
                0xFF
            }
            0xFF4C..0xFF80 => {
                println!("Read to not usable area: 0x{:04x}", address);
                0xFF
            }
            _ => panic!("Interconnect: Can't read memory address: 0x{:04x}", address),
        }
    }

    fn io_port_read(&self, address: u16) -> u8 {
        if address == 0xFF46 {
            // DMA address. Only write
            return 0xFF;
        }
        let res = self.ppu.read(address);
        if let Some(ret) = res {
            return ret;
        }
        let res = self.sound.read(address);
        if let Some(ret) = res {
            return ret;
        }
        let res = self.timer.read(address);
        if let Some(ret) = res {
            return ret;
        }
        let res = self.joypad.read(address);
        if let Some(ret) = res {
            return ret;
        }
        match address {
            0xFF0F => self.interrupt_flag,
            _ => {
                println!("Read to unknown IO port: {:04x}", address);
                0xFF
            }
        }
    }

    fn io_port_write(&mut self, address: u16, value: u8) {
        if address == 0xFF46 {
            // Start dma
            let start_add = value << 2;
            for i in 0..=0x9F {
                let val = self.read_mem(start_add as u16 + i);
                self.ppu.write_sprite_mem(0xFE00 + i, val);
            }
            self.ppu.add_cycles(200);
            return;
        }
        if self.ppu.write(address, value) {
            return;
        }
        if self.sound.write(address, value) {
            return;
        }
        if self.timer.write(address, value) {
            return;
        }
        if self.joypad.write(address, value) {
            return;
        }
        match address {
            0xFF0F => self.interrupt_flag = value,
            0xFF01 => {
                //println!("Can't send serial data!");
            }
            0xFF02 => {
                if value >= 0b1000_0000 {
                    //println!(
                    //    "Write to serial port: addr: 0x{:04x}, 0x{:02x}",
                    //    address, value
                    //);
                }
            }
            _ => println!(
                "Write to IO port. Not implemented: 0x{:04x}, val: 0x{:02x}",
                address, value
            ),
        }
    }

    pub fn get_interrupt(&mut self) -> Option<Interrupt> {
        for i in 0..=4 {
            if check_bit(self.interrupt_flag, i) && check_bit(self.interrupt_enable, i) {
                // Asking for an interrupt means cpu will take it.
                // So reset the interrupt flag here, instead of after a call from cpu
                self.interrupt_flag &= !(1 << i);
                // From_u8 already returns an option. However if something breaks this'll panic then
                return Some(Interrupt::from_u8(i).unwrap());
            }
        }
        None
    }

    pub fn get_button_interrupt(&mut self) -> Option<Interrupt> {
        if check_bit(self.interrupt_flag, 4) {
            // Clear the interrupt
            self.interrupt_flag &= !(1 << 4);
            Some(Interrupt::Joypad)
        } else {
            None
        }
    }

    pub fn update(&mut self) {
        if self.ppu.update() {
            // vblank interrupt
            self.interrupt_flag |= 1;
            // Update joypad
            if self.joypad.update(&self.ppu.main_window) {
                // joypad interrupt
                self.interrupt_flag |= 1 << 4;
            }
        }
    }

    pub fn boot(&self) -> &Vec<u8> {
        &self.boot
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_bit() {
        assert!(check_bit(0b0100_0000, 6));
        assert!(check_bit(0b0000_1000, 3));
        assert!(check_bit(0b0100_0001, 0));
        assert!(!check_bit(0b0100_0001, 3));
        assert!(!check_bit(0b0100_0001, 7));
    }
}
