#![feature(nll)]
#![allow(unused)]
#![feature(exclusive_range_pattern)]

extern crate core;
extern crate minifb;

use minifb::{Key, Window, WindowOptions};
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

mod cpu;
mod instruction;
mod interconnect;
mod memory_map;
mod ppu;
mod sound_subsystem;
mod timer;

const WIDTH: usize = 256;
const HEIGHT: usize = 256;

fn main() -> io::Result<()> {
    let boot = read_file("resources/boot/DMG_ROM.bin")?;
    //let rom = read_file("resources/roms/Tetris-USA.gb")?;
    let rom = read_file("resources/roms/cpu_instrs/individual/06-ld r,r.gb")?;
    let ic = interconnect::Interconnect::new(boot, rom);
    let mut cpu = cpu::Cpu::new(ic);

    while cpu.interconnect.ppu.main_window.is_open()
        && !cpu.interconnect.ppu.main_window.is_key_down(Key::Escape)
    {
        cpu.step();
    }

    Ok(())
}

fn read_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    let mut f = File::open(path)?;
    let mut buf_reader = BufReader::new(f);
    let mut rom = vec![];
    buf_reader.read_to_end(&mut rom)?;
    Ok(rom)
}
