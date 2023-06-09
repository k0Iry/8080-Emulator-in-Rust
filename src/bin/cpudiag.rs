use std::{
    fs::File,
    io::{BufReader, Read},
};

use emulator::{Cpu8080, InvalidFile, Result};

fn main() -> Result<()> {
    let cpudiag_prog = std::env::current_dir()?.join("roms/cpudiag");

    println!(
        "executing {:?}....",
        cpudiag_prog.file_name().ok_or(InvalidFile)?
    );
    let bytes = BufReader::new(File::open(cpudiag_prog)?).bytes();
    let rom = bytes.collect::<std::result::Result<Vec<u8>, std::io::Error>>()?;
    let mut cpu = Cpu8080::new(&rom);
    cpu.run()?;
    Ok(())
}
