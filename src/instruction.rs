#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
// The enum value is the only one, or one of many on the list
pub enum Instruction {
    LD_r1_n(u8),
    LD_r1_r2(u8, u8),
    LD_A_nnptr,
    LD_nnptr_A,
    LD_A_Cptr,
    LD_Cptr_A,

    LDD_A_HLptr,
    LDD_HLptr_A,
    LDI_A_HLptr,
    LDI_HLptr_A,

    LDH_nptr_A,
    LDH_A_nptr,

    LD_rr_nn,
    LD_SP_HL,
    LDHL_SPn,
    LD_nn_SP,

    PUSH_nn,
    POP_nn,

    ADD_n(u8),
    ADC_n(u8),
    SUB_n(u8),
    SBC_n(u8),
    AND_n(u8),
    OR_n(u8),
    XOR_n(u8),
    CP_n(u8),
    INC_n(u8),
    DEC_n(u8),

    ADD_HL_nn(u8),
    ADD_SP_n,
    INC_nn(u8),
    DEC_nn(u8),

    DAA,
    CPL,
    CCF,
    SCF,
    NOP,
    HALT,
    STOP,
    DI,
    EI,

    RLCA,
    RLA,
    RRCA,
    RRA,
    CB,

    JP_nn,
    JP_cc_nn(u8),
    JP_HLptr,
    JR_n,
    JR_cc_n(u8),

    CALL_nn,
    CALL_cc_nn(u8),

    RST_n(u8),
    RET,
    RET_cc(u8),
    RETI,
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
pub enum CB_Instruction {
    BIT_b_r(u8, u8),
    RES_b_r(u8, u8),
    SET_b_r(u8, u8),

    RL_n(u8),
    RLC_n(u8),
    RR_n(u8),
    RRC_n(u8),
    SLA_n(u8),
    SRA_n(u8),
    SRL_n(u8),

