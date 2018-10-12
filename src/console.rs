use std::io;
use std::io::Write;
use std::sync::mpsc;
use std::thread::sleep;

pub enum CpuText {
    Instruction(String),
    Interrupt(String),
    Registers(String),
}

pub struct Console {
    instr_rx: mpsc::Receiver<CpuText>,
}

impl Console {
    pub fn new(rx: mpsc::Receiver<CpuText>) -> Self {
        Console { instr_rx: rx }
    }

    pub fn start(&mut self) {
        let mut stdout = io::stdout();
        loop {
            let instr = self.instr_rx.recv().unwrap();
            let mut handle = stdout.lock();
            let string = match instr {
                CpuText::Instruction(string) => string,
                _ => String::new(),
            };
            if string.len() > 1 {
                handle.write(string.as_bytes());
                handle.write(b"\n");
            }
        }
    }
}
