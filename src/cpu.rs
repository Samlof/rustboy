use super::instruction;
use super::instruction::{CB_Instruction, Instruction};
use super::interconnect::*;
use super::ppu::Color;

// Clock Speed: 4.194304 MHz
const CPU_FREQ: f32 = 4.194304;

#[derive(Debug, PartialEq)]
enum InterruptState {
    Enabled,
    EnableNext,
    Disabled,
    DisableNext,
}
#[derive(Debug, PartialEq)]
enum CpuState {
    On,
    OffUntilInterrupt,
    OffUntilButtonPress,
}
#[derive(Debug)]
pub struct Cpu {
    reg_a: u8,
    reg_b: u8,
    reg_c: u8,
    reg_d: u8,
    reg_e: u8,
    reg_f: u8,
    reg_h: u8,
    reg_l: u8,

    reg_sp: u16,
    reg_pc: u16,

    flag_z: bool,
    flag_n: bool,
    /*This bit is set if a carry occurred from the lower
        nibble in the last math operation.
    */
    flag_h: bool,
    /*This bit is set if a carry occurred from the last
        math operation or if register A is the smaller value
        when executing the CP instruction.
    */
    flag_c: bool,

    pub interconnect: Interconnect,
    cycles: i32,
    cpu_state: CpuState,
    interrupt_state: InterruptState,
}

impl Cpu {
    pub fn new(interconnect: Interconnect) -> Self {
        Cpu {
            reg_a: 0,
            reg_b: 0,
            reg_c: 0,
            reg_d: 0,
            reg_e: 0,
            reg_f: 0,
            reg_h: 0,
            reg_l: 0,
            reg_sp: 0,
            reg_pc: 0, // Put boot operation to go from here. As it's unused

            flag_z: false, // Zero flag
            flag_n: false, // Subtract flag
            flag_h: false, // Half Carry flag
            flag_c: false, // Carry flag

            interconnect,
            cycles: 0,
            cpu_state: CpuState::On,
            interrupt_state: InterruptState::Enabled,
        }
    }

    pub fn step(&mut self) {
        // Hand over to interconnect first. So ppu updates
        self.interconnect.update();

        // If cycles to burn, just return
        if self.cycles > 0 {
            self.cycles -= 4;
            return;
        }

        if self.interrupt_state != InterruptState::Disabled {
            self.handle_interrupts();
        }
        if self.cpu_state != CpuState::On {
            return;
        }

        self.do_next_instrution();

        if self.interrupt_state == InterruptState::DisableNext {
            self.interrupt_state = InterruptState::Disabled;
        }
        if self.interrupt_state == InterruptState::EnableNext {
            self.interrupt_state = InterruptState::Enabled;
        }
    }

    fn handle_interrupts(&mut self) {
        // TODO:
    }

    fn do_next_instrution(&mut self) {
        let opcode = self.read_byte();
        let inst = instruction::parse(opcode).expect(&format!(
            "0x{:04x} Unknown opcode: 0x{:02x}",
            self.reg_pc - 1,
            opcode
        ));
        if false {
            if opcode != 0xCB {
                println!(
                    "0x{:04x} op: 0x{:02x} inst: '{:?}'",
                    self.reg_pc - 1,
                    opcode,
                    inst
                );
            }
        }
        self.execute_instruction(opcode, inst);
    }

