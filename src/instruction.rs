use enum_primitive_derive::*;
use num_traits::{FromPrimitive, ToPrimitive};

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Primitive)]
// The enum value is the only one, or one of many on the list
pub enum Instruction {
    LD_nn_n = 0x2E,
    LD_r1_r2 = 0x40,
    LD_HL_ptr_r2 = 0x70,
    LD_n_A = 0x47,
    LD_A_n = 0x1A,
    LD_n_nn = 0x31,
    LD_Cptr_A = 0xE2,
    LD_A_Cptr = 0xF2,
    LDD_HLptr_A = 0x32, // LDD (HL),A
    LDD_A_HLptr = 0x3A,
    LDI_A_HLptr = 0x2A,
    LDI_HLptr_A = 0x22,
    LDH_A_nptr = 0xF0,
    LDH_nptr_A = 0xE0,
    LD_SP_HL = 0xF9,
    LDHL_SP_n = 0xF8,
    LD_nnptr_SP = 0x08,

    ADD_A_n = 0x87,
    ADC_A_n = 0x88,
    ADD_HL_n = 0x09,
    ADD_SP_n = 0xE8,
    SUB_n = 0x90,
    SBC_A_n = 0x98,
    AND_n = 0xA0,
    OR_n = 0xB0,
    XOR_n = 0xAF,
    CP_n = 0xFE,
    RLA = 0x17,

    INC_n = 0x3C,
    INC_nn = 0x23,
    DEC_n = 0x25,
    DEC_nn = 0x0B,

    JP_nn = 0xC3,
    JP_HLptr = 0xE9,
    JP_cc_nn = 0xC2,
    JR_cc_n = 0x38,
    JR_n = 0x18,

    CALL_cc_nn = 0xC4,
    CALL_nn = 0xCD,
    RET = 0xC9,
    RET_cc = 0xC0,

    PUSH_nn = 0xF5,
    POP_nn = 0xE1,

