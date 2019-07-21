use bitvec::prelude::*;

pub struct Address(BitVec<BigEndian, u16>);

impl From<u16> for Address {
    fn from(number: u16) -> Self {
        Address(BitVec::from_element(number & 0xFFF))
    }
}

pub struct U4(BitVec<BigEndian, u8>);

impl U4 {
    pub fn read(code: u16, target: u16) -> Self {
        let shift = match target {
            0x000F => 0,
            0x00F0 => 4,
            0x0F00 => 8,
            0xF000 => 12,
            _ => panic!("Invalid target")
        };

        (((code & target) >> shift) as u8).into()
    }
}

impl From<u8> for U4 {
    fn from(number: u8) -> Self {
        U4(BitVec::from_element(number & 0xF))
    }
}

macro_rules! instructions {
    (
        $(
            $(#[$meta:meta])*
            $bytes:pat $(if $check:expr)* => $instruction:ident $({
                $($field:ident: $type:ty = $expression:expr),*
            })*
        ),*
    ) => {
        /// Enum containing all chip8 instructions.
        ///
        /// Description of each instruction retrieved from the
        /// [Cowngod's Chip-8 Technical Reference v1.0](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
        pub enum Instruction {
            $(
                $(#[$meta])*
                $instruction $({
                    $($field: $type),*
                })*
            ),*
        }

        impl From<u16> for Instruction {
            fn from(number: u16) -> Self {
                match number {
                    $(
                        $bytes $(if $check)* => Instruction::$instruction $({
                            $($field: $expression),*
                        })*
                    ),*
                }
            }
        }
    };
}

instructions! {
    /// Clear the display.
    0x00E0 => Clear,

    /// Return from a subroutine.
    ///
    /// The interpreter sets the program counter to the address at the top of the stack, then
    /// subtracts 1 from the stack pointer.
    0x00EE => ReturnSubroutine,

    /// Jump to a machine code routine at `address`.
    ///
    /// This instruction is only used on the old computers on which Chip-8 was originally
    /// implemented. It is ignored by modern interpreters.
    code @ 0x0000 ..= 0x0FFF => CallProgram {
        address: Address = code.into()
    },

    /// Jump to location `address`.
    ///
    /// The interpreter sets the program counter to `address`.
    code @ 0x1000 ..= 0x1FFF => JumpTo {
        address: Address = code.into()
    },

    /// Call subroutine at `address`.
    ///
    /// The interpreter increments the stack pointer, then puts the current PC on the top of the
    /// stack. The PC is then set to `address`.
    code @ 0x2000 ..= 0x2FFF => CallSubroutine {
        address: Address = code.into()
    },

    /// Skip next instruction if Vx = kk.
    ///
    /// The interpreter compares register Vx to kk, and if they are equal, increments the program
    /// counter by 2.
    code @ 0x3000 ..= 0x3FFF => SkipEqual {
        x: U4 = U4::read(code, 0x0F00),
        value: u8 = (code & 0x00FF) as u8
    },

    /// Skip next instruction if Vx != kk.
    ///
    /// The interpreter compares register Vx to kk, and if they are not equal, increments the
    /// program counter by 2.
    code @ 0x4000 ..= 0x4FFF => SkipNotEqual {
        x: U4 = U4::read(code, 0x0F00),
        value: u8 = (code & 0x00FF) as u8
    },

    /// Skip next instruction if Vx = Vy.
    ///
    /// The interpreter compares register Vx to register Vy, and if they are equal, increments the
    /// program counter by 2.
    code @ 0x5000 ..= 0x5FF0 if (code & 0xF) == 0 => SkipRegisterEqual {
        x: U4 = U4::read(code, 0x0F00),
        y: U4 = U4::read(code, 0x00F0)
    },

    /// Set Vx = kk.
    ///
    /// The interpreter puts the value kk into register Vx.
    code @ 0x6000 ..= 0x6FFF => SetRegister {
        x: U4 = U4::read(code, 0x0F00),
        value: u8 = (code & 0x00FF) as u8
    },

    /// Set Vx = Vx + kk.
    ///
    /// Adds the value kk to the value of register Vx, then stores the result in Vx.
    code @ 0x7000 ..= 0x7FFF => AddRegister {
        x: U4 = U4::read(code, 0x0F00),
        value: u8 = (code & 0x00FF) as u8
    },

    /// Set Vx = Vy.
    ///
    /// Stores the value of register Vy in register Vx.
    code @ 0x8000 ..= 0x8FF0 if (code & 0xF) == 0 => SetVxToVy {
        x: U4 = U4::read(code, 0x0F00),
        y: U4 = U4::read(code, 0x00F0)
    },

    /// Set Vx = Vx OR Vy.
    ///
    /// Performs a bitwise OR on the values of Vx and Vy, then stores the result in Vx. A bitwise
    /// OR compares the corrseponding bits from two values, and if either bit is 1, then the same
    /// bit in the result is also 1. Otherwise, it is 0.
    code @ 0x8001 ..= 0x8FF1 if (code & 0xF) == 0x1 => SetVxToVxOrVy {
        x: U4 = U4::read(code, 0x0F00),
        y: U4 = U4::read(code, 0x00F0)
    },

    /// Set Vx = Vx AND Vy.
    ///
    /// Performs a bitwise AND on the values of Vx and Vy, then stores the result in Vx. A bitwise
    /// AND compares the corrseponding bits from two values, and if both bits are 1, then the same
    /// bit in the result is also 1. Otherwise, it is 0.
    code @ 0x8002 ..= 0x8FF2 if (code & 0xF) == 0x2 => SetVxToVxAndVy {
        x: U4 = U4::read(code, 0x0F00),
        y: U4 = U4::read(code, 0x00F0)
    },

    /// Set Vx = Vx XOR Vy.
    ///
    /// Performs a bitwise exclusive OR on the values of Vx and Vy, then stores the result in Vx.
    /// An exclusive OR compares the corrseponding bits from two values, and if the bits are not
    /// both the same, then the corresponding bit in the result is set to 1. Otherwise, it is 0.
    code @ 0x8003 ..= 0x8FF3 if (code & 0xF) == 0x3 => SetVxToVxXorVy {
        x: U4 = U4::read(code, 0x0F00),
        y: U4 = U4::read(code, 0x00F0)
    },

    /// Set Vx = Vx + Vy, set VF = carry.
    ///
    /// The values of Vx and Vy are added together. If the result is greater than 8 bits
    /// (i.e., > 255,) VF is set to 1, otherwise 0. Only the lowest 8 bits of the result are
    /// kept, and stored in Vx.
    code @ 0x8004 ..= 0x8FF4 if (code & 0xF) == 0x4 => SetVxToVxAndVyCarry {
        x: U4 = U4::read(code, 0x0F00),
        y: U4 = U4::read(code, 0x00F0)
    },

    /// Set Vx = Vx - Vy, set VF = NOT borrow.
    ///
    /// If Vx > Vy, then VF is set to 1, otherwise 0. Then Vy is subtracted from Vx, and the
    /// results stored in Vx.
    code @ 0x8005 ..= 0x8FF5 if (code & 0xF) == 0x5 => SetVxToVxSubVy {
        x: U4 = U4::read(code, 0x0F00),
        y: U4 = U4::read(code, 0x00F0)
    },

    /// Set Vx = Vx SHR 1.
    ///
    /// If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is
    /// divided by 2.
    code @ 0x8006 ..= 0x8FF6 if (code & 0xF) == 0x6 => SetVxToVxShr {
        x: U4 = U4::read(code, 0x0F00)
    },

    /// Set Vx = Vy - Vx, set VF = NOT borrow.
    ///
    /// If Vy > Vx, then VF is set to 1, otherwise 0. Then Vx is subtracted from Vy, and the
    /// results stored in Vx.
    code @ 0x8007 ..= 0x8FF7 if (code & 0xF) == 0x7 => SetVxToVySubVx {
        x: U4 = U4::read(code, 0x0F00),
        y: U4 = U4::read(code, 0x00F0)
    },

    /// Set Vx = Vx SHL 1.
    ///
    /// If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0.
    /// Then Vx is multiplied by 2.
    code @ 0x800E ..= 0x8FFE if (code & 0xF) == 0xE => SetVxToVxShl {
        x: U4 = U4::read(code, 0x0F00)
    },

    /// Skip next instruction if Vx != Vy.
    ///
    /// The values of Vx and Vy are compared, and if they are not equal, the program counter is
    /// increased by 2.
    code @ 0x9000 ..= 0x9FF0 if (code & 0xF) == 0 => SkipRegisterNotEqual {
        x: U4 = U4::read(code, 0x0F00),
        y: U4 = U4::read(code, 0x00F0)
    },

    /// Set I = `address`.
    ///
    /// The value of register I is set to `address`.
    code @ 0xA000 ..= 0xAFFF => SetIToAddress {
        address: Address = code.into()
    },

    /// Jump to location `address` + V0.
    ///
    /// The program counter is set to `address` plus the value of V0.
    code @ 0xB000 ..= 0xBFFF => JumpToPlusV0 {
        address: Address = code.into()
    },

    /// Set Vx = random byte AND kk.
    ///
    /// The interpreter generates a random number from 0 to 255, which is then ANDed with the
    /// value kk. The results are stored in Vx.
    code @ 0xC000 ..= 0xCFFF => SetVxToRandomAndValue {
        x: U4 = U4::read(code, 0x0F00),
        value: u8 = (code & 0x00FF) as u8
    },

    /// Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    ///
    /// The interpreter reads n bytes from memory, starting at the address stored in I. These bytes
    /// are then displayed as sprites on screen at coordinates (Vx, Vy). Sprites are XORed onto the
    /// existing screen. If this causes any pixels to be erased, VF is set to 1, otherwise it is
    /// set to 0. If the sprite is positioned so part of it is outside the coordinates of the
    /// display, it wraps around to the opposite side of the screen.
    code @ 0xD000 ..= 0xDFFF => Draw {
        x: U4 = U4::read(code, 0x0F00),
        y: U4 = U4::read(code, 0x00F0),
        n: U4 = U4::read(code, 0x000F)
    },

    /// Skip next instruction if key with the value of Vx is pressed.
    ///
    /// Checks the keyboard, and if the key corresponding to the value of Vx is currently in the
    /// down position, PC is increased by 2.
    code @ 0xE09E ..= 0xEF9E if (code & 0xFF) == 0x9E => SkipKeyPressed {
        x: U4 = U4::read(code, 0x0F00)
    },

    /// Skip next instruction if key with the value of Vx is not pressed.
    ///
    /// Checks the keyboard, and if the key corresponding to the value of Vx is currently in the
    /// up position, PC is increased by 2.
    code @ 0xE0A1 ..= 0xEFA1 if (code & 0xFF) == 0xA1 => SkipKeyNotPressed {
        x: U4 = U4::read(code, 0x0F00)
    },

    /// Set Vx = delay timer value.
    ///
    /// The value of DT is placed into Vx.
    code @ 0xF007 ..= 0xFF07 if (code & 0xFF) == 0x07 => SetVxToDelayTimer {
        x: U4 = U4::read(code, 0x0F00)
    },

    /// Wait for a key press, store the value of the key in Vx.
    ///
    /// All execution stops until a key is pressed, then the value of that key is stored in Vx.
    code @ 0xF004 ..= 0xFF04 if (code & 0xFF) == 0x04 => SetVxToNextKeyPress {
        x: U4 = U4::read(code, 0x0F00)
    },

    /// Set delay timer = Vx.
    ///
    /// DT is set equal to the value of Vx.
    code @ 0xF015 ..= 0xFF15 if (code & 0xFF) == 0x15 => SetDelayTimerToVx {
        x: U4 = U4::read(code, 0x0F00)
    },

    /// Set sound timer = Vx.
    ///
    /// ST is set equal to the value of Vx.
    code @ 0xF018 ..= 0xFF18 if (code & 0xFF) == 0x18 => SetSoundTimerToVx {
        x: U4 = U4::read(code, 0x0F00)
    },

    /// Set I = I + Vx.
    ///
    /// The values of I and Vx are added, and the results are stored in I.
    code @ 0xF01E ..= 0xFF1E if (code & 0xFF) == 0x1E => AddVxToI {
        x: U4 = U4::read(code, 0x0F00)
    },

    /// Set I = location of sprite for digit Vx.
    ///
    /// The value of I is set to the location for the hexadecimal sprite corresponding to the
    /// value of Vx.
    code @ 0xF029 ..= 0xFF29 if (code & 0xFF) == 0x29 => SetIToSpriteLocation {
        x: U4 = U4::read(code, 0x0F00)
    },

    /// Store BCD representation of Vx in memory locations I, I+1, and I+2.
    ///
    /// The interpreter takes the decimal value of Vx, and places the hundreds digit in memory at
    /// location in I, the tens digit at location I+1, and the ones digit at location I+2.
    code @ 0xF033 ..= 0xFF33 if (code & 0xFF) == 0x33 => StoreBCD {
        x: U4 = U4::read(code, 0x0F00)
    },

    /// Store registers V0 through Vx in memory starting at location I.
    ///
    /// The interpreter copies the values of registers V0 through Vx into memory,
    /// starting at the address in I.
    code @ 0xF055 ..= 0xFF55 if (code & 0xFF) == 0x55 => StoreRegisters {
        x: U4 = U4::read(code, 0x0F00)
    },

    /// Read registers V0 through Vx from memory starting at location I.
    ///
    /// The interpreter reads values from memory starting at location I into registers V0 through Vx.
    /// Store BCD representation of Vx in memory locations I, I+1, and I+2.
    code @ 0xF065 ..= 0xFF65 if (code & 0xFF) == 0x65 => ReadRegisters {
        x: U4 = U4::read(code, 0x0F00)
    },

    _ => InvalidInstruction
}