    fn execute_instruction(&mut self, opcode: u8, inst: Instruction) {
        match inst {
            Instruction::LD_nn_n => {
                let value = self.read_byte();

                let reg = (opcode - 6) / 8;
                self.set_reg_r(reg, value);

                self.add_cycles(8);
            }
            Instruction::LD_n_nn => {
                // LD n,nn
                let value = u8s_as_u16(self.read_nn());
                // Match to see which registers to use
                match opcode {
                    0x01 => self.set_bc(value),
                    0x11 => self.set_de(value),
                    0x21 => self.set_hl(value),
                    0x31 => self.reg_sp = value,
                    _ => unreachable!(),
                };

                self.add_cycles(12);
            }
            Instruction::LD_r1_r2 => {
                let reg1 = (opcode - 0x40) / 8;
                let reg2 = (opcode - 0x40) % 8;
                let value = if reg2 == 6 {
                    self.add_cycles(8);
                    self.read_mem(self.hl())
                } else {
                    self.add_cycles(4);
                    self.read_reg_r(reg2)
                };
                println!("LD_r1_r2 op: {:02x}", opcode);
                self.set_reg_r(reg1, value);
            }
            Instruction::LD_HL_ptr_r2 => {
                let value = if opcode == 0x36 {
                    self.add_cycles(12);
                    self.read_byte()
                } else {
                    self.add_cycles(8);
                    let reg = opcode - 0x70;
                    self.read_reg_r(reg)
                };
                self.write_mem(self.hl(), value);
            }
            Instruction::ADD_A_n => {
                let reg = opcode - 0x80;
                let value = if opcode == 0xC6 {
                    self.add_cycles(8);
                    self.read_byte()
                } else if reg == 6 {
                    self.add_cycles(8);
                    self.read_mem(self.hl())
                } else {
                    self.add_cycles(4);
                    self.read_reg_r(reg)
                };
                let old_value = self.reg_a;
                self.reg_a = old_value.wrapping_add(value);

                self.flag_z = self.reg_a == 0;
                self.flag_n = false;
                // H - Set if carry from bit 3.
                self.flag_h = (self.reg_a & 0xF) >= (old_value & 0xF);
                // C - Set if carry from bit 7. So meaning if it overflowed
                self.flag_c = self.reg_a < old_value;
            }
            Instruction::SUB_n => {
                let reg = opcode - 0x90;
                let n = if opcode == 0xD6 {
                    self.add_cycles(8);
                    self.read_byte()
                } else if reg == 6 {
                    self.add_cycles(8);
                    self.read_mem(self.hl())
                } else {
                    self.add_cycles(4);
                    self.read_reg_r(reg)
                };
                let old_value = self.reg_a;
                self.reg_a = old_value.wrapping_sub(n);

                self.flag_z = self.reg_a == 0;
                self.flag_n == true;

                // H - Set if no borrow from bit 4.
                self.flag_h = (self.reg_a & 0xF) <= (old_value & 0xF);
                // C - Set if no borrow.
                self.flag_c = self.reg_a < old_value;
            }
            Instruction::SBC_A_n => {
                let reg = opcode - 0x98;
                let value = if opcode == 0xDE {
                    self.add_cycles(8);
                    self.read_byte()
                } else if reg == 6 {
                    self.add_cycles(8);
                    self.read_mem(self.hl())
                } else {
                    self.add_cycles(4);
                    self.read_reg_r(reg)
                };
                let old_value = self.reg_a;
                self.reg_a = old_value.wrapping_sub(value);
                // Add carry flag
                if self.flag_c {
                    self.reg_a.wrapping_add(1 << 7);
                }

                self.flag_z = self.reg_a == 0;
                self.flag_n == true;

                // H - Set if no borrow from bit 3.
                self.flag_h = (self.reg_a & 0xF) <= (old_value & 0xF);
                // C - Set if no borrow.
                self.flag_c = self.reg_a < old_value;
            }
            Instruction::AND_n => {
                let reg = opcode - 0xA0;
                let value = if opcode == 0xE6 {
                    self.add_cycles(8);
                    self.read_byte()
                } else if reg == 6 {
                    self.add_cycles(8);
                    self.read_mem(self.hl())
                } else {
                    self.add_cycles(4);
                    self.read_reg_r(reg)
                };

                self.reg_a = self.reg_a & value;

                self.flag_z = self.reg_a == 0;
                self.flag_n = false;
                self.flag_h = true;
                self.flag_c = false;
            }
            Instruction::OR_n => {
                let reg = opcode - 0xB0;
                let value = if opcode == 0xF6 {
                    self.add_cycles(8);
                    self.read_byte()
                } else if reg == 6 {
                    self.add_cycles(8);
                    self.read_mem(self.hl())
                } else {
                    self.add_cycles(4);
                    self.read_reg_r(reg)
                };

                self.reg_a = self.reg_a | value;

                self.flag_z = self.reg_a == 0;
                self.flag_n = false;
                self.flag_h = false;
                self.flag_c = false;
            }
            Instruction::NOP => self.add_cycles(4),
            Instruction::CALL_nn => {
                let address = u8s_as_u16(self.read_nn());

                self.push_stack_u16(self.reg_pc);

                self.reg_pc = address;
                self.add_cycles(12);
            }
            Instruction::PUSH_nn => {
                let value = match opcode {
                    0xF5 => self.af(),
                    0xC5 => self.bc(),
                    0xD5 => self.de(),
                    0xE5 => self.hl(),
                    _ => unreachable!(),
                };
                self.push_stack_u16(value);

                self.add_cycles(16);
            }
            Instruction::CP_n => {
                // Read what to compare with
                let reg = (opcode - 0xB8);
                let n = if opcode == 0xFE {
                    self.add_cycles(8);
                    self.read_byte()
                } else if reg == 6 {
                    self.add_cycles(8);
                    self.read_mem(self.hl())
                } else {
                    self.add_cycles(4);
                    self.read_reg_r(reg)
                };
                // Do the comparing
                let value = self.reg_a.wrapping_sub(n);

                self.flag_z = self.reg_a == n;
                self.flag_n = true;

                // H - Set if no borrow from bit 3.
                self.flag_h = (value & 0xF) < (self.reg_a & 0xF);
                // C - Set if no borrow. or reg_a < n
                self.flag_c = self.reg_a < n;
            }
            Instruction::JP_nn => {
                self.reg_pc = u8s_as_u16(self.read_nn());
                self.add_cycles(12);
            }
            Instruction::JP_HLptr => {
                // Turn it around because little endian
                let second_part = self.read_mem(self.hl()) as u16;
                let first_part = self.read_mem(self.hl() + 1) as u16;
                self.reg_pc = (second_part << 8) | first_part;
                self.add_cycles(4);
            }
            Instruction::LD_A_n => match opcode {
                0x78...0x7F if opcode != 0x7E => {
                    let reg = opcode - 0x78;
                    self.reg_a = self.read_reg_r(reg);
                    self.add_cycles(4);
                }
                0x3E => {
                    self.reg_a = self.read_byte();
                    self.add_cycles(8);
                }
                0x0A => {
                    let address = self.bc();
                    self.reg_a = self.read_mem(address);
                    self.add_cycles(8);
                }
                0x1A => {
                    let address = self.de();
                    self.reg_a = self.read_mem(address);
                    self.add_cycles(8);
                }
                0x7E => {
                    let address = self.hl();
                    self.reg_a = self.read_mem(address);
                    self.add_cycles(8);
                }
                0xFA => {
                    let address = u8s_as_u16(self.read_nn());
                    self.reg_a = self.read_mem(address);
                    self.add_cycles(16);
                }
                _ => unreachable!(),
            },
            Instruction::LD_nnptr_SP => {
                let nn = u8s_as_u16(self.read_nn());
                // Little endian, so save the other part first
                self.write_mem(nn, (self.reg_sp & 0xFF) as u8);
                // Then the first 8 bits
                self.write_mem(nn + 1, (self.reg_sp >> 7) as u8);
                self.add_cycles(20);
            }
            Instruction::ADC_A_n => {
                let reg = opcode - 0x80;
                let value = if opcode == 0xC6 {
                    self.add_cycles(8);
                    self.read_byte()
                } else if reg == 6 {
                    self.add_cycles(8);
                    self.read_mem(self.hl())
                } else {
                    self.add_cycles(4);
                    self.read_reg_r(reg)
                };

                let old_value = self.reg_a;
                self.reg_a = old_value.wrapping_add(value);
                if self.flag_c {
                    // Add the carry flag
                    self.reg_a = self.reg_a.wrapping_add(1 << 7);
                }

                self.flag_z = self.reg_a == 0;
                self.flag_n = false;
                // H - Set if carry from bit 3.
                self.flag_h = (self.reg_a & 0xF) <= (old_value & 0xF);
                // C - set if carry from bit 15
                self.flag_c = self.reg_a < old_value;
            }
            Instruction::LDHL_SP_n => {
                // Sign extend the byte into u16
                let n = ((self.read_byte() as i8) as i16) as u16;
                self.set_hl(self.reg_sp + n);

                self.flag_z = false;
                self.flag_n = false;
                // FIXME: h and c set or reset according to operation
                self.flag_h = false;
                self.flag_c = false;

                self.add_cycles(12);
            }
            Instruction::LD_SP_HL => {
                self.reg_sp = self.hl();
                self.add_cycles(8);
            }
            Instruction::LDI_A_HLptr => {
                let hl = self.hl();
                self.reg_a = self.read_mem(hl);
                self.set_hl(hl + 1);
                self.add_cycles(8);
            }
            Instruction::LDD_A_HLptr => {
                let hl = self.hl();
                self.reg_a = self.read_mem(hl);
                self.set_hl(hl - 1);
                self.add_cycles(8);
            }
            Instruction::LD_A_Cptr => {
                self.reg_a = self.read_mem(0xFF00 + self.reg_c as u16);
                self.add_cycles(8);
            }
            Instruction::LD_Cptr_A => {
                self.write_mem(0xFF00 + self.reg_c as u16, self.reg_a);
                self.add_cycles(8);
            }
            Instruction::ADD_HL_n => {
                let n = match opcode {
                    0x09 => self.bc(),
                    0x19 => self.de(),
                    0x29 => self.hl(),
                    0x39 => self.reg_sp,
                    _ => unreachable!(),
                };
                let hl = self.hl();
                self.set_hl(hl.wrapping_add(n));

                self.flag_n = false;
                // H - Set if carry from bit 11.
                self.flag_h = (self.hl() & 0xFFF) < (hl & 0xFFF);
                // C - set if carry from bit 15
                self.flag_c = self.hl() < hl;

                self.add_cycles(8);
            }
            Instruction::ADD_SP_n => {
                let old_value = self.reg_sp;
                self.reg_sp = self
                    .reg_sp
                    // Sign extend the next byte
                    .wrapping_add(self.read_byte() as i8 as i16 as u16);

                self.flag_z = false;
                self.flag_n = false;

                // H - Set if carry from bit 11.
                self.flag_h = (self.reg_sp & 0xFFF) < (old_value & 0xFFF);
                // C - set if carry from bit 15
                self.flag_c = self.reg_sp < old_value;

                self.add_cycles(16);
            }
            Instruction::INC_n => {
                let reg = (opcode - 0x04) / 8;
                let value = if reg == 6 {
                    self.read_mem(self.hl())
                } else {
                    self.read_reg_r(reg)
                };
                let new_value = value.wrapping_add(1);
                self.flag_z = new_value == 0;
                self.flag_n = false;
                // Set flag h is carry from bit 3
                self.flag_h = (new_value & 0xF) < (value & 0xF);

                if reg == 6 {
                    self.write_mem(self.hl(), new_value);
                    self.add_cycles(12);
                } else {
                    self.set_reg_r(reg, new_value);
                    self.add_cycles(4);
                }
            }
            Instruction::LD_n_A => {
                // Load A into a register
                match opcode {
                    0x47 | 0x4F | 0x57 | 0x5F | 0x67 | 0x6F | 0x7F => {
                        self.set_reg_r((opcode - 0x47) / 8, self.reg_a);
                        self.add_cycles(4);
                    }
                    0x02 => {
                        self.write_mem(self.bc(), self.reg_a);
                        self.add_cycles(8);
                    }
                    0x12 => {
                        self.write_mem(self.de(), self.reg_a);
                        self.add_cycles(8);
                    }
                    0x77 => {
                        self.write_mem(self.hl(), self.reg_a);
                        self.add_cycles(8);
                    }
                    0xEA => {
                        let address = u8s_as_u16(self.read_nn());
                        self.write_mem(address, self.reg_a);
                        self.add_cycles(16);
                    }
                    _ => unreachable!(),
                }
            }
            Instruction::RLCA => {
                let bit7 = self.reg_a >> 7;
                self.reg_a <<= 1;

                self.flag_c = bit7 == 1;
                self.flag_z = self.reg_a == 0;
                self.flag_n = false;
                self.flag_h = false;

                self.add_cycles(4);
            }
            Instruction::RLA => {
                let bit7 = self.reg_a >> 7;
                self.reg_a <<= 1;
                self.reg_a |= self.flag_c as u8;

                self.flag_c = bit7 == 1;
                self.flag_z = self.reg_a == 0;
                self.flag_n = false;
                self.flag_h = false;

                self.add_cycles(4);
            }
            Instruction::RRCA => {
                let bit0 = self.reg_a & 0b1;
                self.reg_a >>= 1;
                self.reg_a |= bit0 << 7;

                self.flag_c = bit0 == 1;
                self.flag_z = self.reg_a == 0;
                self.flag_n = false;
                self.flag_h = false;

                self.add_cycles(4);
            }
            Instruction::RRA => {
                let bit0 = self.reg_a & 0b1;
                self.reg_a >>= 1;

                self.reg_a |= (self.flag_c as u8) << 7;

                self.flag_c = bit0 == 1;
                self.flag_z = self.reg_a == 0;
                self.flag_n = false;
                self.flag_h = false;

                self.add_cycles(4);
            }
            Instruction::HALT => {
                self.cpu_state = CpuState::OffUntilInterrupt;
                self.add_cycles(4);
            }
            Instruction::STOP => {
                // The followup byte should be 00
                let byte = self.read_byte();
                if byte != 0x00 {
                    return;
                }
                self.cpu_state = CpuState::OffUntilButtonPress;
                self.interconnect.ppu.turn_lcd_off();

                self.add_cycles(4);
            }
            Instruction::DEC_n => {
                let reg = (opcode - 0x05) / 8;
                let value = if reg == 6 {
                    self.read_mem(self.hl())
                } else {
                    self.read_reg_r(reg)
                };
                let new_value = value.wrapping_sub(1);

                self.flag_z = new_value == 0;
                self.flag_n = true;
                // H - Set if no borrow from bit 4.
                self.flag_h = (new_value & 0xF) < (value & 0xF);

                if reg == 6 {
                    self.write_mem(self.hl(), new_value);
                    self.add_cycles(12);
                } else {
                    self.set_reg_r(reg, new_value);
                    self.add_cycles(4);
                }
            }
            Instruction::DAA => {
                // https://ehaskins.com/2018-01-30%20Z80%20DAA/
                let value = self.reg_a;
                let mut correction = 0;
                if self.flag_h || (!self.flag_n && (value & 0xF) > 9) {
                    correction |= 0x6;
                    self.flag_c = false;
                }
                if self.flag_c || (!self.flag_n && value > 0x99) {
                    correction |= 0x60;
                    self.flag_c = true;
                }
                let correction = if self.flag_n {
                    // Negate the correction
                    0xFF - correction
                } else {
                    correction
                };
                self.reg_a = self.reg_a.wrapping_add(correction);

                self.flag_z = self.reg_a == 0;
                self.flag_h = false;

                self.add_cycles(4);
            }
            Instruction::CPL => {
                // Complement, so flip all bits
                self.reg_a = !self.reg_a;
                self.flag_n = true;
                self.flag_h = true;

                self.add_cycles(4);
            }
            Instruction::CCF => {
                self.flag_c = !self.flag_c;
                self.flag_n = false;
                self.flag_h = false;
                self.add_cycles(4);
            }
            Instruction::SCF => {
                self.flag_n = false;
                self.flag_h = false;
                self.flag_c = true;

                self.add_cycles(4);
            }
            Instruction::LDI_HLptr_A => {
                let address = self.hl();
                self.write_mem(address, self.reg_a);
                self.set_hl(address + 1);

                self.add_cycles(8);
            }
            Instruction::INC_nn => {
                match opcode {
                    0x03 => self.set_bc(self.bc() + 1),
                    0x13 => self.set_de(self.de() + 1),
                    0x23 => self.set_hl(self.hl() + 1),
                    0x33 => self.reg_sp += 1,
                    _ => unreachable!(),
                };
                self.add_cycles(8);
            }
            Instruction::DEC_nn => {
                match opcode {
                    0x0B => self.set_bc(self.bc() - 1),
                    0x1B => self.set_de(self.de() - 1),
                    0x2B => self.set_hl(self.hl() - 1),
                    0x3B => self.reg_sp -= 1,
                    _ => unreachable!(),
                };
                self.add_cycles(8);
            }
            Instruction::POP_nn => {
                let value = self.pop_stack_u16();
                match opcode {
                    0xF1 => self.set_af(value),
                    0xC1 => self.set_bc(value),
                    0xD1 => self.set_de(value),
                    0xE1 => self.set_hl(value),
                    _ => unreachable!(),
                }
                self.add_cycles(12);
            }
            Instruction::RET => {
                self.reg_pc = self.pop_stack_u16();
                self.add_cycles(8);
            }
            Instruction::RET_cc => {
                if opcode == 0xC0 && self.flag_z == false
                    || opcode == 0xC8 && self.flag_z
                    || opcode == 0xD0 && self.flag_c == false
                    || opcode == 0xD8 && self.flag_c
                {
                    self.reg_pc = self.pop_stack_u16();
                }
                self.add_cycles(8);
            }
            Instruction::DI => {
                self.interrupt_state = InterruptState::DisableNext;
                self.add_cycles(4);
            }
            Instruction::EI => {
                self.interrupt_state = InterruptState::EnableNext;
                self.add_cycles(4);
            }
            Instruction::CALL_cc_nn => {
                let address = u8s_as_u16(self.read_nn());
                if opcode == 0xC4 && self.flag_z == false
                    || opcode == 0xCC && self.flag_z
                    || opcode == 0xD4 && self.flag_c == false
                    || opcode == 0xDC && self.flag_c
                {
                    self.push_stack_u16(self.reg_pc);
                    self.reg_pc = address;
                }
                self.add_cycles(12);
            }
            Instruction::LDH_nptr_A => {
                let address = self.read_byte() as u16 | 0xFF00;
                self.write_mem(address, self.reg_a);

                self.add_cycles(12);
            }
            Instruction::XOR_n => {
                let reg = opcode - 0xA8;
                let value = if opcode == 0xEE {
                    self.add_cycles(8);
                    self.read_byte()
                } else if reg == 6 {
                    self.add_cycles(8);
                    self.read_mem(self.hl())
                } else {
                    self.add_cycles(4);
                    self.read_reg_r(reg)
                };

                // XOR  with A
                self.reg_a ^= value;
                // Set flags
                self.flag_z = self.reg_a == 0;
                self.flag_c = false;
                self.flag_h = false;
                self.flag_n = false;
            }
            Instruction::LDD_HLptr_A => {
                // Set A into address of HL. Then decrement hl
                self.write_mem(self.hl(), self.reg_a);
                self.set_hl(self.hl() - 1);

                self.add_cycles(8);
            }
            Instruction::LDH_A_nptr => {
                let address = 0xFF00 + self.read_byte() as u16;
                self.reg_a = self.read_mem(address);
                self.add_cycles(12);
            }
            Instruction::JR_cc_n => {
                // Add next value to current pc on condition
                let value = self.read_byte();
                let current_address = self.reg_pc as i16;
                self.add_cycles(8);
                // Jump conditions depending on opcode
                if opcode == 0x20 && !self.flag_z
                    || opcode == 0x28 && self.flag_z
                    || opcode == 0x30 && !self.flag_c
                    || opcode == 0x38 && self.flag_c
                {
                    self.reg_pc = (current_address + (value as i8 as i16)) as u16;
                }
            }
            Instruction::RST_n => {
                let address = opcode - 0xC7;
                self.push_stack_u16(self.reg_pc);
                self.reg_pc = address as u16;
                self.add_cycles(32);
            }
            Instruction::RETI => {
                let address = self.pop_stack_u16();
                self.reg_pc = address;
                self.interrupt_state = InterruptState::Enabled;
                self.add_cycles(8);
            }
            Instruction::JP_cc_nn => {
                let next_addr = u8s_as_u16(self.read_nn());
                self.add_cycles(12);
                // Jump conditions depending on opcode
                if opcode == 0xC2 && !self.flag_z
                    || opcode == 0xCA && self.flag_z
                    || opcode == 0xD2 && !self.flag_c
                    || opcode == 0xDA && self.flag_c
                {
                    self.reg_pc = next_addr;
                }
            }
            Instruction::JR_n => {
                let value = self.read_byte();
                let current_address = self.reg_pc as i16;
                self.reg_pc = (current_address + (value as i8 as i16)) as u16;
                self.add_cycles(8);
            }
            Instruction::CB => self.handle_cb_opcode(),

            _ => panic!("0x{:04x} Inst not implemented: {:?}", self.reg_pc - 1, inst),
        }
    }

