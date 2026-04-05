use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use i8080emulator::{Cpu8080, ExecutionState};

fn main() {
    let cpudiag_prog = Path::new(env!("CARGO_MANIFEST_DIR")).join("diagnosis_program/cpudiag");

    println!("executing CPU diagnosis...");
    let bytes =
        BufReader::new(File::open(cpudiag_prog).expect("failed to open cpudiag ROM")).bytes();
    let rom = bytes
        .collect::<std::result::Result<Vec<u8>, std::io::Error>>()
        .expect("failed to read cpudiag ROM");
    let mut ram = vec![0; 0x200];
    let mut cpu = Cpu8080::cpudiag_new(&rom, &mut ram);
    loop {
        if matches!(
            cpu.run().expect("cpudiag execution failed").state,
            ExecutionState::Halted
        ) {
            break;
        }
    }
}
