use super::ppu::*;
use super::sound_subsystem::*;
use super::timer::*;
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
#[derive(Debug)]
pub struct Interconnect {
    rom: Vec<u8>,
    boot: Vec<u8>,

    pub internal_ram2: Box<[u8]>,
    switchable_ram_bank: Box<[u8]>,
    internal_ram: Box<[u8]>,

    pub ppu: Ppu,
    sound: SoundSubsystem,
    timer: Timer,

    interrupt_flag: u8,
    interrupt_enable: u8,

    booting: bool,
}

impl Interconnect {
    pub fn new(boot: Vec<u8>, mut rom: Vec<u8>) -> Self {
        Interconnect {
            rom,
            boot,
            internal_ram2: vec![0; INTERNAL_RAM2_LENGTH as usize].into_boxed_slice(),
            switchable_ram_bank: vec![0; SWITCH_RAM_BANK_LENGTH as usize].into_boxed_slice(),
            internal_ram: vec![0; INTERNAL_RAM_LENGTH as usize].into_boxed_slice(),
            ppu: Ppu::new(),
            sound: SoundSubsystem::new(),
            timer: Timer::new(),
            interrupt_flag: 0,
            interrupt_enable: 0,
            booting: true,
        }
    }

    pub fn write_mem(&mut self, address: u16, value: u8) {
        // Find out where the address points
        match address {
            0xFF50 => {
                if !self.booting {
                    panic!(
                        "Already stopped booting. write {:02x} to {:04x}",
                        value, address
                    );
                }
                // Stop boot mode
                self.booting = false;
            }
            VRAM_START..VRAM_END => self.ppu.write_vram(address, value),
            IO_PORTS_START..IO_PORTS_END => {
                if self.ppu.write(address, value) {
                    return;
                }
                if self.sound.write(address, value) {
                    return;
                }
                if self.timer.write(address, value) {
                    return;
                }

                match address {
                    0xFF0F => self.interrupt_flag = value,
                    _ => panic!(
                        "Write to IO port. Not implemented: 0x{:04x}, val: 0x{:02x}",
                        address, value
                    ),
                }
            }
            INTERNAL_RAM_START..INTERNAL_RAM_END => {
                self.internal_ram[(address - INTERNAL_RAM_START) as usize] = value;
            }
            ECHO_RAM_START..ECHO_RAM_END => {
                self.internal_ram[(address - ECHO_RAM_START) as usize] = value;
            }
            INTERNAL_RAM2_START..INTERNAL_RAM2_END => {
                self.internal_ram2[(address - INTERNAL_RAM2_START) as usize] = value;
            }
            SWITCH_RAM_BANK_START..SWITCH_RAM_BANK_END => {
                self.switchable_ram_bank[(address - SWITCH_RAM_BANK_START) as usize] = value;
            }
            SPRITE_MEM_START..SPRITE_MEM_END => {
                self.ppu.write_sprite_mem(address, value);
            }
            INTERRUPT_REGISTER => self.interrupt_enable = value,
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
        // Find out where the address points
        match address {
            VRAM_START..VRAM_END => self.ppu.read_vram(address),
            IO_PORTS_START..IO_PORTS_END => {
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
                match address {
                    0xFF0F => self.interrupt_flag,
                    _ => panic!("Read to unknown IO port: {:04x}", address),
                }
            }
            ROM_BANK0_START..ROM_BANK0_END => {
                // TODO: for now just reads from rom. Bank switching and stuff later
                *self
                    .rom
                    .get((address - ROM_BANK0_START) as usize)
                    .expect(&format!(
                        "Read out of rom range: 0x{:04x} of 0x{:04x}",
                        address,
                        self.rom.len()
                    ))
            }

            SWITCH_ROM_BANK_START..SWITCH_ROM_BANK_END => {
                // TODO: for now just reads from rom. Bank switching and stuff later
                *self
                    .rom
                    .get((address - ROM_BANK0_START) as usize)
                    .expect(&format!(
                        "Read out of rom range: 0x{:04x} of 0x{:04x}",
                        address,
                        self.rom.len()
                    ))
            }

            INTERNAL_RAM_START..INTERNAL_RAM_END => {
                self.internal_ram[(address - INTERNAL_RAM_START) as usize]
            }
            ECHO_RAM_START..ECHO_RAM_END => self.internal_ram[(address - ECHO_RAM_START) as usize],
            INTERNAL_RAM2_START..INTERNAL_RAM2_END => {
                self.internal_ram2[(address - INTERNAL_RAM2_START) as usize]
            }
            SWITCH_RAM_BANK_START..SWITCH_RAM_BANK_END => {
                self.switchable_ram_bank[(address - SWITCH_RAM_BANK_START) as usize]
            }
            SPRITE_MEM_START..SPRITE_MEM_END => self.ppu.read_sprite_mem(address),
            INTERRUPT_REGISTER => self.interrupt_enable,
            _ => panic!("Interconnect: Can't read memory address: 0x{:04x}", address),
        }
    }

    pub fn get_interrupt(&mut self) -> Option<Interrupt> {
        for i in 0..=4 {
            if check_bit(self.interrupt_flag, i) && check_bit(self.interrupt_enable, i) {
                // Asking for an interrupt means cpu will take it.
                // So reset the interrupt flag
                self.interrupt_flag &= !(1 << i);
                // From_u8 already returns an option. However if something breaks this'll panic then
                return Some(Interrupt::from_u8(i).unwrap());
            }
        }
        None
    }

    pub fn update(&mut self) {
        self.ppu.update();
    }

    pub fn rom(&self) -> &Vec<u8> {
        &self.rom
    }
    pub fn boot(&self) -> &Vec<u8> {
        &self.boot
    }
}

#[inline(always)]
fn check_bit(val: u8, b: u8) -> bool {
    val & (1 << b) > 0
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