    fn print_stack_size(&self) {
        let data = &self.interconnect.internal_ram2;
        use crate::memory_map::INTERNAL_RAM2_START;
        let sp = self.reg_sp.wrapping_sub(INTERNAL_RAM2_START) + 1;
        println!("stacksize: {}", data.len().wrapping_sub(sp as usize));
    }
    fn print_stack(&self) {
        let data = &self.interconnect.internal_ram2;
        println!("Printing stack..");
        use crate::memory_map::INTERNAL_RAM2_START;
        let sp = self.reg_sp - INTERNAL_RAM2_START + 1;
        println!("stacksize: {}", data.len() - sp as usize);
        for i in data.iter().skip(sp as usize) {
            println!("{:02x}", i);
        }
    }

    fn handle_cb_opcode(&mut self) {
        {
            // CB means a bit operation. Find out which one
            let opcode = self.read_byte();
            let inst = instruction::parse_cb(opcode).expect(&format!(
                "0x{:04x} Unknown CB opcode: 0x{:02x}",
                self.reg_pc - 2,
                opcode
            ));

            if false {
                println!(
                    "0x{:04x} op: 0xCB 0x{:02x} inst: '{:?}'",
                    self.reg_pc - 2,
                    opcode,
                    inst
                );
            }

            match inst {
                CB_Instruction::BIT_b_r(b, r) => {
                    // Get r value and check bit b on it
                    let value = if r == 6 {
                        self.add_cycles(12);
                        self.read_mem(self.hl())
                    } else {
                        self.add_cycles(8);
                        self.read_reg_r(r)
                    };
                    self.flag_z = value & (1 << b) == 0;
                    self.flag_h = true;
                    self.flag_n = false;
                }
                CB_Instruction::SET_b_r(b, r) => {
                    let mut value = if r == 6 {
                        self.add_cycles(12);
                        self.read_mem(self.hl())
                    } else {
                        self.add_cycles(8);
                        self.read_reg_r(r)
                    };
                    value |= 1 << b;
                    if r == 6 {
                        self.write_mem(self.hl(), value);
                    } else {
                        self.set_reg_r(r, value);
                    }
                }
                CB_Instruction::RES_b_r(b, r) => {
                    let mut value = if r == 6 {
                        self.add_cycles(12);
                        self.read_mem(self.hl())
                    } else {
                        self.add_cycles(8);
                        self.read_reg_r(r)
                    };
                    value &= !(1 << b);
                    if r == 6 {
                        self.write_mem(self.hl(), value);
                    } else {
                        self.set_reg_r(r, value);
                    }
                }
                CB_Instruction::RL_n(n) => {
                    let mut value = if n == 6 {
                        self.read_mem(self.hl())
                    } else {
                        self.read_reg_r(n)
                    };
                    let bit7 = value >> 7;
                    value <<= 1;
                    value += self.flag_c as u8;

                    self.flag_c = bit7 == 1;
                    self.flag_z = value == 0;
                    self.flag_n = false;
                    self.flag_h = false;

                    if n == 6 {
                        self.write_mem(self.hl(), value);
                        self.add_cycles(16);
                    } else {
                        self.set_reg_r(n, value);
                        self.add_cycles(8);
                    }
                }
                CB_Instruction::RLC_n(n) => {
                    let mut value = if n == 6 {
                        self.read_mem(self.hl())
                    } else {
                        self.read_reg_r(n)
                    };
                    let bit7 = value >> 7;
                    value <<= 1;
                    value |= bit7;

                    self.flag_c = bit7 == 1;
                    self.flag_z = value == 0;
                    self.flag_n = false;
                    self.flag_h = false;

                    if n == 6 {
                        self.write_mem(self.hl(), value);
                        self.add_cycles(16);
                    } else {
                        self.set_reg_r(n, value);
                        self.add_cycles(8);
                    }
                }

                CB_Instruction::SLA_n(n) => {
                    let mut value = if n == 6 {
                        self.read_mem(self.hl())
                    } else {
                        self.read_reg_r(n)
                    };
                    let bit7 = value >> 7;
                    value <<= 1;

                    self.flag_c = bit7 == 1;
                    self.flag_z = value == 0;
                    self.flag_n = false;
                    self.flag_h = false;

                    if n == 6 {
                        self.write_mem(self.hl(), value);
                        self.add_cycles(16);
                    } else {
                        self.set_reg_r(n, value);
                        self.add_cycles(8);
                    }
                }
                CB_Instruction::RRC_n(n) => {
                    let mut value = if n == 6 {
                        self.read_mem(self.hl())
                    } else {
                        self.read_reg_r(n)
                    };
                    let bit0 = value & 0b1;
                    value >>= 1;
                    value |= (bit0) << 7;

                    self.flag_c = bit0 == 1;
                    self.flag_z = value == 0;
                    self.flag_n = false;
                    self.flag_h = false;

                    if n == 6 {
                        self.write_mem(self.hl(), value);
                        self.add_cycles(16);
                    } else {
                        self.set_reg_r(n, value);
                        self.add_cycles(8);
                    }
                }
                CB_Instruction::SRA_n(n) => {
                    let mut value = if n == 6 {
                        self.read_mem(self.hl())
                    } else {
                        self.read_reg_r(n)
                    };
                    let bit0 = value & 0b1;
                    let bit7 = value >> 7;
                    value >>= 1;
                    // bit 7 stays where it was
                    value |= (bit7) << 7;

                    self.flag_c = bit0 == 1;
                    self.flag_z = value == 0;
                    self.flag_n = false;
                    self.flag_h = false;

                    if n == 6 {
                        self.write_mem(self.hl(), value);
                        self.add_cycles(16);
                    } else {
                        self.set_reg_r(n, value);
                        self.add_cycles(8);
                    }
                }
                CB_Instruction::SRL_n(n) => {
                    let mut value = if n == 6 {
                        self.read_mem(self.hl())
                    } else {
                        self.read_reg_r(n)
                    };
                    let bit0 = value & 0b1;
                    value >>= 1;

                    self.flag_c = bit0 == 1;
                    self.flag_z = value == 0;
                    self.flag_n = false;
                    self.flag_h = false;

                    if n == 6 {
                        self.write_mem(self.hl(), value);
                        self.add_cycles(16);
                    } else {
                        self.set_reg_r(n, value);
                        self.add_cycles(8);
                    }
                }
                CB_Instruction::RR_n(n) => {
                    let mut value = if n == 6 {
                        self.read_mem(self.hl())
                    } else {
                        self.read_reg_r(n)
                    };
                    let bit0 = value & 0b1;
                    value >>= 1;
                    value |= (self.flag_c as u8) << 7;

                    self.flag_c = bit0 == 1;
                    self.flag_z = value == 0;
                    self.flag_n = false;
                    self.flag_h = false;

                    if n == 6 {
                        self.write_mem(self.hl(), value);
                        self.add_cycles(16);
                    } else {
                        self.set_reg_r(n, value);
                        self.add_cycles(8);
                    }
                }
                CB_Instruction::SWAP_n(n) => {
                    let mut value = if n == 6 {
                        self.read_mem(self.hl())
                    } else {
                        self.read_reg_r(n)
                    };
                    value = value.swap_bytes();

                    self.flag_z = value == 0;
                    self.flag_n = false;
                    self.flag_h = false;
                    self.flag_c = false;

                    if n == 6 {
                        self.write_mem(self.hl(), value);
                        self.add_cycles(16);
                    } else {
                        self.set_reg_r(n, value);
                        self.add_cycles(8);
                    }
                }

                _ => panic!("Unimplemented cb instruction: {:?}", inst),
            }
        }
    }

