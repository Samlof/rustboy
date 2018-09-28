use std::sync::mpsc;

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

    pub fn step(&mut self) {
        let string = self.instr_rx.recv().unwrap();
        match string {
            CpuText::Instruction(str) => println!("{}", str),
            _ => (),
        };
    }
}
