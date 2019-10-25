use crate::program::{ Cursor, Program };

use rand::Rng;

fn address(x: u8, y: u8, n: u8) -> u16 {
    (((x as u16) << 8) & 0xF00) | value(y, n) as u16
}

fn value(y: u8, n: u8) -> u8 {
    ((y << 4) & 0xF0)| (n & 0xF)
}

macro_rules! instructions {
    (
        $(
            $(#[$meta:meta])*
            $bytes:pat => $instruction:ident $({
                $($field:ident: $type:ty = $expression:expr),*
            })*,

            fn run(&$fn_arg0:ident, $fn_arg1:ident: &mut Program) -> Cursor $fn_body:block
        ),*
    ) => {
        $(
            $(#[$meta])*
            pub struct $instruction {
                $($($field: $type),*)*
            }

            impl $instruction {
                pub fn run(&$fn_arg0, $fn_arg1: &mut Program) -> Cursor {
                    $fn_body
                }
            }
        )*

        /// Enum containing all chip8 instructions.
        ///
        /// Description of each instruction retrieved from the
        /// [Cowngod's Chip-8 Technical Reference v1.0](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
        pub enum Instruction {
            $(
                $(#[$meta])*
                $instruction($instruction)
            ),*
        }

        impl Instruction {
            pub fn run(&self, program: &mut Program) -> Cursor {
                match self {
                    $(
                        Instruction::$instruction(instruction) => instruction.run(program)
                    ),*
                }
            }
        }

        impl From<u16> for Instruction {
            fn from(code: u16) -> Self {
                let nibbles = (
                    ((code & 0xF000) >> 12) as u8,
                    ((code & 0x0F00) >> 8) as u8,
                    ((code & 0x00F0) >> 4) as u8,
                    (code & 0x000F) as u8
                );

                match nibbles {
                    $(
                        $bytes => Instruction::$instruction(
                            $instruction {
                                $($($field: $expression),*)*
                            }
                        )
                    ),*
                }
            }
        }
    };
}

instructions! {
    /// Clear the display.
    (0x0, 0x0, 0xE, 0x0) => Clear,
    fn run(&self, program: &mut Program) -> Cursor {
        program.screen = [[false; 64]; 32];

        Cursor::Next
    },

    /// Return from a subroutine.
    ///
    /// The interpreter sets the program counter to the address at the top of the stack, then
    /// subtracts 1 from the stack pointer.
    (0x0, 0x0, 0xE, 0xE) => ReturnSubroutine,
    fn run(&self, program: &mut Program) -> Cursor {
        program.stack_pointer -= 1;

        Cursor::Jump(program.stack[program.stack_pointer as usize])
    },

    /// Jump to location `address`.
    ///
    /// The interpreter sets the program counter to `address`.
    (0x1, x, y, n) => JumpTo {
        address: u16 = address(x, y, n)
    },
    fn run(&self, _program: &mut Program) -> Cursor {
        Cursor::Jump(self.address)
    },

    /// Call subroutine at `address`.
    ///
    /// The interpreter increments the stack pointer, then puts the current PC on the top of the
    /// stack. The PC is then set to `address`.
    (0x2, x, y, n) => CallSubroutine {
        address: u16 = address(x, y, n)
    },
    fn run(&self, program: &mut Program) -> Cursor {
        program.stack[program.stack_pointer as usize] = program.program_counter + 2;
        program.stack_pointer += 1;

        Cursor::Jump(self.address)
    },

    /// Skip next instruction if Vx = kk.
    ///
    /// The interpreter compares register Vx to kk, and if they are equal, increments the program
    /// counter by 2.
    (0x3, x, y, n) => SkipEqual {
        x: usize = x as usize,
        value: u8 = value(y, n)
    },
    fn run(&self, program: &mut Program) -> Cursor {
        if program.v[self.x] == self.value {
            Cursor::Skip
        } else {
            Cursor::Next
        }
    },

    /// Skip next instruction if Vx != kk.
    ///
    /// The interpreter compares register Vx to kk, and if they are not equal, increments the
    /// program counter by 2.
    (0x4, x, y, n) => SkipNotEqual {
        x: usize = x as usize,
        value: u8 = value(y, n)
    },
    fn run(&self, program: &mut Program) -> Cursor {
        if program.v[self.x] != self.value {
            Cursor::Skip
        } else {
            Cursor::Next
        }
    },

    /// Skip next instruction if Vx = Vy.
    ///
    /// The interpreter compares register Vx to register Vy, and if they are equal, increments the
    /// program counter by 2.
    (0x5, x, y, 0x0) => SkipRegisterEqual {
        x: usize = x as usize,
        y: usize = y as usize
    },
    fn run(&self, program: &mut Program) -> Cursor {
        if program.v[self.x] == program.v[self.y] {
            Cursor::Skip
        } else {
            Cursor::Next
        }
    },

    /// Set Vx = kk.
    ///
    /// The interpreter puts the value kk into register Vx.
    (0x6, x, y, n) => SetRegister {
        x: usize = x as usize,
        value: u8 = value(y, n)
    },
    fn run(&self, program: &mut Program) -> Cursor {
        program.v[self.x] = self.value;

        Cursor::Next
    },

    /// Set Vx = Vx + kk.
    ///
    /// Adds the value kk to the value of register Vx, then stores the result in Vx.
    (0x7, x, y, n) => AddRegister {
        x: usize = x as usize,
        value: u8 = value(y, n)
    },
    fn run(&self, program: &mut Program) -> Cursor {
        let idx = self.x;
        program.v[idx] = (program.v[idx] as u16 + self.value as u16) as u8;

        Cursor::Next
    },

    /// Set Vx = Vy.
    ///
    /// Stores the value of register Vy in register Vx.
    (0x8, x, y, 0x0) => SetVxToVy {
        x: usize = x as usize,
        y: usize = y as usize
    },
    fn run(&self, program: &mut Program) -> Cursor {
        program.v[self.x] = program.v[self.y];

        Cursor::Next
    },

    /// Set Vx = Vx OR Vy.
    ///
    /// Performs a bitwise OR on the values of Vx and Vy, then stores the result in Vx. A bitwise
    /// OR compares the corrseponding bits from two values, and if either bit is 1, then the same
    /// bit in the result is also 1. Otherwise, it is 0.
    (0x8, x, y, 0x1) => SetVxToVxOrVy {
        x: usize = x as usize,
        y: usize = y as usize
    },
    fn run(&self, program: &mut Program) -> Cursor {
        program.v[self.x] |= program.v[self.y];

        Cursor::Next
    },

    /// Set Vx = Vx AND Vy.
    ///
    /// Performs a bitwise AND on the values of Vx and Vy, then stores the result in Vx. A bitwise
    /// AND compares the corrseponding bits from two values, and if both bits are 1, then the same
    /// bit in the result is also 1. Otherwise, it is 0.
    (0x8, x, y, 0x2) => SetVxToVxAndVy {
        x: usize = x as usize,
        y: usize = y as usize
    },
    fn run(&self, program: &mut Program) -> Cursor {
        program.v[self.x] &= program.v[self.y];

        Cursor::Next
    },

    /// Set Vx = Vx XOR Vy.
    ///
    /// Performs a bitwise exclusive OR on the values of Vx and Vy, then stores the result in Vx.
    /// An exclusive OR compares the corrseponding bits from two values, and if the bits are not
    /// both the same, then the corresponding bit in the result is set to 1. Otherwise, it is 0.
    (0x8, x, y, 0x3) => SetVxToVxXorVy {
        x: usize = x as usize,
        y: usize = y as usize
    },
    fn run(&self, program: &mut Program) -> Cursor {
        program.v[self.x] ^= program.v[self.y];

        Cursor::Next
    },

    /// Set Vx = Vx + Vy, set VF = carry.
    ///
    /// The values of Vx and Vy are added together. If the result is greater than 8 bits
    /// (i.e., > 255,) VF is set to 1, otherwise 0. Only the lowest 8 bits of the result are
    /// kept, and stored in Vx.
    (0x8, x, y, 0x4) => SetVxToVxAndVyCarry {
        x: usize = x as usize,
        y: usize = y as usize
    },
    fn run(&self, program: &mut Program) -> Cursor {
        let result = program.v[self.x] as u16 + program.v[self.y] as u16;

        program.v[self.x] = result as u8;
        program.v[0xF] = if result > 0xFF { 1 } else { 0 };

        Cursor::Next
    },

    /// Set Vx = Vx - Vy, set VF = NOT borrow.
    ///
    /// If Vx > Vy, then VF is set to 1, otherwise 0. Then Vy is subtracted from Vx, and the
    /// results stored in Vx.
    (0x8, x, y, 0x5) => SetVxToVxSubVy {
        x: usize = x as usize,
        y: usize = y as usize
    },
    fn run(&self, program: &mut Program) -> Cursor {
        program.v[0xF] = if program.v[self.x] > program.v[self.y] { 1 } else { 0 };
        program.v[self.x] = program.v[self.x].wrapping_sub(program.v[self.y]);

        Cursor::Next
    },

    /// Set Vx = Vx SHR 1.
    ///
    /// If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is
    /// divided by 2.
    (0x8, x, _, 0x6) => SetVxToVxShr {
        x: usize = x as usize
    },
    fn run(&self, program: &mut Program) -> Cursor {
        program.v[0xF] = program.v[self.x] & 1;
        program.v[self.x] >>= 1;

        Cursor::Next
    },

    /// Set Vx = Vy - Vx, set VF = NOT borrow.
    ///
    /// If Vy > Vx, then VF is set to 1, otherwise 0. Then Vx is subtracted from Vy, and the
    /// results stored in Vx.
    (0x8, x, y, 0x7) => SetVxToVySubVx {
        x: usize = x as usize,
        y: usize = y as usize
    },
    fn run(&self, program: &mut Program) -> Cursor {
        program.v[0xF] = if program.v[self.y] > program.v[self.x] { 1 } else { 0 };
        program.v[self.x] = program.v[self.y].wrapping_sub(program.v[self.x]);

        Cursor::Next
    },

    /// Set Vx = Vx SHL 1.
    ///
    /// If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0.
    /// Then Vx is multiplied by 2.
    (0x8, x, _, 0xE) => SetVxToVxShl {
        x: usize = x as usize
    },
    fn run(&self, program: &mut Program) -> Cursor {
        program.v[0xF] = (program.v[self.x] >> 7) & 1;
        program.v[self.x] <<= 1;

        Cursor::Next
    },

    /// Skip next instruction if Vx != Vy.
    ///
    /// The values of Vx and Vy are compared, and if they are not equal, the program counter is
    /// increased by 2.
    (0x9, x, y, 0x0) => SkipRegisterNotEqual {
        x: usize = x as usize,
        y: usize = y as usize
    },
    fn run(&self, program: &mut Program) -> Cursor {
        if program.v[self.x] != program.v[self.y] {
            Cursor::Skip
        } else {
            Cursor::Next
        }
    },

    /// Set I = `address`.
    ///
    /// The value of register I is set to `address`.
    (0xA, x, y, n) => SetIToAddress {
        address: u16 = address(x, y, n)
    },
    fn run(&self, program: &mut Program) -> Cursor {
        program.i = self.address;

        Cursor::Next
    },

    /// Jump to location `address` + V0.
    ///
    /// The program counter is set to `address` plus the value of V0.
    (0xB, x, y, n) => JumpToPlusV0 {
        address: u16 = address(x, y, n)
    },
    fn run(&self, program: &mut Program) -> Cursor {
        Cursor::Jump(program.v[0] as u16 + self.address)
    },

    /// Set Vx = random byte AND kk.
    ///
    /// The interpreter generates a random number from 0 to 255, which is then ANDed with the
    /// value kk. The results are stored in Vx.
    (0xC, x, y, n) => SetVxToRandomAndValue {
        x: usize = x as usize,
        value: u8 = value(y, n)
    },
    fn run(&self, program: &mut Program) -> Cursor {
        program.v[self.x] = program.rng.gen::<u8>() & self.value;

        Cursor::Next
    },

    /// Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    ///
    /// The interpreter reads n bytes from memory, starting at the address stored in I. These bytes
    /// are then displayed as sprites on screen at coordinates (Vx, Vy). Sprites are XORed onto the
    /// existing screen. If this causes any pixels to be erased, VF is set to 1, otherwise it is
    /// set to 0. If the sprite is positioned so part of it is outside the coordinates of the
    /// display, it wraps around to the opposite side of the screen.
    (0xD, x, y, n) => Draw {
        x: usize = x as usize,
        y: usize = y as usize,
        n: usize = n as usize
    },
    fn run(&self, program: &mut Program) -> Cursor {
        program.v[0xF] = 0;

        for byte in 0..self.n {
            let y = (program.v[self.y] as usize + byte) % 32;
            let byte = program.memory[program.i as usize + byte];

            for bit in 0..8 {
                let x = (program.v[self.x] as usize + bit) % 64;
                let bit = ((byte >> (7 - bit)) & 1) == 1;

                program.v[0xF] |= if bit & program.screen[y][x] { 1 } else { 0 };
                program.screen[y][x] ^= bit;
            }
        }

        Cursor::Next
    },

    /// Skip next instruction if key with the value of Vx is pressed.
    ///
    /// Checks the keyboard, and if the key corresponding to the value of Vx is currently in the
    /// down position, PC is increased by 2.
    (0xE, x, 0x9, 0xE) => SkipKeyPressed {
        x: usize = x as usize
    },
    fn run(&self, program: &mut Program) -> Cursor {
        if program.keypad[program.v[self.x] as usize] {
            Cursor::Skip
        } else {
            Cursor::Next
        }
    },

    /// Skip next instruction if key with the value of Vx is not pressed.
    ///
    /// Checks the keyboard, and if the key corresponding to the value of Vx is currently in the
    /// up position, PC is increased by 2.
    (0xE, x, 0xA, 0x1) => SkipKeyNotPressed {
        x: usize = x as usize
    },
    fn run(&self, program: &mut Program) -> Cursor {
        if !program.keypad[program.v[self.x] as usize] {
            Cursor::Skip
        } else {
            Cursor::Next
        }
    },

    /// Set Vx = delay timer value.
    ///
    /// The value of DT is placed into Vx.
    (0xF, x, 0x0, 0x7) => SetVxToDelayTimer {
        x: usize = x as usize
    },
    fn run(&self, program: &mut Program) -> Cursor {
        program.v[self.x] = program.delay_timer;
        Cursor::Next
    },

    /// Wait for a key press, store the value of the key in Vx.
    ///
    /// All execution stops until a key is pressed, then the value of that key is stored in Vx.
    (0xF, x, 0x0, 0x4) => SetVxToNextKeyPress {
        x: usize = x as usize
    },
    fn run(&self, program: &mut Program) -> Cursor {
        if let Some((i, _)) = program.keypad.iter().enumerate().find(|&(_, &value)| value) {
            program.v[self.x] = i as u8;
            Cursor::Next
        } else {
            Cursor::Stay
        }
    },

    /// Set delay timer = Vx.
    (0xF, x, 0x1, 0x5) => SetDelayTimerToVx {
        x: usize = x as usize
    },
    fn run(&self, program: &mut Program) -> Cursor {
        program.delay_timer = program.v[self.x];
        Cursor::Next
    },

    /// Set sound timer = Vx.
    (0xF, x, 0x1, 0x8) => SetSoundTimerToVx {
        x: usize = x as usize
    },
    fn run(&self, program: &mut Program) -> Cursor {
        program.sound_timer = program.v[self.x];
        Cursor::Next
    },

    /// Set I = I + Vx.
    (0xF, x, 0x1, 0xE) => AddVxToI {
        x: usize = x as usize
    },
    fn run(&self, program: &mut Program) -> Cursor {
        program.i += program.v[self.x] as u16;
        Cursor::Next
    },

    /// Set I = location of sprite for digit Vx.
    ///
    /// The value of I is set to the location for the hexadecimal sprite corresponding to the
    /// value of Vx.
    (0xF, x, 0x2, 0x9) => SetIToSpriteLocation {
        x: usize = x as usize
    },
    fn run(&self, program: &mut Program) -> Cursor {
        program.i = program.v[self.x] as u16 * 5;
        Cursor::Next
    },

    /// Store BCD representation of Vx in memory locations I, I+1, and I+2.
    ///
    /// The interpreter takes the decimal value of Vx, and places the hundreds digit in memory at
    /// location in I, the tens digit at location I+1, and the ones digit at location I+2.
    (0xF, x, 0x3, 0x3) => StoreBCD {
        x: usize = x as usize
    },
    fn run(&self, program: &mut Program) -> Cursor {
        let idx = program.i as usize;
        let value = program.v[self.x];

        program.memory[idx] = value / 100;
        program.memory[idx + 1] = (value % 100) / 10;
        program.memory[idx + 2] = value % 10;
        Cursor::Next
    },

    /// Store registers V0 through Vx in memory starting at location I.
    ///
    /// The interpreter copies the values of registers V0 through Vx into memory,
    /// starting at the address in I.
    (0xF, x, 0x5, 0x5) => StoreRegisters {
        x: usize = x as usize
    },
    fn run(&self, program: &mut Program) -> Cursor {
        // TODO: Use copy from slice
        for i in 0..=self.x {
            program.memory[program.i as usize + i] = program.v[i];
        }

        Cursor::Next
    },

    /// Read registers V0 through Vx from memory starting at location I.
    ///
    /// The interpreter reads values from memory starting at location I into registers V0 through Vx.
    /// Store BCD representation of Vx in memory locations I, I+1, and I+2.
    (0xF, x, 0x6, 0x5) => ReadRegisters {
        x: usize = x as usize
    },
    fn run(&self, program: &mut Program) -> Cursor {
        // TODO: Use copy from slice
        for i in 0..=self.x {
            program.v[i] = program.memory[program.i as usize + i];
        }

        Cursor::Next
    },

    (a, b, c, d) => InvalidInstruction {
        a: u8 = a,
        b: u8 = b,
        c: u8 = c,
        d: u8 = d
    },
    fn run(&self, _program: &mut Program) -> Cursor {
        panic!("invalid instruction ({:x}, {:x}, {:x}, {:x})", self.a, self.b, self.c, self.d);
        unreachable!()
    }
}