    fn push_stack(&mut self, value: u8) {
        self.write_mem(self.reg_sp, value);
        self.reg_sp -= 1;
    }
    fn push_stack_u16(&mut self, value: u16) {
        let (first, second) = u16_as_u8s(value);
        self.push_stack(first);
        self.push_stack(second);
    }
    fn pop_stack(&mut self) -> u8 {
        // Add first, so we are reading the old value
        // As reg_sp always points to the next empty spot
        self.reg_sp += 1;
        let ret = self.read_mem(self.reg_sp);
        ret
    }
    fn pop_stack_u16(&mut self) -> u16 {
        // Saving and reading are in opposite directions
        let second = self.pop_stack();
        let first = self.pop_stack();
        u8s_as_u16((first, second))
    }

    fn add_cycles(&mut self, amount: i32) {
        self.cycles += amount;
    }

    fn read_reg_r(&self, r: u8) -> u8 {
        match r {
            0 => self.reg_b,
            1 => self.reg_c,
            2 => self.reg_d,
            3 => self.reg_e,
            4 => self.reg_h,
            5 => self.reg_l,
            6 => panic!("Cpu::read_reg_r  (HL) not handled with this function"),
            7 => self.reg_a,

            _ => panic!("Cpu::read_reg_r  Invalid r: {}", r),
        }
    }

