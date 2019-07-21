use std::path::{ Path, PathBuf };
use std::convert::TryFrom;
use std::io;

use chip8::program::Program;

fn main() -> io::Result<()> {
    let rom: PathBuf = Path::new("./.roms/PONG").into();
    let program = Program::try_from(rom)?;

    println!("{}", program);

    Ok(())
}
