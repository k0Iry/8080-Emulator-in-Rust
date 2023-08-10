use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use i8080emulator::{Cpu8080, Result};

fn main() -> Result<()> {
    let cpudiag_prog = Path::new(env!("CARGO_MANIFEST_DIR")).join("diagnosis_program/cpudiag");

    println!("executing CPU diagnosis...");
    let bytes = BufReader::new(File::open(cpudiag_prog)?).bytes();
    let rom = bytes.collect::<std::result::Result<Vec<u8>, std::io::Error>>()?;
    let ram = vec![0; 0x200];
    let mut cpu = Cpu8080::cpudiag_new(rom, ram);
    cpu.run()?;
    Ok(())
}
