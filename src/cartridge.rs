use crate::memory_map::*;

#[allow(non_camel_case_types)]
#[derive(PartialEq, Clone, Copy)]
enum MemoryModel {
    ROM16M_RAM8K,
    ROM4M_RAM32K,
}

pub struct Cartridge {
    rom: Vec<u8>,
    ram_bank: Vec<u8>,

    rom_bank_nr: u8,
    ram_bank_nr: u8,
    memory_model: MemoryModel,
    ram_bank_write_enable: bool,
}

impl Cartridge {
    pub fn new(rom: Vec<u8>) -> Self {
        Cartridge {
            rom: rom,
            // TODO: generate ram bank from rom information instead
            ram_bank: vec![0; SWITCH_RAM_BANK_LENGTH as usize * 16],
            rom_bank_nr: 0,
            ram_bank_nr: 0,
            memory_model: MemoryModel::ROM16M_RAM8K,
            ram_bank_write_enable: false,
        }
    }
    pub fn read_mem(&self, address: u16) -> Option<u8> {
        match address {
            ROM_BANK0_START..ROM_BANK0_END => {
                Some(self.rom[address as usize - ROM_BANK0_START as usize])
            }
            SWITCH_ROM_BANK_START..SWITCH_ROM_BANK_END => {
                let mut bank_nr = self.rom_bank_nr;
                if bank_nr == 0 {
                    bank_nr = 1;
                }
                println!("rom_nr: {}", self.rom_bank_nr);
                let start_address = bank_nr as usize * SWITCH_ROM_BANK_LENGTH as usize;
                Some(self.rom[start_address + (address - SWITCH_ROM_BANK_START) as usize])
            }

            SWITCH_RAM_BANK_START..SWITCH_RAM_BANK_END => {
                let start_address = self.ram_bank_nr as usize * SWITCH_RAM_BANK_LENGTH as usize;
                Some(self.ram_bank[start_address + (address - SWITCH_RAM_BANK_START) as usize])
            }
            _ => None,
        }
    }

    // Returns true if the write was handled. False otherwise
    pub fn write_mem(&mut self, address: u16, value: u8) -> bool {
        match address {
            CHOOSE_MEMORY_MODE_START..CHOOSE_MEMORY_MODE_END => {
                let value = value & 0b1;
                if value == 1 {
                    self.memory_model = MemoryModel::ROM4M_RAM32K;
                } else {
                    self.memory_model = MemoryModel::ROM16M_RAM8K;
                }
            }

            ENABLE_RAM_BANK_START..ENABLE_RAM_BANK_END => {
                self.ram_bank_write_enable = value == 0xA;
            }
            CHOOSE_ROM_BANK_START..CHOOSE_ROM_BANK_END => {
                // 0 means 1 in choosing rom bank
                let mut value = if value == 0 { 1 } else { value };
                value &= 0b0001_1111;
                self.rom_bank_nr = value;
            }
            CHOOSE_RAM_BANK_START..CHOOSE_RAM_BANK_END => {
                self.ram_bank_nr = value & 0b11;
                // TODO: handle 16/8 mode somehow
            }

            SWITCH_RAM_BANK_START..SWITCH_RAM_BANK_END => {
                self.ram_bank[self.ram_bank_nr as usize * SWITCH_RAM_BANK_LENGTH as usize
                    + (address - SWITCH_RAM_BANK_START) as usize] = value;
            }
            _ => return false,
        }
        true
    }
}
