/*

Name - NR 52 (Value at reset: $F1-GB, $F0-SGB)
 Contents - Sound on/off (R/W)


Bit 7 - All sound on/off
 0: stop all sound circuits
 1: operate all sound circuits
 Bit 3 - Sound 4 ON flag
 Bit 2 - Sound 3 ON flag
 Bit 1 - Sound 2 ON flag
 Bit 0 - Sound 1 ON flag
 Bits 0 - 3 of this register are meant to
 be status bits to be read.  Writing to
 these bits does NOT enable/disable
 sound.


 10. FF11 (NR 11)
 Name - NR 11
 Contents - Sound Mode 1 register, Sound length/Wave
 pattern duty (R/W)
 Only Bits 7-6 can be read.
 Bit 7-6 - Wave Pattern Duty
 Bit 5-0 - Sound length data (t1: 0-63)
 Wave Duty: (default: 10)
 00: 12.5% ( _--------_--------_-------- )
 01: 25% ( __-------__-------__------- )
 10: 50% ( ____-----____-----____----- )
 11: 75% ( ______---______---______--- )
 Sound Length = (64-t1)*(1/256) seconds

11. FF12 (NR12)
 Name - NR 12
 Contents - Sound Mode 1 register, Envelope (R/W)
 Bit 7-4 - Initial volume of envelope
 Bit 3 - Envelope UP/DOWN
 0: Attenuate
 1: Amplify
 Bit 2-0 - Number of envelope sweep
 (n: 0-7) (If zero, stop
 envelope operation.)
 Initial volume of envelope is from 0 to
 $F. Zero being no sound.
 Length of 1 step = n*(1/64) seconds

28. FF25 (NR 51)
 Name - NR 51
 Contents - Selection of Sound output terminal (R/W)
 Bit 7 - Output sound 4 to SO2 terminal
 Bit 6 - Output sound 3 to SO2 terminal
 Bit 5 - Output sound 2 to SO2 terminal
 Bit 4 - Output sound 1 to SO2 terminal
 Bit 3 - Output sound 4 to SO1 terminal
 Bit 2 - Output sound 3 to SO1 terminal
 Bit 1 - Output sound 2 to SO1 terminal
 Bit 0 - Output sound 1 to SO1 terminal


27. FF24 (NR 50)
 Name - NR 50
 Contents - Channel control / ON-OFF / Volume (R/W)
 Bit 7 - Vin->SO2 ON/OFF
 Bit 6-4 - SO2 output level (volume) (# 0-7)
 Bit 3 - Vin->SO1 ON/OFF
 Bit 2-0 - SO1 output level (volume) (# 0-7)
 Vin->SO1 (Vin->SO2)
 By synthesizing the sound from sound 1
 through 4, the voice input from Vin
 terminal is put out.
 0: no output
 1: output OK

12. FF13 (NR 13)
 Name - NR 13
 Contents - Sound Mode 1 register, Frequency lo (W)
 Lower 8 bits of 11 bit frequency (x).
 Next 3 bit are in NR 14 ($FF14)

13. FF14 (NR 14)
 Name - NR 14
 Contents - Sound Mode 1 register, Frequency hi (R/W)
 Only Bit 6 can be read.
 Bit 7 - Initial (when set, sound
 restarts)
 Bit 6 - Counter/consecutive selection
 Bit 2-0 - Frequency's higher 3 bits (x)
 Frequency = 4194304/(32*(2048-x)) Hz
 = 131072/(2048-x) Hz
 Counter/consecutive Selection
 0 = Regardless of the length data in
 NR11 sound can be produced
 consecutively.
 1 = Sound is generated during the time
 period set by the length data in
 NR11. After this period the sound 1
 ON flag (bit 0 of NR52) is reset.



*/

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct SoundSubsystem {
    NR11: u8,
    NR12: u8,
    NR13: u8,
    NR14: u8,

    NR50: u8,
    NR51: u8,
    NR52: u8,
}

impl SoundSubsystem {
    pub fn new() -> Self {
        SoundSubsystem {
            NR11: 0,
            NR12: 0,
            NR13: 0,
            NR14: 0,

            NR50: 0,
            NR51: 0,
            NR52: 0,
        }
    }

    pub fn write(&mut self, address: u16, value: u8) -> bool {
        match address {
            0xFF11 => {
                self.NR11 = value;
            }
            0xFF12 => {
                self.NR12 = value;
            }
            0xFF13 => {
                self.NR13 = value;
            }
            0xFF14 => {
                self.NR14 = value;
            }
            0xFF24 => {
                self.NR50 = value;
            }
            0xFF25 => {
                self.NR51 = value;
            }
            0xFF26 => {
                self.NR52 = value;
            }
            _ => return false,
        }
        return true;
    }

    pub fn read(&self, address: u16) -> Option<u8> {
        match address {
            0xFF11 => Some(self.NR11),
            0xFF12 => Some(self.NR12),
            0xFF13 => Some(self.NR13),
            0xFF14 => Some(self.NR14),

            0xFF24 => Some(self.NR50),
            0xFF25 => Some(self.NR51),
            0xFF26 => Some(self.NR52),
            _ => None,
        }
    }
}
