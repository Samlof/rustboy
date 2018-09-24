pub const ROM_BANK0_START: u16 = 0x0000;
pub const ROM_BANK0_END: u16 = 0x4000;
pub const ROM_BANK0_LENGTH: u16 = ROM_BANK0_END - ROM_BANK0_START;

pub const SWITCH_ROM_BANK_START: u16 = 0x4000;
pub const SWITCH_ROM_BANK_END: u16 = 0x8000;
pub const SWITCH_ROM_BANK_LENGTH: u16 = SWITCH_ROM_BANK_END - SWITCH_ROM_BANK_START;

pub const VRAM_START: u16 = 0x8000;
pub const VRAM_END: u16 = 0xA000;
pub const VRAM_LENGTH: u16 = VRAM_END - VRAM_START;

pub const SWITCH_RAM_BANK_START: u16 = 0xA000;
pub const SWITCH_RAM_BANK_END: u16 = 0xC000;
pub const SWITCH_RAM_BANK_LENGTH: u16 = SWITCH_RAM_BANK_END - SWITCH_RAM_BANK_START;

pub const INTERNAL_RAM_START: u16 = 0xC000;
pub const INTERNAL_RAM_END: u16 = 0xE000;
pub const INTERNAL_RAM_LENGTH: u16 = INTERNAL_RAM_END - INTERNAL_RAM_START;

/*
The addresses E000-FE00 appear to access the internal
RAM the same as C000-DE00. (i.e. If you write a byte to
address E000 it will appear at C000 and E000.
Similarly, writing a byte to C000 will appear at C000
and E000.)

*/
pub const ECHO_RAM_START: u16 = 0xE000;
pub const ECHO_RAM_END: u16 = 0xFE00;
pub const ECHO_RAM_LENGTH: u16 = ECHO_RAM_END - ECHO_RAM_START;

pub const SPRITE_MEM_START: u16 = 0xFE00;
pub const SPRITE_MEM_END: u16 = 0xFEA0;
pub const SPRITE_MEM_LENGTH: u16 = SPRITE_MEM_END - SPRITE_MEM_START;

pub const IO_PORTS_START: u16 = 0xFF00;
pub const IO_PORTS_END: u16 = 0xFF4C;
pub const IO_PORTS_LENGTH: u16 = IO_PORTS_END - IO_PORTS_START;

pub const INTERNAL_RAM2_START: u16 = 0xFF80;
pub const INTERNAL_RAM2_END: u16 = 0xFFFF;
pub const INTERNAL_RAM2_LENGTH: u16 = INTERNAL_RAM2_END - INTERNAL_RAM2_START;

pub const INTERRUPT_REGISTER: u16 = 0xFFFF;

/*
2.5.4. Reserved Memory Locations
0000 Restart $00 Address
 (RST $00 calls this address.)
0008 Restart $08 Address
 (RST $08 calls this address.)
0010 Restart $10 Address
 (RST $10 calls this address.)
0018 Restart $18 Address
 (RST $18 calls this address.)
0020 Restart $20 Address
 (RST $20 calls this address.)
0028 Restart $28 Address
 (RST $28 calls this address.)
0030 Restart $30 Address
 (RST $30 calls this address.)
0038 Restart $38 Address
 (RST $38 calls this address.)
0040 Vertical Blank Interrupt Start Address
0048 LCDC Status Interrupt Start Address
0050 Timer Overflow Interrupt Start Address
0058 Serial Transfer Completion Interrupt
 Start Address
0060 High-to-Low of P10-P13 Interrupt
 Start Address

*/

/*
An internal information area is located at 0100-014F in
each cartridge. It contains the following values:

0100-0103 This is the begin code execution point in a
 cart. Usually there is a NOP and a JP
 instruction here but not always.

0104-0133 Scrolling Nintendo graphic:
 CE ED 66 66 CC 0D 00 0B 03 73 00 83 00 0C 00 0D
 00 08 11 1F 88 89 00 0E DC CC 6E E6 DD DD D9 99
 BB BB 67 63 6E 0E EC CC DD DC 99 9F BB B9 33 3E
 ( PROGRAM WON'T RUN IF CHANGED!!!)

0134-0142 Title of the game in UPPER CASE ASCII. If it
 is less than 16 characters then the
 remaining bytes are filled with 00's.

0143 $80 = Color GB, $00 or other = not Color GB

0144 Ascii hex digit, high nibble of licensee
 code (new).

0145 Ascii hex digit, low nibble of licensee
 code (new). (These are normally $00 if
 [$014B] <> $33.)

0146 GB/SGB Indicator (00 = GameBoy, 03 = Super
 GameBoy functions)
 (Super GameBoy functions won't work
 if <> $03.)

0147 Cartridge type:
0-ROM ONLY              12-ROM+MBC3+RAM
1-ROM+MBC1              13-ROM+MBC3+RAM+BATT
2-ROM+MBC1+RAM          19-ROM+MBC5
3-ROM+MBC1+RAM+BATT     1A-ROM+MBC5+RAM
5-ROM+MBC2              1B-ROM+MBC5+RAM+BATT
6-ROM+MBC2+BATTERY      1C-ROM+MBC5+RUMBLE
8-ROM+RAM               1D-ROM+MBC5+RUMBLE+SRAM
9-ROM+RAM+BATTERY       1E-ROM+MBC5+RUMBLE+SRAM+BATT
B-ROM+MMM01             1F-Pocket Camera
C-ROM+MMM01+SRAM        FD-Bandai TAMA5
D-ROM+MMM01+SRAM+BATT   FE - Hudson HuC-3
F-ROM+MBC3+TIMER+BATT   FF - Hudson HuC-1
10-ROM+MBC3+TIMER+RAM+BATT
11-ROM+MBC3

0148 ROM size:
 0 - 256Kbit = 32KByte = 2 banks
 1 - 512Kbit = 64KByte = 4 banks
 2 - 1Mbit = 128KByte = 8 banks
 3 - 2Mbit = 256KByte = 16 banks
 4 - 4Mbit = 512KByte = 32 banks
 5 - 8Mbit = 1MByte = 64 banks
 6 - 16Mbit = 2MByte = 128 banks
 $52 - 9Mbit = 1.1MByte = 72 banks
 $53 - 10Mbit = 1.2MByte = 80 banks
 $54 - 12Mbit = 1.5MByte = 96 banks

0149 RAM size:
 0 - None
 1 - 16kBit = 2kB = 1 bank
 2 - 64kBit = 8kB = 1 bank
 3 - 256kBit = 32kB = 4 banks
 4 - 1MBit =128kB =16 banks

014A Destination code:
 0 - Japanese
 1 - Non-Japanese

014B Licensee code (old):
 33 - Check 0144/0145 for Licensee code.
 79 - Accolade
 A4 - Konami
 (Super GameBoy function won't work
 if <> $33.)

014C Mask ROM Version number (Usually $00)

014D Complement check
 (PROGRAM WON'T RUN ON GB IF NOT CORRECT!!!)
 (It will run on Super GB, however,
 if incorrect.)

014E-014F Checksum (higher byte first) produced by
 adding all bytes of a cartridge except for
 two checksum bytes and taking two lower
 bytes of the result. (GameBoy ignores this
 value.)

 */
