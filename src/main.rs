use std::{
    fs::File,
    io::{BufReader, Read},
};

use emulator::{Cpu8080, Result, RomReadFailure, ROM_SIZE};

/// Brief information of registers and instructions
/// The 8080 provides the programmer with an 8-bit ac- cumulator and 6 additional 8-bit "scratchpad" registers.
/// Which are B, C, D, E, H, L, A
///
/// Some 8080 operations reference the working registers in pairs for storing 16-bit address, e.g. LXI
///
/// Registers assign:
///
/// B assigned to 0 representing register B
///
/// C ........... 1 ..................... C
///
/// D ........... 2 ..................... D
///
/// E ........... 3 ..................... E
///
/// H ........... 4 ..................... H
///
/// L ........... 5 ..................... L
///
/// M ........... 6 ..................... a memory reference
///
/// A ........... 7 ..................... A (accumulator)
///
/// Program Counter is specified as `$`
///
/// operators priorities:
/// 1. Parenthesized expressions
/// 2. *,/M, MOD, SHL, SHR
/// 3. +, - (unary and binary)
/// 4. NOT
/// 5. ADD
/// 6. OR, XOR

fn main() -> Result<()> {
    let files_dir = std::env::current_dir()?.join("invaders");

    let files = std::fs::read_dir(files_dir)?
        .map(|res| res.map(|entry| entry.path()))
        .collect::<std::result::Result<Vec<_>, std::io::Error>>()?;

    for file in files {
        if file.file_name().unwrap() == "invaders" {
            continue;
        }
        println!("executing {:?}....", file.file_name().unwrap());
        let bytes = BufReader::new(File::open(file)?).bytes();
        let rom = bytes.collect::<std::result::Result<Vec<u8>, std::io::Error>>()?;
        let rom: &mut [u8; ROM_SIZE] = &mut rom.try_into().map_err(|_| RomReadFailure)?;
        let mut cpu = Cpu8080::new(rom);
        cpu.run()?
    }
    Ok(())
}
