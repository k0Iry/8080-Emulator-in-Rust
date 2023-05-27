use std::{
    fs::File,
    io::{BufReader, Read},
};

use emulator::{ConditionCodes, Cpu8080, Result, RomReadFailure, RAM_SIZE};

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
        let bytes = BufReader::new(File::open(file)?).bytes();
        let rom = bytes.collect::<std::result::Result<Vec<u8>, std::io::Error>>()?;
        let mut cpu = Cpu8080 {
            reg_a: 0,
            reg_b: 0,
            reg_c: 0,
            reg_d: 0,
            reg_e: 0,
            reg_h: 0,
            reg_l: 0,
            sp: 0,
            pc: 0,
            rom: &rom.try_into().map_err(|_| RomReadFailure)?,
            ram: [0; RAM_SIZE],
            conditon_codes: ConditionCodes::default(),
            interrupt_enabled: 0,
        };
        while cpu.pc < 0x2000 {
            cpu.execute()?
        }
    }
    Ok(())
}
