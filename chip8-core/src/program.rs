use crate::instructions::Instruction;

use std::convert::TryFrom;
use std::path::PathBuf;
use std::fs::File;
use std::io::{ self, Read };

pub struct Program {
    memory: [u8; 4096],
    registers: [u8; 16],
    i: u16,
    delay_timer: u8,
    sound_timer: u8,
    program_counter: u16,
    stack_pointer: u8,
    keyboard: [bool; 16],
    screen: [[bool; 64]; 32],
    stack: [u16; 16],
}

impl Program {
    fn instruction(&self) -> Instruction {
        let code = (self.memory[self.program_counter as usize] as u16) << 8 | (self.memory[self.program_counter as usize + 1] as u16);

        Instruction::from(code)
    }
}

impl TryFrom<PathBuf> for Program {
    type Error = io::Error;

    fn try_from(path: PathBuf) -> io::Result<Self> {
        let file = File::open(path)?;
        let mut memory = [0; 4096];

        for (index, byte) in file.bytes().skip(0x200).enumerate() {
            memory[0x200 + index] = byte?;
        }

        Ok(Program {
            memory,
            registers: [0; 16],
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
            program_counter: 0x200,
            stack_pointer: 0,
            keyboard: [false; 16],
            screen: [[false; 64]; 32],
            stack: [0; 16]
        })
    }
}

use std::fmt;

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        for line in &self.screen {
            let line: String = (&line).iter().map(|&x| if x { 'x' } else { ' ' }).collect();

            write!(f, "{}\n", line)?;
        }

        Ok(())
    }
}
