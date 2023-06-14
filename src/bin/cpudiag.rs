use std::{
    fs::File,
    io::{BufReader, Read},
};

use emulator::{Cpu8080, InvalidFile, Result, SwiftCallbacks};

fn main() -> Result<()> {
    let cpudiag_prog = std::env::current_dir()?.join("roms/cpudiag");

    println!(
        "executing {:?}....",
        cpudiag_prog.file_name().ok_or(InvalidFile)?
    );
    let bytes = BufReader::new(File::open(cpudiag_prog)?).bytes();
    let rom = bytes.collect::<std::result::Result<Vec<u8>, std::io::Error>>()?;
    let mut ram = vec![0; 0x2000];
    pub extern "C" fn input(port: u8) -> u8 {
        port
    }
    pub extern "C" fn output(port: u8, value: u8) {
        println!("{port}, {value}")
    }
    let mut cpu = Cpu8080::new(&rom, &mut ram, SwiftCallbacks { input, output });
    cpu.run()?;
    Ok(())
}
