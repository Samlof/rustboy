use super::ppu::*;
use super::sound_subsystem::*;
use super::timer::*;
use crate::memory_map::*;

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
                panic!("Write to IO port. Not implemented: {:04x}", address)
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
            _ => panic!(
                "Interconnect: Can't write memory address: 0x{:04x}",
                address
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
                panic!("Read to unknown IO port: {:04x}", address)
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
            _ => panic!("Interconnect: Can't read memory address: 0x{:04x}", address),
        }
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
