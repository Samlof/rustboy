use super::console::CpuText;
use super::instruction;
use super::instruction::{CB_Instruction, Instruction};
use super::interconnect::*;
use super::ppu::Color;
use std::sync::mpsc;

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

    // Interrupt related flags
    // Interrupt master flag
    flag_ime: bool,
    flag_disabling_interrupts: bool,
    flag_enabling_interrupts: bool,

    pub interconnect: Interconnect,
    cycles: i32,
    halt: bool,
    stop: bool,

    // Debug variables
    print_instructions: bool,
    console_tx: Option<mpsc::Sender<CpuText>>,

    test_counter: i64,
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
            reg_pc: 0xFE,

            flag_ime: false,
            flag_disabling_interrupts: false,
            flag_enabling_interrupts: false,
            halt: false,
            stop: false,
            interconnect,
            cycles: 0,

            print_instructions: false,
            console_tx: None,
            test_counter: 0,
        }
    }

    pub fn step(&mut self) {
        // If cycles to burn, just return
        if self.cycles > 0 {
            self.cycles -= 4;
            return;
        }
        // TODO: Handle stop
        if self.stop {
            self.stop = false;
        }
        // Handle Halt
        if self.halt {
            if !self.interconnect.check_interrupt() {
                return;
            }
            self.halt = false;
        }
        // Interrupts
        if self.flag_ime {
            self.handle_interrupts();
        }

        // Handle the change interrupt flags
        if self.flag_disabling_interrupts {
            self.flag_disabling_interrupts = false;
            self.flag_ime = false;
        }
        if self.flag_enabling_interrupts {
            self.flag_enabling_interrupts = false;
            self.flag_ime = true;
        }
        self.do_next_instrution();
    }

    fn handle_interrupts(&mut self) {
        let interrupt = match self.interconnect.get_interrupt() {
            Some(i) => i,
            None => return,
        };

        if let Some(ref tx) = self.console_tx {
            tx.send(CpuText::Interrupt(format!("{:?}", interrupt)));
        }

        // Disable interrupts
        self.flag_ime = false;

        // Jump to interrupt address
        self.push_stack_u16(self.reg_pc);
        self.reg_pc = match interrupt {
            Interrupt::VBLANK => 0x0040,
            Interrupt::LCDStatus => 0x0048,
            Interrupt::TimerOverflow => 0x0050,
            Interrupt::SerialTransfer => 0x0058,
            Interrupt::Joypad => 0x0060,
        };
    }

    fn send_instr_text(&self, str: String) {
        println!("got: {}", str);
        return;
        if let Some(ref tx) = self.console_tx {
            tx.send(CpuText::Instruction(str));
        }
    }

    fn do_next_instrution(&mut self) {
        let opcode = self.read_byte();
        let instr = match instruction::parse(opcode) {
            Some(o) => o,
            None => {
                self.send_instr_text(format!(
                    "0x{:04x}  Undefined opcode: 0x{:02x}",
                    self.reg_pc - 1,
                    opcode
                ));
                return;
            }
        };

        // instruction string is only used if self.print_instructions is true
        // But need to declare it still here, to use it later in the same function
        let mut instruction_string = String::with_capacity(20);
        if self.print_instructions {
            instruction_string.push_str(&format!("0x{:04x} ", self.reg_pc - 1));
        }
        self.add_cycles(4);

        match instr {
            Instruction::LD_r1_r2(r1, r2) => {
                if self.print_instructions {
                    instruction_string.push_str(&format!("LD {}, {}", reg_char(r1), reg_char(r2)));
                }
                let value = self.read_reg_r(r2);
                self.write_reg_r(r1, value);
            }
            Instruction::LD_r1_n(r1) => {
                let value = self.read_byte();
                if self.print_instructions {
                    instruction_string.push_str(&format!("LD {}, ${:02x}", reg_char(r1), value));
                }
                self.write_reg_r(r1, value);
            }
            Instruction::LD_A_nnptr => {
                self.reg_a = match opcode {
                    0x0A => {
                        if self.print_instructions {
                            instruction_string.push_str(&format!("LD A, (BC)"));
                        };
                        self.read_mem(self.bc())
                    }
                    0x1A => {
                        if self.print_instructions {
                            instruction_string.push_str(&format!("LD A, (DE)"));
                        };
                        self.read_mem(self.de())
                    }
                    0xFA => {
                        let address = u8s_as_u16(self.read_nn());
                        if self.print_instructions {
                            instruction_string.push_str(&format!("LD A, $({:04x})", address));
                        };
                        self.read_mem(address)
                    }
                    _ => unreachable!(),
                };
            }
            Instruction::LD_nnptr_A => {
                match opcode {
                    0x02 => {
                        if self.print_instructions {
                            instruction_string.push_str(&format!("LD (BC), A"));
                        };
                        self.write_mem(self.bc(), self.reg_a);
                    }
                    0x12 => {
                        if self.print_instructions {
                            instruction_string.push_str(&format!("LD (DE), A"));
                        };
                        self.write_mem(self.de(), self.reg_a);
                    }
                    0xEA => {
                        let address = u8s_as_u16(self.read_nn());
                        if self.print_instructions {
                            instruction_string.push_str(&format!("LD 0x({:04x}), A", address));
                        };
                        self.write_mem(address, self.reg_a);
                    }
                    _ => unreachable!(),
                };
            }
            Instruction::LD_A_Cptr => {
                let address = 0xFF00 + self.reg_c as u16;
                if self.print_instructions {
                    instruction_string.push_str(&format!("LD A, ($FF00+C)"));
                }
                self.reg_a = self.read_mem(address);
            }
            Instruction::LD_Cptr_A => {
                let address = 0xFF00 + self.reg_c as u16;
                if self.print_instructions {
                    instruction_string.push_str(&format!("LD (C), A"));
                }
                self.write_mem(address, self.reg_a);
            }
            Instruction::LDD_A_HLptr => {
                if self.print_instructions {
                    instruction_string.push_str(&format!("LD A, (HL-)"));
                }
                let address = self.hl();
                self.reg_a = self.read_mem(address);
                self.set_hl(address - 1);
            }
            Instruction::LDD_HLptr_A => {
                if self.print_instructions {
                    instruction_string.push_str(&format!("LD (HL-), A"));
                }
                let address = self.hl();
                self.write_mem(address, self.reg_a);
                self.set_hl(address - 1);
            }
            Instruction::LDI_A_HLptr => {
                if self.print_instructions {
                    instruction_string.push_str(&format!("LD A, (HL+)"));
                }
                let address = self.hl();
                self.reg_a = self.read_mem(address);
                self.set_hl(address + 1);
            }
            Instruction::LDI_HLptr_A => {
                if self.print_instructions {
                    instruction_string.push_str(&format!("LD (HL-), A"));
                }
                let address = self.hl();
                self.write_mem(address, self.reg_a);
                self.set_hl(address + 1);
            }

            Instruction::LDH_nptr_A => {
                let byte = 0xFF00 + self.read_byte() as u16;
                if self.print_instructions {
                    instruction_string.push_str(&format!("LDH $({:02x}), A", byte));
                }
                self.write_mem(byte, self.reg_a);
            }
            Instruction::LDH_A_nptr => {
                let byte = 0xFF00 + self.read_byte() as u16;
                if self.print_instructions {
                    instruction_string.push_str(&format!("LDH A, $({:02x})", byte));
                }
                self.reg_a = self.read_mem(byte);
            }

            Instruction::LD_rr_nn => {
                let value = u8s_as_u16(self.read_nn());
                match opcode {
                    0x01 => {
                        if self.print_instructions {
                            instruction_string.push_str(&format!("LD BC, ${:04x}", value));
                        }
                        self.set_bc(value);
                    }
                    0x11 => {
                        if self.print_instructions {
                            instruction_string.push_str(&format!("LD DE, ${:04x}", value));
                        }
                        self.set_de(value);
                    }
                    0x21 => {
                        if self.print_instructions {
                            instruction_string.push_str(&format!("LD HL, ${:04x}", value));
                        }
                        self.set_hl(value);
                    }
                    0x31 => {
                        if self.print_instructions {
                            instruction_string.push_str(&format!("LD SP, ${:04x}", value));
                        }
                        self.reg_sp = value;
                    }
                    _ => unreachable!(),
                }
            }
            Instruction::LD_SP_HL => {
                if self.print_instructions {
                    instruction_string.push_str(&format!("LD SP,HL"));
                }
                self.reg_sp = self.hl();
                // Need to add 4 more to total 8
                self.add_cycles(4);
            }
            Instruction::LDHL_SPn => {
                // Sign extending
                let n = ((self.read_byte() as i8) as i16) as u16;
                if self.print_instructions {
                    instruction_string.push_str(&format!("LD HL, SP+${:02x}", n));
                }
                let result = self.reg_sp + n;
                self.set_hl(result);

                self.set_flag_z(false);
                self.set_flag_n(false);

                self.set_flag_h(((self.reg_sp ^ n ^ result) & 0x10) == 0x10);
                self.set_flag_c(((self.reg_sp ^ n ^ result) & 0x100) == 0x100);

                // Need to add 4 more to total 12
                self.add_cycles(4);
            }
            Instruction::LD_nn_SP => {
                let nn = u8s_as_u16(self.read_nn());
                if self.print_instructions {
                    instruction_string.push_str(&format!("LD (${:04x}), SP", nn));
                }
                let (high, low) = u16_as_u8s(self.reg_sp);
                self.write_mem(nn, low);
                self.write_mem(nn + 1, high);
            }

            Instruction::PUSH_nn => {
                match opcode {
                    0xF5 => {
                        if self.print_instructions {
                            instruction_string.push_str(&format!("PUSH AF"));
                        }
                        self.push_stack_u16(self.af());
                    }
                    0xC5 => {
                        if self.print_instructions {
                            instruction_string.push_str(&format!("PUSH BC"));
                        }
                        self.push_stack_u16(self.bc());
                    }
                    0xD5 => {
                        if self.print_instructions {
                            instruction_string.push_str(&format!("PUSH DE"));
                        }
                        self.push_stack_u16(self.de());
                    }
                    0xE5 => {
                        if self.print_instructions {
                            instruction_string.push_str(&format!("PUSH HL"));
                        }
                        self.push_stack_u16(self.hl());
                    }
                    _ => unreachable!(),
                };
                // Need to add 12 more to total 16
                self.add_cycles(12);
            }
            Instruction::POP_nn => {
                let value = self.pop_stack_u16();
                match opcode {
                    0xF1 => {
                        if self.print_instructions {
                            instruction_string.push_str(&format!("POP AF"));
                        }
                        self.set_af(value);
                    }
                    0xC1 => {
                        if self.print_instructions {
                            instruction_string.push_str(&format!("POP BC"));
                        }
                        self.set_bc(value);
                    }
                    0xD1 => {
                        if self.print_instructions {
                            instruction_string.push_str(&format!("POP DE"));
                        }
                        self.set_de(value);
                    }
                    0xE1 => {
                        if self.print_instructions {
                            instruction_string.push_str(&format!("POP HL"));
                        }
                        self.set_hl(value);
                    }
                    _ => unreachable!(),
                }
                // Add 8 more to total 12
                self.add_cycles(8);
            }

            Instruction::ADD_n(n) => {
                let n = if n == 8 {
                    let value = self.read_byte();
                    if self.print_instructions {
                        instruction_string.push_str(&format!("ADD ${:02x}", value));
                    }
                    value
                } else {
                    if self.print_instructions {
                        instruction_string.push_str(&format!("ADD {}", reg_char(n)));
                    }
                    self.read_reg_r(n)
                };
                let result = self.reg_a as u16 + n as u16;
                let carrybits = (self.reg_a ^ n) as u16 ^ result;
                self.reg_a = result as u8;

                self.set_flag_z(self.reg_a == 0);
                self.set_flag_n(false);
                self.set_flag_c(carrybits & 0x100 != 0);
                self.set_flag_h(carrybits & 0x10 != 0);
            }
            Instruction::ADC_n(n) => {
                let n = if n == 8 {
                    let value = self.read_byte();
                    if self.print_instructions {
                        instruction_string.push_str(&format!("ADC ${:02x}", value));
                    }
                    value
                } else {
                    if self.print_instructions {
                        instruction_string.push_str(&format!("ADC {}", reg_char(n)));
                    }
                    self.read_reg_r(n)
                };
                let carry = self.flag_c() as u16;
                let result: u16 = self.reg_a as u16 + n as u16 + carry;

                self.set_flag_z(result as u8 == 0);
                self.set_flag_n(false);
                self.set_flag_h((self.reg_a & 0xF) + (n & 0xF) + carry as u8 > 0xF);
                self.set_flag_c(result > 0xFF);

                self.reg_a = result as u8;
            }
            Instruction::SUB_n(n) => {
                let n = if n == 8 {
                    let value = self.read_byte();
                    if self.print_instructions {
                        instruction_string.push_str(&format!("SUB ${:02x}", value));
                    }
                    value
                } else {
                    if self.print_instructions {
                        instruction_string.push_str(&format!("SUB {}", reg_char(n)));
                    }
                    self.read_reg_r(n)
                };

                let result = self.reg_a as i16 - n as i16;
                let carrybits = (self.reg_a ^ n) as i16 ^ result;
                self.reg_a = result as u8;

                self.set_flag_z(self.reg_a == 0);
                self.set_flag_n(true);
                self.set_flag_h(carrybits & 0x10 != 0);
                self.set_flag_c(carrybits & 0x100 != 0);
            }
            Instruction::SBC_n(n) => {
                let n = if n == 8 {
                    let value = self.read_byte();
                    if self.print_instructions {
                        instruction_string.push_str(&format!("SBC ${:02x}", value));
                    }
                    value
                } else {
                    if self.print_instructions {
                        instruction_string.push_str(&format!("SBC {}", reg_char(n)));
                    }
                    self.read_reg_r(n)
                };
                let carry = if self.flag_c() { 1 } else { 0 };

                let result = self.reg_a as i16 - n as i16 - carry;
                let carrybits = (self.reg_a ^ n) as i16 ^ carry ^ result;
                self.reg_a = result as u8;

                self.set_flag_z(self.reg_a == 0);
                self.set_flag_n(true);
                self.set_flag_h(carrybits & 0x10 != 0);
                self.set_flag_c(carrybits & 0x100 != 0);
            }
            Instruction::AND_n(n) => {
                let n = if n == 8 {
                    let value = self.read_byte();
                    if self.print_instructions {
                        instruction_string.push_str(&format!("AND ${:02x}", value));
                    }
                    value
                } else {
                    if self.print_instructions {
                        instruction_string.push_str(&format!("AND {}", reg_char(n)));
                    }
                    self.read_reg_r(n)
                };
                self.reg_a = self.reg_a & n;

                self.set_flag_z(self.reg_a == 0);
                self.set_flag_n(false);
                self.set_flag_h(true);
                self.set_flag_c(false);
            }
            Instruction::OR_n(n) => {
                let n = if n == 8 {
                    let value = self.read_byte();
                    if self.print_instructions {
                        instruction_string.push_str(&format!("OR ${:02x}", value));
                    }
                    value
                } else {
                    if self.print_instructions {
                        instruction_string.push_str(&format!("OR {}", reg_char(n)));
                    }
                    self.read_reg_r(n)
                };
                self.reg_a = self.reg_a | n;

                self.set_flag_z(self.reg_a == 0);
                self.set_flag_n(false);
                self.set_flag_h(false);
                self.set_flag_c(false);
            }
            Instruction::XOR_n(n) => {
                let n = if n == 8 {
                    let value = self.read_byte();
                    if self.print_instructions {
                        instruction_string.push_str(&format!("XOR ${:02x}", value));
                    }
                    value
                } else {
                    if self.print_instructions {
                        instruction_string.push_str(&format!("XOR {}", reg_char(n)));
                    }
                    self.read_reg_r(n)
                };
                self.reg_a = self.reg_a ^ n;

                self.set_flag_z(self.reg_a == 0);
                self.set_flag_n(false);
                self.set_flag_h(false);
                self.set_flag_c(false);
            }
            Instruction::CP_n(n) => {
                let n = if n == 8 {
                    let value = self.read_byte();
                    if self.print_instructions {
                        instruction_string.push_str(&format!("CP ${:02x}", value));
                    }
                    value
                } else {
                    if self.print_instructions {
                        instruction_string.push_str(&format!("CP {}", reg_char(n)));
                    }
                    self.read_reg_r(n)
                };
                self.set_flag_n(true);
                self.set_flag_c(self.reg_a < n);
                self.set_flag_z(self.reg_a == n);
                self.set_flag_h((self.reg_a.wrapping_sub(n)) & 0xF > self.reg_a & 0xF);
            }
            Instruction::INC_n(r) => {
                if self.print_instructions {
                    instruction_string.push_str(&format!("INC {}", reg_char(r)));
                }

                let n = self.read_reg_r(r);
                let result = n.wrapping_add(1);

                self.set_flag_z(result == 0);
                self.set_flag_n(false);
                self.set_flag_h(result & 0x0F == 0);
                self.write_reg_r(r, result);
            }
            Instruction::DEC_n(r) => {
                if self.print_instructions {
                    instruction_string.push_str(&format!("DEC {}", reg_char(r)));
                }

                let n = self.read_reg_r(r);
                let result = n.wrapping_sub(1);

                self.set_flag_z(result == 0);
                self.set_flag_n(true);
                self.set_flag_h(result & 0x0F == 0x0F);
                self.write_reg_r(r, result);
            }

            Instruction::ADD_HL_nn(nn) => {
                let nn = match nn {
                    0 => {
                        if self.print_instructions {
                            instruction_string.push_str(&format!("ADD HL, BC"));
                        }
                        self.bc()
                    }
                    1 => {
                        if self.print_instructions {
                            instruction_string.push_str(&format!("ADD HL, DE"));
                        }
                        self.de()
                    }
                    2 => {
                        if self.print_instructions {
                            instruction_string.push_str(&format!("ADD HL, HL"));
                        }
                        self.hl()
                    }
                    3 => {
                        if self.print_instructions {
                            instruction_string.push_str(&format!("ADD HL, SP"));
                        }
                        self.reg_sp
                    }
                    _ => unreachable!(),
                };
                let nn = nn as u32;
                let result = self.hl() as u32 + nn;

                self.set_flag_n(false);
                self.set_flag_h(((self.hl() as u32 ^ nn) ^ (result & 0xFFFF)) & 0x1000 > 0);
                self.set_flag_c(result & 0x10000 > 0);

                self.set_hl(result as u16);

                self.add_cycles(4);
            }
            Instruction::ADD_SP_n => {
                // sign extend
                let n = ((self.read_byte() as i8) as i16) as u16;
                if self.print_instructions {
                    instruction_string.push_str(&format!("ADD SP, ${:x}", n));
                }
                let result = self.reg_sp + n;

                self.set_flag_z(false);
                self.set_flag_n(false);
                self.set_flag_h((self.reg_sp ^ n ^ (result & 0xFFFF)) & 0x10 == 0x10);
                self.set_flag_c((self.reg_sp ^ n ^ (result & 0xFFFF)) & 0x100 == 0x100);

                self.reg_sp = result;
                self.add_cycles(8);
            }
            Instruction::INC_nn(nn) => {
                match nn {
                    0 => {
                        if self.print_instructions {
                            instruction_string.push_str(&format!("INC BC"));
                        }
                        let value = self.bc();
                        self.set_bc(value + 1);
                    }
                    1 => {
                        if self.print_instructions {
                            instruction_string.push_str(&format!("INC DE"));
                        }
                        let value = self.de();
                        self.set_de(value + 1);
                    }
                    2 => {
                        if self.print_instructions {
                            instruction_string.push_str(&format!("INC HL"));
                        }
                        let value = self.hl();
                        self.set_hl(value + 1);
                    }
                    3 => {
                        if self.print_instructions {
                            instruction_string.push_str(&format!("INC SP"));
                        }
                        self.reg_sp += 1;
                    }
                    _ => unreachable!(),
                };
                self.add_cycles(4);
            }
            Instruction::DEC_nn(nn) => {
                match nn {
                    0 => {
                        if self.print_instructions {
                            instruction_string.push_str(&format!("DEC BC"));
                        }
                        let value = self.bc();
                        self.set_bc(value - 1);
                    }
                    1 => {
                        if self.print_instructions {
                            instruction_string.push_str(&format!("DEC DE"));
                        }
                        let value = self.de();
                        self.set_de(value - 1);
                    }
                    2 => {
                        if self.print_instructions {
                            instruction_string.push_str(&format!("DEC HL"));
                        }
                        let value = self.hl();
                        self.set_hl(value - 1);
                    }
                    3 => {
                        if self.print_instructions {
                            instruction_string.push_str(&format!("DEC SP"));
                        }
                        self.reg_sp -= 1;
                    }
                    _ => unreachable!(),
                };
                self.add_cycles(4);
            }

            Instruction::CPL => {
                if self.print_instructions {
                    instruction_string.push_str(&format!("CPL"));
                }
                self.reg_a = !self.reg_a;
                self.set_flag_h(true);
                self.set_flag_n(true);
            }
            Instruction::CCF => {
                if self.print_instructions {
                    instruction_string.push_str(&format!("CCF"));
                }
                self.set_flag_c(!self.flag_c());
                self.set_flag_n(false);
                self.set_flag_h(false);
            }
            Instruction::SCF => {
                if self.print_instructions {
                    instruction_string.push_str(&format!("SCF"));
                }
                self.set_flag_c(true);
                self.set_flag_n(false);
                self.set_flag_h(false);
            }
            Instruction::NOP => {
                if self.print_instructions {
                    instruction_string.push_str(&format!("NOP"));
                }
            }
            Instruction::HALT => {
                if self.print_instructions {
                    instruction_string.push_str(&format!("HALT"));
                }
                self.halt = true;
            }
            Instruction::STOP => {
                // STOP always follows a 00
                let byte = self.read_byte();
                if self.print_instructions {
                    instruction_string.push_str(&format!("STOP"));
                }
                self.stop = true;
                self.interconnect.ppu.turn_lcd_off();
            }
            Instruction::DI => {
                if self.print_instructions {
                    instruction_string.push_str(&format!("DI"));
                }
                self.flag_disabling_interrupts = true;
            }
            Instruction::EI => {
                if self.print_instructions {
                    instruction_string.push_str(&format!("EI"));
                }
                self.flag_enabling_interrupts = true;
            }

            Instruction::RLCA => {
                if self.print_instructions {
                    instruction_string.push_str(&format!("RLCA"));
                }
                let bit7 = self.reg_a >> 7;
                self.reg_a <<= 1;
                self.reg_a += bit7;

                self.set_flag_z(false);
                self.set_flag_n(false);
                self.set_flag_h(false);
                self.set_flag_c(bit7 == 1);
            }
            Instruction::RLA => {
                if self.print_instructions {
                    instruction_string.push_str(&format!("RLA"));
                }
                let bit7 = self.reg_a >> 7;
                self.reg_a <<= 1;
                self.reg_a += self.flag_c() as u8;

                self.set_flag_z(false);
                self.set_flag_n(false);
                self.set_flag_h(false);
                self.set_flag_c(bit7 == 1);
            }
            Instruction::RRCA => {
                if self.print_instructions {
                    instruction_string.push_str(&format!("RRCA"));
                }
                let bit0 = self.reg_a & 1;
                self.reg_a >>= 1;
                self.reg_a += bit0 << 7;

                self.set_flag_z(false);
                self.set_flag_n(false);
                self.set_flag_h(false);
                self.set_flag_c(bit0 == 1);
            }
            Instruction::RRA => {
                if self.print_instructions {
                    instruction_string.push_str(&format!("RRA"));
                }
                let bit0 = self.reg_a & 1;
                self.reg_a >>= 1;
                self.reg_a += (self.flag_c() as u8) << 7;

                self.set_flag_z(false);
                self.set_flag_n(false);
                self.set_flag_h(false);
                self.set_flag_c(bit0 == 1);
            }

            Instruction::JP_nn => {
                let address = u8s_as_u16(self.read_nn());
                if self.print_instructions {
                    instruction_string.push_str(&format!("JP ${:04x}", address));
                }
                self.reg_pc = address;
            }
            Instruction::JP_cc_nn(cc) => {
                let address = u8s_as_u16(self.read_nn());
                if self.print_instructions {
                    instruction_string.push_str(&format!("JP {} ${:04x}", cc_to_char(cc), address));
                }
                if self.check_cc(cc) {
                    self.reg_pc = address;
                }
            }
            Instruction::JP_HLptr => {
                if self.print_instructions {
                    instruction_string.push_str(&format!("JP (HL)"));
                }
                self.reg_pc = self.hl();
            }
            Instruction::JR_n => {
                // Sign extend
                let n = ((self.read_byte() as i8) as i16) as u16;
                if self.print_instructions {
                    instruction_string.push_str(&format!("JR {}", n as i16));
                }
                self.reg_pc = self.reg_pc.wrapping_add(n);
                self.add_cycles(4);
            }
            Instruction::JR_cc_n(cc) => {
                // Sign extend
                let n = ((self.read_byte() as i8) as i16) as u16;
                if self.print_instructions {
                    instruction_string.push_str(&format!("JR {} {}", cc_to_char(cc), n as i16));
                }
                if self.check_cc(cc) {
                    self.reg_pc = self.reg_pc.wrapping_add(n);
                }
                self.add_cycles(4);
            }

            Instruction::CALL_nn => {
                let nn = u8s_as_u16(self.read_nn());
                if self.print_instructions {
                    instruction_string.push_str(&format!("CALL ${:04x}", nn));
                }
                self.push_stack_u16(self.reg_pc);
                self.reg_pc = nn;
                self.add_cycles(8);
            }

            Instruction::CALL_cc_nn(cc) => {
                let nn = u8s_as_u16(self.read_nn());
                if self.print_instructions {
                    instruction_string.push_str(&format!("CALL {} ${:04x}", cc_to_char(cc), nn));
                }
                if self.check_cc(cc) {
                    self.push_stack_u16(self.reg_pc);
                    self.reg_pc = nn;
                }
                self.add_cycles(8);
            }

            Instruction::RST_n(n) => {
                if self.print_instructions {
                    instruction_string.push_str(&format!("RST ${:02x}H", n));
                }
                self.push_stack_u16(self.reg_pc);
                self.reg_pc = n as u16;
                self.add_cycles(28);
            }
            Instruction::RET => {
                if self.print_instructions {
                    instruction_string.push_str(&format!("RET"));
                }
                let address = self.pop_stack_u16();
                self.reg_pc = address;
                self.add_cycles(4);
            }
            Instruction::RET_cc(cc) => {
                if self.print_instructions {
                    instruction_string.push_str(&format!("RET {}", cc_to_char(cc)));
                }
                if self.check_cc(cc) {
                    let address = self.pop_stack_u16();
                    self.reg_pc = address;
                }
                self.add_cycles(4);
            }
            Instruction::RETI => {
                if self.print_instructions {
                    instruction_string.push_str("RETI");
                }
                let address = self.pop_stack_u16();
                self.reg_pc = address;
                self.flag_ime = true;
                self.add_cycles(8);
            }
            Instruction::DAA => {
                if self.print_instructions {
                    instruction_string.push_str("DAA");
                }
                let mut a = self.reg_a as u16;
                if !self.flag_n() {
                    if self.flag_h() || ((a & 0xF) > 9) {
                        a += 0x06;
                    }
                    if self.flag_c() || (a > 0x9F) {
                        a += 0x60;
                    }
                } else {
                    if self.flag_h() {
                        a = (a.wrapping_sub(6)) & 0xFF;
                    }
                    if self.flag_c() {
                        a = a.wrapping_sub(0x60);
                    }
                }

                self.set_flag_h(false);
                if a >= 0x100 {
                    self.set_flag_c(true);
                }

                a &= 0xFF;
                self.set_flag_z(a == 0);

                self.reg_a = a as u8;
            }
            Instruction::CB => self.handle_cb_opcode(),
        }
        if self.print_instructions && instr != Instruction::CB {
            self.send_instr_text(instruction_string);
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
            let inst = instruction::parse_cb(opcode);

            self.add_cycles(4);

            let mut instruction_string = String::with_capacity(20);
            if self.print_instructions {
                instruction_string.push_str(&format!("0x{:04x} ", self.reg_pc - 2));
            }

            match inst {
                CB_Instruction::BIT_b_r(b, r) => {
                    if self.print_instructions {
                        instruction_string =
                            instruction_string + &format!("BIT {}, {}", b, reg_char(r));
                    }
                    // Get r value and check bit b on it
                    let value = self.read_reg_r(r);
                    self.set_flag_z(value & (1 << b) == 0);
                    self.set_flag_h(true);
                    self.set_flag_n(false);
                }
                CB_Instruction::SET_b_r(b, r) => {
                    if self.print_instructions {
                        instruction_string =
                            instruction_string + &format!("SET {}, {}", b, reg_char(r));
                    }
                    let mut value = self.read_reg_r(r);
                    value |= 1 << b;
                    self.write_reg_r(r, value);
                }
                CB_Instruction::RES_b_r(b, r) => {
                    if self.print_instructions {
                        instruction_string =
                            instruction_string + &format!("RES {}, {}", b, reg_char(r));
                    }
                    let mut value = self.read_reg_r(r);
                    value &= !(1 << b);
                    self.write_reg_r(r, value);
                }

                CB_Instruction::RL_n(n) => {
                    if self.print_instructions {
                        instruction_string.push_str(&format!("RL {}", reg_char(n)));
                    }
                    let mut value = self.read_reg_r(n);
                    let bit7 = value >> 7;
                    value <<= 1;
                    value += self.flag_c() as u8;

                    self.set_flag_c(bit7 == 1);
                    self.set_flag_z(value == 0);
                    self.set_flag_n(false);
                    self.set_flag_h(false);

                    self.write_reg_r(n, value);
                }
                CB_Instruction::RLC_n(n) => {
                    if self.print_instructions {
                        instruction_string.push_str(&format!("RLC {}", reg_char(n)));
                    }
                    let mut value = self.read_reg_r(n);
                    let bit7 = value >> 7;
                    value <<= 1;
                    value |= bit7;

                    self.set_flag_c(bit7 == 1);
                    self.set_flag_z(value == 0);
                    self.set_flag_n(false);
                    self.set_flag_h(false);

                    self.write_reg_r(n, value);
                }

                CB_Instruction::SLA_n(n) => {
                    if self.print_instructions {
                        instruction_string.push_str(&format!("SLA {}", reg_char(n)));
                    }
                    let mut value = self.read_reg_r(n);
                    let bit7 = value >> 7;
                    value <<= 1;

                    self.set_flag_c(bit7 == 1);
                    self.set_flag_z(value == 0);
                    self.set_flag_n(false);
                    self.set_flag_h(false);

                    self.write_reg_r(n, value);
                }
                CB_Instruction::RRC_n(n) => {
                    if self.print_instructions {
                        instruction_string.push_str(&format!("RRC {}", reg_char(n)));
                    }
                    let mut value = self.read_reg_r(n);
                    let bit0 = value & 0b1;
                    value >>= 1;
                    value |= (bit0) << 7;

                    self.set_flag_c(bit0 == 1);
                    self.set_flag_z(value == 0);
                    self.set_flag_n(false);
                    self.set_flag_h(false);

                    self.write_reg_r(n, value);
                }
                CB_Instruction::SRA_n(n) => {
                    if self.print_instructions {
                        instruction_string.push_str(&format!("SRA {}", reg_char(n)));
                    }
                    let mut value = self.read_reg_r(n);
                    let bit0 = value & 0b1;
                    let bit7 = value >> 7;
                    value >>= 1;
                    // bit 7 stays where it was
                    value |= (bit7) << 7;

                    self.set_flag_c(bit0 == 1);
                    self.set_flag_z(value == 0);
                    self.set_flag_n(false);
                    self.set_flag_h(false);

                    self.write_reg_r(n, value);
                }
                CB_Instruction::SRL_n(n) => {
                    if self.print_instructions {
                        instruction_string.push_str(&format!("SRL {}", reg_char(n)));
                    }
                    let mut value = self.read_reg_r(n);
                    let bit0 = value & 0b1;
                    value >>= 1;

                    self.set_flag_c(bit0 == 1);
                    self.set_flag_z(value == 0);
                    self.set_flag_n(false);
                    self.set_flag_h(false);

                    self.write_reg_r(n, value);
                }
                CB_Instruction::RR_n(n) => {
                    if self.print_instructions {
                        instruction_string.push_str(&format!("RR {}", reg_char(n)));
                    }
                    let mut value = self.read_reg_r(n);
                    let bit0 = value & 0b1;
                    value >>= 1;
                    value |= (self.flag_c() as u8) << 7;

                    self.set_flag_c(bit0 == 1);
                    self.set_flag_z(value == 0);
                    self.set_flag_n(false);
                    self.set_flag_h(false);

                    self.write_reg_r(n, value);
                }
                CB_Instruction::SWAP_n(n) => {
                    if self.print_instructions {
                        instruction_string.push_str(&format!("SWAP {}", reg_char(n)));
                    }
                    let mut value = self.read_reg_r(n);
                    let low = value & 0xF;
                    let high = (value & 0xF0) >> 4;
                    value = (low << 4) | high;

                    self.set_flag_z(value == 0);
                    self.set_flag_n(false);
                    self.set_flag_h(false);
                    self.set_flag_c(false);

                    self.write_reg_r(n, value);
                }
            }

            if self.print_instructions {
                self.send_instr_text(instruction_string);
            }
        }
    }

    fn push_stack(&mut self, value: u8) {
        self.reg_sp = self.reg_sp.wrapping_sub(1);
        self.write_mem(self.reg_sp, value);
    }
    fn push_stack_u16(&mut self, value: u16) {
        let (high, low) = u16_as_u8s(value);
        self.push_stack(high);
        self.push_stack(low);
    }
    fn pop_stack(&mut self) -> u8 {
        // Add first, so we are reading the old value
        // As reg_sp always points to the next empty spot
        let ret = self.read_mem(self.reg_sp);
        self.reg_sp = self.reg_sp.wrapping_add(1);
        ret
    }
    fn pop_stack_u16(&mut self) -> u16 {
        let low = self.pop_stack();
        let high = self.pop_stack();
        u8s_as_u16((high, low))
    }

    fn add_cycles(&mut self, amount: i32) {
        self.cycles += amount;
    }

    fn read_reg_r(&mut self, r: u8) -> u8 {
        match r {
            0 => self.reg_b,
            1 => self.reg_c,
            2 => self.reg_d,
            3 => self.reg_e,
            4 => self.reg_h,
            5 => self.reg_l,
            6 => self.read_mem(self.hl()),
            7 => self.reg_a,

            _ => panic!("Cpu::read_reg_r  Invalid r: {}", r),
        }
    }

    fn print_registers(&self) {
        print!("a: 0x{:02x}, ", self.reg_a);
        print!("f: 0x{:02x}, ", self.reg_f);
        print!("b: 0x{:02x}, ", self.reg_b);
        print!("c: 0x{:02x}, ", self.reg_c);
        print!("d: 0x{:02x}, ", self.reg_d);
        println!("e: 0x{:02x}", self.reg_e);
        print!("Flag Z: {}, ", self.flag_z());
        print!("Flag N: {}, ", self.flag_n());
        print!("Flag H: {}, ", self.flag_h());
        println!("Flag C: {}, ", self.flag_c());
        println!(
            "HL: {:04x}, PC: {:04x}, SP: {:04x}",
            self.hl(),
            self.reg_pc,
            self.reg_sp
        );
    }

    fn check_cc(&self, cc: u8) -> bool {
        match cc {
            0 => !self.flag_z(),
            1 => self.flag_z(),
            2 => !self.flag_c(),
            3 => self.flag_c(),
            _ => unreachable!(),
        }
    }
    fn read_mem(&mut self, address: u16) -> u8 {
        self.add_cycles(4);
        self.interconnect.read_mem(address)
    }

    fn write_mem(&mut self, address: u16, value: u8) {
        self.add_cycles(4);
        self.interconnect.write_mem(address, value);
    }

    fn write_reg_r(&mut self, r: u8, value: u8) {
        match r {
            0 => self.reg_b = value,
            1 => self.reg_c = value,
            2 => self.reg_d = value,
            3 => self.reg_e = value,
            4 => self.reg_h = value,
            5 => self.reg_l = value,
            6 => self.write_mem(self.hl(), value),
            7 => self.reg_a = value,

            _ => panic!("Cpu::read_reg_r  Invalid r: {}", r),
        }
    }

    fn read_byte(&mut self) -> u8 {
        self.add_cycles(4);
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

    #[inline(always)]
    fn af(&self) -> u16 {
        u8s_as_u16((self.reg_a, self.reg_f))
    }

    #[inline(always)]
    fn flag_z(&self) -> bool {
        (self.reg_f & 0x80) > 0
    }
    #[inline(always)]
    fn flag_n(&self) -> bool {
        (self.reg_f & 0x40) > 0
    }
    #[inline(always)]
    fn flag_h(&self) -> bool {
        (self.reg_f & 0x20) > 0
    }
    #[inline(always)]
    fn flag_c(&self) -> bool {
        (self.reg_f & 0x10) > 0
    }

    #[inline(always)]
    fn set_flag_z(&mut self, value: bool) {
        if value {
            self.reg_f |= 1 << 7;
        } else {
            self.reg_f &= !(1 << 7);
        }
    }
    #[inline(always)]
    fn set_flag_n(&mut self, value: bool) {
        if value {
            self.reg_f |= 1 << 6;
        } else {
            self.reg_f &= !(1 << 6);
        }
    }
    #[inline(always)]
    fn set_flag_h(&mut self, value: bool) {
        if value {
            self.reg_f |= 1 << 5;
        } else {
            self.reg_f &= !(1 << 5);
        }
    }
    #[inline(always)]
    fn set_flag_c(&mut self, value: bool) {
        if value {
            self.reg_f |= 1 << 4;
        } else {
            self.reg_f &= !(1 << 4);
        }
    }

    #[inline(always)]
    fn bc(&self) -> u16 {
        u8s_as_u16((self.reg_b, self.reg_c))
    }

    #[inline(always)]
    fn de(&self) -> u16 {
        u8s_as_u16((self.reg_d, self.reg_e))
    }

    #[inline(always)]
    fn hl(&self) -> u16 {
        u8s_as_u16((self.reg_h, self.reg_l))
    }

    fn set_af(&mut self, val: u16) {
        let (h, l) = u16_as_u8s(val);
        self.reg_a = h;
        self.reg_f = l & 0xF0;
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

    pub fn set_print_instruction(&mut self, b: bool) {
        self.print_instructions = b;
    }
    pub fn set_console_tx(&mut self, tx: mpsc::Sender<CpuText>) {
        self.console_tx = Some(tx);
    }

    pub fn reset_console_tx(&mut self) {
        self.console_tx = None;
    }
}

fn reg_char(r: u8) -> &'static str {
    match r {
        0 => "B",
        1 => "C",
        2 => "D",
        3 => "E",
        4 => "H",
        5 => "L",
        6 => "(HL)",
        7 => "A",

        _ => panic!("Cpu::read_reg_r  Invalid r: {}", r),
    }
}

fn cc_to_char(cc: u8) -> &'static str {
    match cc {
        0 => "NZ",
        1 => "Z",
        2 => "NC",
        3 => "C",
        _ => unreachable!(),
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