    CB = 0xcb,
    NOP = 0x00,
    DI = 0xF3,
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
pub enum CB_Instruction {
    BIT_b_r(u8, u8),
    RL_n(u8),
    SWAP_n(u8),
}

pub fn parse(byte: u8) -> Option<Instruction> {
    match byte {
        0xA8...0xAF | 0xEE => Some(Instruction::XOR_n),
        0x01 | 0x11 | 0x21 | 0x31 => Some(Instruction::LD_n_nn),
        0x78...0x7F | 0x0A | 0x1A | 0xFA | 0x3E => Some(Instruction::LD_A_n),
        0x47 | 0x4F | 0x57 | 0x5F | 0x67 | 0x6F | 0x77 | 0x7F | 0x02 | 0x12 | 0xEA => {
            Some(Instruction::LD_n_A)
        }
        0x70...0x75 | 0x36 => Some(Instruction::LD_HL_ptr_r2),
        0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E => Some(Instruction::LD_nn_n),
        0x20 | 0x28 | 0x30 | 0x38 => Some(Instruction::JR_cc_n),
        0x04 | 0x0C | 0x14 | 0x1C | 0x24 | 0x2C | 0x34 | 0x3C => Some(Instruction::INC_n),
        0x80...0x87 | 0xC6 => Some(Instruction::ADD_A_n),
        0x88...0x8F | 0xCE => Some(Instruction::ADC_A_n),
        0xF5 | 0xC5 | 0xD5 | 0xE5 => Some(Instruction::PUSH_nn),
        0xF1 | 0xC1 | 0xD1 | 0xE1 => Some(Instruction::POP_nn),
        0x40...0x46 | 0x48...0x4E | 0x50...0x56 | 0x58...0x5E | 0x60...0x66 | 0x68...0x6E => {
            Some(Instruction::LD_r1_r2)
        }
        0x05 | 0x0D | 0x15 | 0x1D | 0x25 | 0x2D | 0x35 | 0x3D => Some(Instruction::DEC_n),
        0x0B | 0x1B | 0x2B | 0x3B => Some(Instruction::DEC_nn),
        0x03 | 0x13 | 0x23 | 0x33 => Some(Instruction::INC_nn),
        0xC0 | 0xC8 | 0xD0 | 0xD8 => Some(Instruction::RET_cc),
        0xC4 | 0xCC | 0xD4 | 0xDC => Some(Instruction::CALL_cc_nn),
        0xB8...0xBF | 0xFE => Some(Instruction::CP_n),
        0xC2 | 0xCA | 0xD2 | 0xDA => Some(Instruction::JP_cc_nn),
        0x90...0x97 | 0xD6 => Some(Instruction::SUB_n),
        0x98...0x9F | 0xDE => Some(Instruction::SBC_A_n),
        0xA0...0xA7 | 0xE6 => Some(Instruction::AND_n),
        0xB0...0xB7 | 0xF6 => Some(Instruction::OR_n),
        0x09 | 0x19 | 0x29 | 0x39 => Some(Instruction::ADD_HL_n),
        _ => Instruction::from_u8(byte),
    }
}

pub fn parse_cb(byte: u8) -> Option<CB_Instruction> {
    match byte {
        0x40...0x7F => {
            let byte = byte - 0x40;
            let b = byte / 8;
            let r = byte % 8;
            Some(CB_Instruction::BIT_b_r(b, r))
        }
        0x10...0x17 => {
            let byte = byte - 0x10;
            Some(CB_Instruction::RL_n(byte))
        }
        0x30...0x37 => {
            let byte = byte - 0x30;
            Some(CB_Instruction::SWAP_n(byte))
        }
        _ => None,
    }
}

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_LD_nn_n_parse() {
        assert_eq!(parse(0x06).unwrap(), Instruction::LD_nn_n);
        assert_eq!(parse(0x0E).unwrap(), Instruction::LD_nn_n);
        assert_eq!(parse(0x16).unwrap(), Instruction::LD_nn_n);
        assert_eq!(parse(0x1E).unwrap(), Instruction::LD_nn_n);
        assert_eq!(parse(0x26).unwrap(), Instruction::LD_nn_n);
        assert_eq!(parse(0x2E).unwrap(), Instruction::LD_nn_n);

        assert_eq!(parse(0x40).unwrap(), Instruction::LD_r1_r2);
        assert_eq!(parse(0x41).unwrap(), Instruction::LD_r1_r2);
        assert_eq!(parse(0x42).unwrap(), Instruction::LD_r1_r2);
        assert_eq!(parse(0x43).unwrap(), Instruction::LD_r1_r2);
        assert_eq!(parse(0x44).unwrap(), Instruction::LD_r1_r2);
        assert_eq!(parse(0x45).unwrap(), Instruction::LD_r1_r2);
        assert_eq!(parse(0x46).unwrap(), Instruction::LD_r1_r2);

        assert_eq!(parse(0x48).unwrap(), Instruction::LD_r1_r2);
        assert_eq!(parse(0x49).unwrap(), Instruction::LD_r1_r2);
        assert_eq!(parse(0x4A).unwrap(), Instruction::LD_r1_r2);
        assert_eq!(parse(0x4B).unwrap(), Instruction::LD_r1_r2);
        assert_eq!(parse(0x4C).unwrap(), Instruction::LD_r1_r2);
        assert_eq!(parse(0x4D).unwrap(), Instruction::LD_r1_r2);
        assert_eq!(parse(0x4E).unwrap(), Instruction::LD_r1_r2);

        assert_eq!(parse(0x50).unwrap(), Instruction::LD_r1_r2);
        assert_eq!(parse(0x51).unwrap(), Instruction::LD_r1_r2);
        assert_eq!(parse(0x52).unwrap(), Instruction::LD_r1_r2);
        assert_eq!(parse(0x53).unwrap(), Instruction::LD_r1_r2);
        assert_eq!(parse(0x54).unwrap(), Instruction::LD_r1_r2);
        assert_eq!(parse(0x55).unwrap(), Instruction::LD_r1_r2);
        assert_eq!(parse(0x56).unwrap(), Instruction::LD_r1_r2);

        assert_eq!(parse(0x58).unwrap(), Instruction::LD_r1_r2);
        assert_eq!(parse(0x59).unwrap(), Instruction::LD_r1_r2);
        assert_eq!(parse(0x5A).unwrap(), Instruction::LD_r1_r2);
        assert_eq!(parse(0x5B).unwrap(), Instruction::LD_r1_r2);
        assert_eq!(parse(0x5C).unwrap(), Instruction::LD_r1_r2);
        assert_eq!(parse(0x5D).unwrap(), Instruction::LD_r1_r2);
        assert_eq!(parse(0x5E).unwrap(), Instruction::LD_r1_r2);

        assert_eq!(parse(0x60).unwrap(), Instruction::LD_r1_r2);
        assert_eq!(parse(0x61).unwrap(), Instruction::LD_r1_r2);
        assert_eq!(parse(0x62).unwrap(), Instruction::LD_r1_r2);
        assert_eq!(parse(0x63).unwrap(), Instruction::LD_r1_r2);
        assert_eq!(parse(0x64).unwrap(), Instruction::LD_r1_r2);
        assert_eq!(parse(0x65).unwrap(), Instruction::LD_r1_r2);
        assert_eq!(parse(0x66).unwrap(), Instruction::LD_r1_r2);

        assert_eq!(parse(0x68).unwrap(), Instruction::LD_r1_r2);
        assert_eq!(parse(0x69).unwrap(), Instruction::LD_r1_r2);
        assert_eq!(parse(0x6A).unwrap(), Instruction::LD_r1_r2);
        assert_eq!(parse(0x6B).unwrap(), Instruction::LD_r1_r2);
        assert_eq!(parse(0x6C).unwrap(), Instruction::LD_r1_r2);
        assert_eq!(parse(0x6D).unwrap(), Instruction::LD_r1_r2);
        assert_eq!(parse(0x6E).unwrap(), Instruction::LD_r1_r2);

        assert_eq!(parse(0x70).unwrap(), Instruction::LD_HL_ptr_r2);
        assert_eq!(parse(0x71).unwrap(), Instruction::LD_HL_ptr_r2);
        assert_eq!(parse(0x72).unwrap(), Instruction::LD_HL_ptr_r2);
        assert_eq!(parse(0x73).unwrap(), Instruction::LD_HL_ptr_r2);
        assert_eq!(parse(0x74).unwrap(), Instruction::LD_HL_ptr_r2);
        assert_eq!(parse(0x75).unwrap(), Instruction::LD_HL_ptr_r2);
        assert_eq!(parse(0x36).unwrap(), Instruction::LD_HL_ptr_r2);

        assert_eq!(parse(0x7F).unwrap(), Instruction::LD_A_n);
        assert_eq!(parse(0x78).unwrap(), Instruction::LD_A_n);
        assert_eq!(parse(0x79).unwrap(), Instruction::LD_A_n);
        assert_eq!(parse(0x7A).unwrap(), Instruction::LD_A_n);
        assert_eq!(parse(0x7B).unwrap(), Instruction::LD_A_n);
        assert_eq!(parse(0x7C).unwrap(), Instruction::LD_A_n);
        assert_eq!(parse(0x7D).unwrap(), Instruction::LD_A_n);
        assert_eq!(parse(0x0A).unwrap(), Instruction::LD_A_n);
        assert_eq!(parse(0x1A).unwrap(), Instruction::LD_A_n);
        assert_eq!(parse(0x7E).unwrap(), Instruction::LD_A_n);
        assert_eq!(parse(0xFA).unwrap(), Instruction::LD_A_n);
        assert_eq!(parse(0x3E).unwrap(), Instruction::LD_A_n);

        assert_eq!(parse(0x47).unwrap(), Instruction::LD_n_A);
        assert_eq!(parse(0x4F).unwrap(), Instruction::LD_n_A);
        assert_eq!(parse(0x57).unwrap(), Instruction::LD_n_A);
        assert_eq!(parse(0x5F).unwrap(), Instruction::LD_n_A);
        assert_eq!(parse(0x67).unwrap(), Instruction::LD_n_A);
        assert_eq!(parse(0x6F).unwrap(), Instruction::LD_n_A);
        assert_eq!(parse(0x02).unwrap(), Instruction::LD_n_A);
        assert_eq!(parse(0x12).unwrap(), Instruction::LD_n_A);
        assert_eq!(parse(0x77).unwrap(), Instruction::LD_n_A);
        assert_eq!(parse(0xEA).unwrap(), Instruction::LD_n_A);

        assert_eq!(parse(0xF2).unwrap(), Instruction::LD_A_Cptr);

        assert_eq!(parse(0xE2).unwrap(), Instruction::LD_Cptr_A);

        assert_eq!(parse(0x3A).unwrap(), Instruction::LDD_A_HLptr);

        assert_eq!(parse(0x32).unwrap(), Instruction::LDD_HLptr_A);

        assert_eq!(parse(0x2A).unwrap(), Instruction::LDI_A_HLptr);

        assert_eq!(parse(0x22).unwrap(), Instruction::LDI_HLptr_A);

        assert_eq!(parse(0xE0).unwrap(), Instruction::LDH_nptr_A);
        assert_eq!(parse(0xF0).unwrap(), Instruction::LDH_A_nptr);

        assert_eq!(parse(0x01).unwrap(), Instruction::LD_n_nn);
        assert_eq!(parse(0x11).unwrap(), Instruction::LD_n_nn);
        assert_eq!(parse(0x21).unwrap(), Instruction::LD_n_nn);
        assert_eq!(parse(0x31).unwrap(), Instruction::LD_n_nn);

        assert_eq!(parse(0xF9).unwrap(), Instruction::LD_SP_HL);
        assert_eq!(parse(0xF8).unwrap(), Instruction::LDHL_SP_n);

        assert_eq!(parse(0x08).unwrap(), Instruction::LD_nnptr_SP);

        assert_eq!(parse(0xF5).unwrap(), Instruction::PUSH_nn);
        assert_eq!(parse(0xC5).unwrap(), Instruction::PUSH_nn);
        assert_eq!(parse(0xD5).unwrap(), Instruction::PUSH_nn);
        assert_eq!(parse(0xE5).unwrap(), Instruction::PUSH_nn);

        assert_eq!(parse(0xF1).unwrap(), Instruction::POP_nn);
        assert_eq!(parse(0xC1).unwrap(), Instruction::POP_nn);
        assert_eq!(parse(0xD1).unwrap(), Instruction::POP_nn);
        assert_eq!(parse(0xE1).unwrap(), Instruction::POP_nn);

        assert_eq!(parse(0x87).unwrap(), Instruction::ADD_A_n);
        assert_eq!(parse(0x80).unwrap(), Instruction::ADD_A_n);
        assert_eq!(parse(0x81).unwrap(), Instruction::ADD_A_n);
        assert_eq!(parse(0x82).unwrap(), Instruction::ADD_A_n);
        assert_eq!(parse(0x83).unwrap(), Instruction::ADD_A_n);
        assert_eq!(parse(0x84).unwrap(), Instruction::ADD_A_n);
        assert_eq!(parse(0x85).unwrap(), Instruction::ADD_A_n);
        assert_eq!(parse(0x86).unwrap(), Instruction::ADD_A_n);
        assert_eq!(parse(0xC6).unwrap(), Instruction::ADD_A_n);

        assert_eq!(parse(0x8F).unwrap(), Instruction::ADC_A_n);
        assert_eq!(parse(0x88).unwrap(), Instruction::ADC_A_n);
        assert_eq!(parse(0x89).unwrap(), Instruction::ADC_A_n);
        assert_eq!(parse(0x8A).unwrap(), Instruction::ADC_A_n);
        assert_eq!(parse(0x8B).unwrap(), Instruction::ADC_A_n);
        assert_eq!(parse(0x8C).unwrap(), Instruction::ADC_A_n);
        assert_eq!(parse(0x8D).unwrap(), Instruction::ADC_A_n);
        assert_eq!(parse(0x8E).unwrap(), Instruction::ADC_A_n);
        assert_eq!(parse(0xCE).unwrap(), Instruction::ADC_A_n);

        assert_eq!(parse(0x97).unwrap(), Instruction::SUB_n);
        assert_eq!(parse(0x90).unwrap(), Instruction::SUB_n);
        assert_eq!(parse(0x91).unwrap(), Instruction::SUB_n);
        assert_eq!(parse(0x92).unwrap(), Instruction::SUB_n);
        assert_eq!(parse(0x93).unwrap(), Instruction::SUB_n);
        assert_eq!(parse(0x94).unwrap(), Instruction::SUB_n);
        assert_eq!(parse(0x95).unwrap(), Instruction::SUB_n);
        assert_eq!(parse(0x96).unwrap(), Instruction::SUB_n);
        assert_eq!(parse(0xD6).unwrap(), Instruction::SUB_n);

        assert_eq!(parse(0x9F).unwrap(), Instruction::SBC_A_n);
        assert_eq!(parse(0x98).unwrap(), Instruction::SBC_A_n);
        assert_eq!(parse(0x99).unwrap(), Instruction::SBC_A_n);
        assert_eq!(parse(0x9A).unwrap(), Instruction::SBC_A_n);
        assert_eq!(parse(0x9B).unwrap(), Instruction::SBC_A_n);
        assert_eq!(parse(0x9C).unwrap(), Instruction::SBC_A_n);
        assert_eq!(parse(0x9D).unwrap(), Instruction::SBC_A_n);
        assert_eq!(parse(0x9E).unwrap(), Instruction::SBC_A_n);
        assert_eq!(parse(0xDE).unwrap(), Instruction::SBC_A_n);

        assert_eq!(parse(0xA7).unwrap(), Instruction::AND_n);
        assert_eq!(parse(0xA0).unwrap(), Instruction::AND_n);
        assert_eq!(parse(0xA1).unwrap(), Instruction::AND_n);
        assert_eq!(parse(0xA2).unwrap(), Instruction::AND_n);
        assert_eq!(parse(0xA3).unwrap(), Instruction::AND_n);
        assert_eq!(parse(0xA4).unwrap(), Instruction::AND_n);
        assert_eq!(parse(0xA5).unwrap(), Instruction::AND_n);
        assert_eq!(parse(0xA6).unwrap(), Instruction::AND_n);
        assert_eq!(parse(0xE6).unwrap(), Instruction::AND_n);

        assert_eq!(parse(0xB7).unwrap(), Instruction::OR_n);
        assert_eq!(parse(0xB0).unwrap(), Instruction::OR_n);
        assert_eq!(parse(0xB1).unwrap(), Instruction::OR_n);
        assert_eq!(parse(0xB2).unwrap(), Instruction::OR_n);
        assert_eq!(parse(0xB3).unwrap(), Instruction::OR_n);
        assert_eq!(parse(0xB4).unwrap(), Instruction::OR_n);
        assert_eq!(parse(0xB5).unwrap(), Instruction::OR_n);
        assert_eq!(parse(0xB6).unwrap(), Instruction::OR_n);
        assert_eq!(parse(0xF6).unwrap(), Instruction::OR_n);

        assert_eq!(parse(0xAF).unwrap(), Instruction::XOR_n);
        assert_eq!(parse(0xA8).unwrap(), Instruction::XOR_n);
        assert_eq!(parse(0xA9).unwrap(), Instruction::XOR_n);
        assert_eq!(parse(0xAA).unwrap(), Instruction::XOR_n);
        assert_eq!(parse(0xAB).unwrap(), Instruction::XOR_n);
        assert_eq!(parse(0xAC).unwrap(), Instruction::XOR_n);
        assert_eq!(parse(0xAD).unwrap(), Instruction::XOR_n);
        assert_eq!(parse(0xAE).unwrap(), Instruction::XOR_n);
        assert_eq!(parse(0xEE).unwrap(), Instruction::XOR_n);

        assert_eq!(parse(0xBF).unwrap(), Instruction::CP_n);
        assert_eq!(parse(0xB8).unwrap(), Instruction::CP_n);
        assert_eq!(parse(0xB9).unwrap(), Instruction::CP_n);
        assert_eq!(parse(0xBA).unwrap(), Instruction::CP_n);
        assert_eq!(parse(0xBB).unwrap(), Instruction::CP_n);
        assert_eq!(parse(0xBC).unwrap(), Instruction::CP_n);
        assert_eq!(parse(0xBD).unwrap(), Instruction::CP_n);
        assert_eq!(parse(0xBE).unwrap(), Instruction::CP_n);
        assert_eq!(parse(0xFE).unwrap(), Instruction::CP_n);

        assert_eq!(parse(0x3C).unwrap(), Instruction::INC_n);
        assert_eq!(parse(0x04).unwrap(), Instruction::INC_n);
        assert_eq!(parse(0x0C).unwrap(), Instruction::INC_n);
        assert_eq!(parse(0x14).unwrap(), Instruction::INC_n);
        assert_eq!(parse(0x1C).unwrap(), Instruction::INC_n);
        assert_eq!(parse(0x24).unwrap(), Instruction::INC_n);
        assert_eq!(parse(0x2C).unwrap(), Instruction::INC_n);
        assert_eq!(parse(0x34).unwrap(), Instruction::INC_n);

        assert_eq!(parse(0x3D).unwrap(), Instruction::DEC_n);
        assert_eq!(parse(0x05).unwrap(), Instruction::DEC_n);
        assert_eq!(parse(0x0D).unwrap(), Instruction::DEC_n);
        assert_eq!(parse(0x15).unwrap(), Instruction::DEC_n);
        assert_eq!(parse(0x1D).unwrap(), Instruction::DEC_n);
        assert_eq!(parse(0x25).unwrap(), Instruction::DEC_n);
        assert_eq!(parse(0x2D).unwrap(), Instruction::DEC_n);
        assert_eq!(parse(0x35).unwrap(), Instruction::DEC_n);

        assert_eq!(parse(0x09).unwrap(), Instruction::ADD_HL_n);
        assert_eq!(parse(0x19).unwrap(), Instruction::ADD_HL_n);
        assert_eq!(parse(0x29).unwrap(), Instruction::ADD_HL_n);
        assert_eq!(parse(0x39).unwrap(), Instruction::ADD_HL_n);

        assert_eq!(parse(0xE8).unwrap(), Instruction::ADD_SP_n);

        assert_eq!(parse(0x03).unwrap(), Instruction::INC_nn);
        assert_eq!(parse(0x13).unwrap(), Instruction::INC_nn);
        assert_eq!(parse(0x23).unwrap(), Instruction::INC_nn);
        assert_eq!(parse(0x33).unwrap(), Instruction::INC_nn);

        assert_eq!(parse(0x0B).unwrap(), Instruction::DEC_nn);
        assert_eq!(parse(0x1B).unwrap(), Instruction::DEC_nn);
        assert_eq!(parse(0x2B).unwrap(), Instruction::DEC_nn);
        assert_eq!(parse(0x3B).unwrap(), Instruction::DEC_nn);

        assert_eq!(parse(0x0).unwrap(), Instruction::NOP);

        assert_eq!(parse(0x17).unwrap(), Instruction::RLA);

        assert_eq!(parse(0xC3).unwrap(), Instruction::JP_nn);

        assert_eq!(parse(0xC2).unwrap(), Instruction::JP_cc_nn);
        assert_eq!(parse(0xCA).unwrap(), Instruction::JP_cc_nn);
        assert_eq!(parse(0xD2).unwrap(), Instruction::JP_cc_nn);
        assert_eq!(parse(0xDA).unwrap(), Instruction::JP_cc_nn);

        assert_eq!(parse(0xE9).unwrap(), Instruction::JP_HLptr);
        assert_eq!(parse(0x18).unwrap(), Instruction::JR_n);

        assert_eq!(parse(0x20).unwrap(), Instruction::JR_cc_n);
        assert_eq!(parse(0x28).unwrap(), Instruction::JR_cc_n);
        assert_eq!(parse(0x30).unwrap(), Instruction::JR_cc_n);
        assert_eq!(parse(0x38).unwrap(), Instruction::JR_cc_n);

        assert_eq!(parse(0xCD).unwrap(), Instruction::CALL_nn);
        assert_eq!(parse(0xC4).unwrap(), Instruction::CALL_cc_nn);
        assert_eq!(parse(0xCC).unwrap(), Instruction::CALL_cc_nn);
        assert_eq!(parse(0xD4).unwrap(), Instruction::CALL_cc_nn);
        assert_eq!(parse(0xDC).unwrap(), Instruction::CALL_cc_nn);
    }
}
