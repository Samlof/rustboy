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
use std::sync::mpsc::channel;
use std::thread;
use std::thread::sleep;
use std::time::{Duration, Instant};

mod cartridge;
mod console;
mod cpu;
mod instruction;
mod interconnect;
mod joypad;
mod memory_map;
mod ppu;
mod sound_subsystem;
mod timer;
mod utils;

const WIDTH: usize = 256;
const HEIGHT: usize = 256;

const FPS: u64 = 60;
const CLOCKS_PER_FRAME: u64 = 4194304 / FPS;
const MS_PER_FRAME: u64 = ((1 as f32 / FPS as f32) * 1000.0) as u64;

fn main() -> io::Result<()> {
    let boot = read_file("resources/boot/DMG_ROM.bin")?;
    let rom = cartridge::Cartridge::new(read_file("resources/roms/Tetris-USA.gb")?);
    /*
    let rom = cartridge::Cartridge::new(read_file(
        "resources/roms/cpu_instrs/individual/03-op sp,hl.gb",
    )?);
    */
    let ic = interconnect::Interconnect::new(boot, rom);
    let mut cpu = cpu::Cpu::new(ic);

    let (tx, rx) = channel::<console::CpuText>();

    cpu.set_console_tx(tx);
    cpu.set_print_instruction(false);
    let fps_cap = true;

    let mut console = console::Console::new(rx);
    thread::spawn(move || console.start());

    let mut start_time = Instant::now();
    let mut clocks = 0;

    while cpu.interconnect.ppu.main_window.is_open()
        && !cpu.interconnect.ppu.main_window.is_key_down(Key::Escape)
    {
        if fps_cap && clocks > CLOCKS_PER_FRAME {
            let elapsed = start_time.elapsed();
            if let Some(dur) = Duration::from_millis(MS_PER_FRAME).checked_sub(elapsed) {
                thread::sleep(dur);
            }
            start_time = Instant::now();
            clocks = 0;
        }
        cpu.step();
        if fps_cap {
            clocks += 1;
        }
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
