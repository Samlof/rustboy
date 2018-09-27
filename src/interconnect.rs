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

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Clone, Copy)]
enum MemoryModel {
    ROM16M_RAM8K,
    ROM4M_RAM32K,
}

#[derive(Debug)]
pub struct Interconnect {
    rom: Vec<u8>,
    boot: Vec<u8>,

    pub internal_ram2: Box<[u8]>,
    ram_bank: Box<[u8]>,
    internal_ram: Box<[u8]>,

    rom_bank_nr: u8,
    ram_bank_nr: u8,
    memory_model: MemoryModel,

    pub ppu: Ppu,
    sound: SoundSubsystem,
    timer: Timer,

    interrupt_flag: u8,
    interrupt_enable: u8,

    ram_bank_write_enable: bool,

    booting: bool,
}

impl Interconnect {
    pub fn new(boot: Vec<u8>, mut rom: Vec<u8>) -> Self {
        Interconnect {
            rom,
            boot,
            internal_ram2: vec![0; INTERNAL_RAM2_LENGTH as usize].into_boxed_slice(),
            ram_bank: vec![0; SWITCH_RAM_BANK_LENGTH as usize * 16].into_boxed_slice(),
            internal_ram: vec![0; INTERNAL_RAM_LENGTH as usize].into_boxed_slice(),
            rom_bank_nr: 1,
            ppu: Ppu::new(),
            sound: SoundSubsystem::new(),
            timer: Timer::new(),
            memory_model: MemoryModel::ROM16M_RAM8K,
            interrupt_flag: 0,
            interrupt_enable: 0,
            ram_bank_write_enable: true,
            booting: true,
            ram_bank_nr: 0,
        }
    }

    pub fn write_mem(&mut self, address: u16, value: u8) {
        // Find out where the address points
        match address {
            ENABLE_RAM_BANK_START..ENABLE_RAM_BANK_END => {
                self.ram_bank_write_enable = value == 0xA;
            }
            CHOOSE_MEMORY_MODE_START..CHOOSE_MEMORY_MODE_END => {
                let value = value & 0b1;
                if value == 1 {
                    self.memory_model = MemoryModel::ROM4M_RAM32K;
                } else {
                    self.memory_model = MemoryModel::ROM16M_RAM8K;
                }
            }
            CHOOSE_ROM_BANK_START..CHOOSE_ROM_BANK_END => {
                let mut value = if value == 0 { 1 } else { value };
                value &= 0b0001_1111;
                self.rom_bank_nr = value;
            }
            CHOOSE_RAM_BANK_START..CHOOSE_RAM_BANK_END => {
                self.ram_bank_nr = value & 0b11;
                // TODO: handle 16/8 mode somehow
            }
            0xFF50 => {
                if !self.booting {}
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
                    0xFF01 => {
                        println!("Can't send serial data!");
                    }
                    0xFF02 => {
                        if value >= 0b1000_0000 {
                            println!(
                                "Write to serial port: addr: 0x{:04x}, 0x{:02x}",
                                address, value
                            );
                        }
                    }
                    _ => println!(
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
                self.ram_bank[self.ram_bank_nr as usize * SWITCH_RAM_BANK_LENGTH as usize
                    + (address - SWITCH_RAM_BANK_START) as usize] = value;
            }
            SPRITE_MEM_START..SPRITE_MEM_END => {
                self.ppu.write_sprite_mem(address, value);
            }
            INTERRUPT_REGISTER => self.interrupt_enable = value,
            0xFEA0...0xFEFF => println!("Not usable area"),
            IO_PORTS_END...0xFF7F => println!("No idea what's here"),
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
                    _ => {
                        println!("Read to unknown IO port: {:04x}", address);
                        0xFF
                    }
                }
            }
            ROM_BANK0_START..ROM_BANK0_END => *self
                .rom
                .get((address as usize - ROM_BANK0_START as usize))
                .expect(&format!(
                    "Read out of rom range: 0x{:04x} of 0x{:04x}",
                    address,
                    self.rom.len()
                )),

            SWITCH_ROM_BANK_START..SWITCH_ROM_BANK_END => *self
                .rom
                .get(self.rom_bank_nr as usize * 16000 + (address - SWITCH_ROM_BANK_START) as usize)
                .expect(&format!(
                    "Read out of rom range: 0x{:04x} of 0x{:04x}",
                    address,
                    self.rom.len()
                )),

            INTERNAL_RAM_START..INTERNAL_RAM_END => {
                self.internal_ram[(address - INTERNAL_RAM_START) as usize]
            }
            ECHO_RAM_START..ECHO_RAM_END => self.internal_ram[(address - ECHO_RAM_START) as usize],
            INTERNAL_RAM2_START..INTERNAL_RAM2_END => {
                self.internal_ram2[(address - INTERNAL_RAM2_START) as usize]
            }
            SWITCH_RAM_BANK_START..SWITCH_RAM_BANK_END => {
                self.ram_bank[self.ram_bank_nr as usize * SWITCH_RAM_BANK_LENGTH as usize
                    + (address - SWITCH_RAM_BANK_START) as usize]
            }
            SPRITE_MEM_START..SPRITE_MEM_END => self.ppu.read_sprite_mem(address),
            INTERRUPT_REGISTER => {
                self.interrupt_enable;
                0xFF
            }
            0xFEA0...0xFEFF => {
                println!("Not usable area");
                0xFF
            }
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
        if self.ppu.update() {
            self.interrupt_flag |= 1;
        }
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
