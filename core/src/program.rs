use crate::instructions::Instruction;

use std::convert::TryFrom;
use std::path::PathBuf;
use std::fs::File;
use std::io::{ self, Read };

use rand::rngs::ThreadRng;

pub const SPRITES: [[u8; 5]; 16] = [
    [0xF0, 0x90, 0x90, 0x90, 0xF0],
    [0x20, 0x60, 0x20, 0x20, 0x70],
    [0xF0, 0x10, 0xF0, 0x80, 0xF0],
    [0xF0, 0x10, 0xF0, 0x10, 0xF0],
    [0x90, 0x90, 0xF0, 0x10, 0x10],
    [0xF0, 0x80, 0xF0, 0x10, 0xF0],
    [0xF0, 0x80, 0xF0, 0x90, 0xF0],
    [0xF0, 0x10, 0x20, 0x40, 0x40],
    [0xF0, 0x90, 0xF0, 0x90, 0xF0],
    [0xF0, 0x90, 0xF0, 0x10, 0xF0],
    [0xF0, 0x90, 0xF0, 0x90, 0x90],
    [0xE0, 0x90, 0xE0, 0x90, 0xE0],
    [0xF0, 0x80, 0x80, 0x80, 0xF0],
    [0xE0, 0x90, 0x90, 0x90, 0xE0],
    [0xF0, 0x80, 0xF0, 0x80, 0xF0],
    [0xF0, 0x80, 0xF0, 0x80, 0x80]
];

pub enum Cursor {
    Stay,
    Next,
    Skip,
    Jump(u16)
}

pub struct Program {
    pub memory: [u8; 4096],
    pub(crate) v: [u8; 16],
    pub(crate) i: u16,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub program_counter: u16,
    pub(crate) stack_pointer: u8,
    pub(crate) keypad: [bool; 16],
    pub screen: [[bool; 64]; 32],
    pub(crate) stack: [u16; 16],
    pub(crate) rng: ThreadRng,
}

use std::iter::repeat;

impl Program {
    fn instruction(&self) -> Instruction {
        let counter = self.program_counter as usize;
        let code = &self.memory[counter..=counter+1];
        let code = ((code[0] as u16) << 8) | (code[1] as u16);
        //let code = u16::from_be_bytes(<[u8; 2]>::try_from(code).unwrap());

        Instruction::from(code)
    }

    pub fn run(&mut self) {
        match self.instruction().run(self) {
            Cursor::Stay => {},
            Cursor::Next => self.program_counter += 2,
            Cursor::Skip => self.program_counter += 4,
            Cursor::Jump(address) => self.program_counter = address
        }
    }

    pub fn new() -> Self {
        let mut memory = [0u8; 4096];

        let mut i = 0;
        for sprit in &SPRITES {
            for byte in sprit {
                memory[i] = *byte;
                i += 1;
            }
        }

        Program {
            memory,
            v: [0; 16],
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
            program_counter: 0x200,
            stack_pointer: 0,
            keypad: [false; 16],
            screen: [[false; 64]; 32],
            stack: [0; 16],
            rng: rand::thread_rng()
        }
    }

    pub fn load(&mut self, data: &[u8]) {
        let iter = data.iter().chain(repeat(&0)).enumerate().take(4096 - 0x200);

        for (index, value) in iter {
            self.memory[0x200 + index] = *value;
        }
    }

    pub fn keydown(&mut self, key: usize) {
        // TODO: Check key value
        self.keypad[key] = true;
    }

    pub fn keyup(&mut self, key: usize) {
        // TODO: Check key value
        self.keypad[key] = false;
    }

    // fn run_instruction(&mut self, instruction: Instruction) {
    //     match instruction {
    //         Instruction::Clear => {
    //             for line in self.screen.iter_mut() {
    //                 for column in line.iter_mut() {
    //                     *column = false;
    //                 }
    //             }
    //         }
    //     }
    // }
}
