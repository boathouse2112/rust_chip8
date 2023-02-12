mod cpu;
use cpu::CPU;
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
    cpu.memory[0x200..0x200 + rom.len()].clone_from_slice(&rom[..]);

    // Test run_instruction
    cpu.i = 0x50;
    cpu.run_cycle(0xD005);
    println!(
        "{:?}",
        cpu.display.map(|row| { row.map(|cell| { cell as u8 }) })
    );
}