    fn read_mem(&mut self, address: u16) -> u8 {
        if address == 0xFFFF {
            println!("pc at {:04x}", self.reg_pc - 1);
        }
        self.interconnect.read_mem(address)
    }

    fn write_mem(&mut self, address: u16, value: u8) {
        if address == 0xFFFF {
            println!("pc at {:04x}", self.reg_pc - 1);
        }
        self.interconnect.write_mem(address, value);
    }

    fn set_reg_r(&mut self, r: u8, value: u8) {
        match r {
            0 => self.reg_b = value,
            1 => self.reg_c = value,
            2 => self.reg_d = value,
            3 => self.reg_e = value,
            4 => self.reg_h = value,
            5 => self.reg_l = value,
            6 => panic!("Cpu::read_reg_r  (HL) not handled with this function"),
            7 => self.reg_a = value,

            _ => panic!("Cpu::read_reg_r  Invalid r: {}", r),
        }
    }

    fn read_byte(&mut self) -> u8 {
        let ret = self.read_mem(self.reg_pc);
        self.reg_pc += 1;
        ret
    }

    fn read_nn(&mut self) -> (u8, u8) {
        let first = self.read_byte();
        let second = self.read_byte();
        // Endianness so turn it around
        (second, first)
    }

    fn af(&self) -> u16 {
        u8s_as_u16((self.reg_a, self.reg_f))
    }