    SWAP_n(u8),
}

pub fn parse(byte: u8) -> Option<Instruction> {
    match byte {
        0x40...0x7F => {
            let r1 = (byte >> 3) & 7;
            let r2 = byte & 7;
            Some(Instruction::LD_r1_r2(r1, r2))
        }
        0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E => {
            Some(Instruction::LD_r1_n((byte - 0x06) / 8))
        }
        0x0A | 0x1A | 0xFA => Some(Instruction::LD_A_nnptr),
        0x02 | 0x12 | 0xEA => Some(Instruction::LD_nnptr_A),
        0xF2 => Some(Instruction::LD_A_Cptr),
        0xE2 => Some(Instruction::LD_Cptr_A),

        0x3A => Some(Instruction::LDD_A_HLptr),
        0x32 => Some(Instruction::LDD_HLptr_A),
        0x2A => Some(Instruction::LDI_A_HLptr),
        0x22 => Some(Instruction::LDI_HLptr_A),

        0xE0 => Some(Instruction::LDH_nptr_A),
        0xF0 => Some(Instruction::LDH_A_nptr),

        0x01 | 0x11 | 0x21 | 0x31 => Some(Instruction::LD_rr_nn),

        0xF9 => Some(Instruction::LD_SP_HL),
        0xF8 => Some(Instruction::LDHL_SPn),
        0x08 => Some(Instruction::LD_nn_SP),

        0xF5 | 0xC5 | 0xD5 | 0xE5 => Some(Instruction::PUSH_nn),
        0xF1 | 0xC1 | 0xD1 | 0xE1 => Some(Instruction::POP_nn),

        0x80...0x87 => Some(Instruction::ADD_n(byte & 7)),
        0xC6 => Some(Instruction::ADD_n(8)),
        0x88...0x8f => Some(Instruction::ADC_n(byte & 7)),
        0xCE => Some(Instruction::ADC_n(8)),
        0x90...0x97 => Some(Instruction::SUB_n(byte & 7)),
        0xD6 => Some(Instruction::SUB_n(8)),
        0x98...0x9f => Some(Instruction::SBC_n(byte & 7)),
        0xDE => Some(Instruction::SBC_n(8)),
        0xA0...0xA7 => Some(Instruction::AND_n(byte & 7)),
        0xE6 => Some(Instruction::AND_n(8)),
        0xA8...0xAF => Some(Instruction::XOR_n(byte & 7)),
        0xEE => Some(Instruction::XOR_n(8)),
        0xB0...0xB7 => Some(Instruction::OR_n(byte & 7)),
        0xF6 => Some(Instruction::OR_n(8)),
        0xB8...0xBF => Some(Instruction::CP_n(byte & 7)),
        0xFE => Some(Instruction::CP_n(8)),
        0x04 | 0x0C | 0x14 | 0x1C | 0x24 | 0x2C | 0x34 | 0x3C => {
            Some(Instruction::INC_n((byte - 0x04) / 8))
        }
        0x05 | 0x0D | 0x15 | 0x1D | 0x25 | 0x2D | 0x35 | 0x3D => {
            Some(Instruction::DEC_n((byte - 0x04) / 8))
        }

        0x09 | 0x19 | 0x29 | 0x39 => Some(Instruction::ADD_HL_nn((byte - 0x09) / 0x10)),
        0xE8 => Some(Instruction::ADD_SP_n),
        0x03 | 0x13 | 0x23 | 0x33 => Some(Instruction::INC_nn((byte - 0x03) / 0x10)),
        0x0B | 0x1B | 0x2B | 0x3B => Some(Instruction::DEC_nn((byte - 0x0B) / 0x10)),

        0x27 => Some(Instruction::DAA),
        0x2F => Some(Instruction::CPL),
        0x3F => Some(Instruction::CCF),
        0x37 => Some(Instruction::SCF),
        0x00 => Some(Instruction::NOP),
        0x76 => Some(Instruction::HALT),
        0x10 => Some(Instruction::STOP),
        0xF3 => Some(Instruction::DI),
        0xFB => Some(Instruction::EI),

        0x07 => Some(Instruction::RLCA),
        0x17 => Some(Instruction::RLA),
        0x0F => Some(Instruction::RRCA),
        0x1F => Some(Instruction::RRA),

        0xCB => Some(Instruction::CB),

        0xC3 => Some(Instruction::JP_nn),
        0xC2 | 0xCA | 0xD2 | 0xDA => Some(Instruction::JP_cc_nn((byte - 0xC2) / 8)),
        0xE9 => Some(Instruction::JP_HLptr),
        0x18 => Some(Instruction::JR_n),
        0x20 | 0x28 | 0x30 | 0x38 => Some(Instruction::JR_cc_n((byte - 0x20) / 8)),
        0xCD => Some(Instruction::CALL_nn),
        0xC4 | 0xCC | 0xD4 | 0xDC => Some(Instruction::CALL_cc_nn((byte - 0xC4) / 8)),
        0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF => {
            Some(Instruction::RST_n(byte - 0xC7))
        }
        0xC9 => Some(Instruction::RET),
        0xC0 | 0xC8 | 0xD0 | 0xD8 => Some(Instruction::RET_cc((byte - 0xC0) / 8)),
        0xD9 => Some(Instruction::RETI),
        _ => None,
    }
}

pub fn parse_cb(byte: u8) -> CB_Instruction {
    match byte {
        0x00...0x07 => {
            let byte = byte - 0x00;
            CB_Instruction::RLC_n(byte)
        }
        0x08...0x0F => {
            let byte = byte - 0x08;
            CB_Instruction::RRC_n(byte)
        }
        0x10...0x17 => {
            let byte = byte - 0x10;
            CB_Instruction::RL_n(byte)
        }
        0x18...0x1F => {
            let byte = byte - 0x18;
            CB_Instruction::RR_n(byte)
        }
        0x20...0x27 => {
            let byte = byte - 0x20;
            CB_Instruction::SLA_n(byte)
        }
        0x28...0x2F => {
            let byte = byte - 0x28;
            CB_Instruction::SRA_n(byte)
        }
        0x30...0x37 => {
            let byte = byte - 0x30;
            CB_Instruction::SWAP_n(byte)
        }
        0x38...0x3F => {
            let byte = byte - 0x38;
            CB_Instruction::SRL_n(byte)
        }
        0x40...0x7F => {
            let byte = byte - 0x40;
            let b = byte / 8;
            let r = byte % 8;
            CB_Instruction::BIT_b_r(b, r)
        }
        0x80...0xBF => {
            let byte = byte - 0x80;
            let b = byte / 8;
            let r = byte % 8;
            CB_Instruction::RES_b_r(b, r)
        }
        0xC0...0xFF => {
            let byte = byte - 0xC0;
            let b = byte / 8;
            let r = byte % 8;
            CB_Instruction::SET_b_r(b, r)
        }
        _ => unreachable!(),
    }
}
