use std::{
    fs::File,
    io::{BufReader, Read},
};

use emulator::{Cpu8080, InvalidFile, Result, SwiftCallbacks};

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
    let invader = std::env::current_dir()?.join("roms/invaders");

    println!(
        "executing {:?}....",
        invader.file_name().ok_or(InvalidFile)?
    );
    let bytes = BufReader::new(File::open(invader)?).bytes();
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