    fn bc(&self) -> u16 {
        u8s_as_u16((self.reg_b, self.reg_c))
    }

    fn de(&self) -> u16 {
        u8s_as_u16((self.reg_d, self.reg_e))
    }

    fn hl(&self) -> u16 {
        u8s_as_u16((self.reg_h, self.reg_l))
    }

    fn set_af(&mut self, val: u16) {
        let (h, l) = u16_as_u8s(val);
        self.reg_a = h;
        self.reg_f = l;
    }

    fn set_bc(&mut self, val: u16) {
        let (h, l) = u16_as_u8s(val);
        self.reg_b = h;
        self.reg_c = l;
    }

    fn set_de(&mut self, val: u16) {
        let (h, l) = u16_as_u8s(val);
        self.reg_d = h;
        self.reg_e = l;
    }

    fn set_hl(&mut self, val: u16) {
        let (h, l) = u16_as_u8s(val);
        self.reg_h = h;
        self.reg_l = l;
    }
}

#[inline(always)]
fn u16_as_u8s(val: u16) -> (u8, u8) {
    ((val >> 8) as u8, (val & 0xFF) as u8)
}

#[inline(always)]
fn u8s_as_u16(val: (u8, u8)) -> u16 {
    ((val.0 as u16) << 8) + val.1 as u16
}

#[inline(always)]
fn check_borrow_bit4(old: u8, new: u8) -> bool {
    (old >> 3) == (new >> 3)
}

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u8s_as_u16() {
        assert_eq!(u8s_as_u16((0x12, 0x34)), 0x1234);
        assert_eq!(u8s_as_u16((0xFF, 0xEF)), 0xFFEF);
    }

    #[test]
    fn test_u16_as_u8s() {
        assert_eq!(u16_as_u8s(0x1234), (0x12, 0x34));
        assert_eq!(u16_as_u8s(0xFFFF), (0xFF, 0xFF));
    }
}
