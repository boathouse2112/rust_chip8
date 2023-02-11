mod cpu;
use cpu::CPU;
use std::fmt::Write;
use std::fs;

fn main() {
    let rom = fs::read("roms/test_opcode.ch8").unwrap();

    // Print rom
    // let mut s = String::new();
    // for byte in &rom {
    //     write!(&mut s, "{:0>4o} ", byte).expect("");
    // }
    // println!("{}", s);

    // Load ROM into CPU memory
    let mut cpu = CPU::new();
    cpu.memory[200..200 + rom.len()].clone_from_slice(&rom[..]);
}
