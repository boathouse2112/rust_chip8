mod cpu;
use cpu::CPU;
use std::fs;

fn print_array(arr: [[bool; 64]; 32]) {
    for row in arr {
        for cell in row {
            let char = if cell { "O" } else { "_" };
            print!("{} ", char);
        }
        println!();
    }
}

fn main() {
    let rom = fs::read("roms/test_suite.ch8").unwrap();

    // Load ROM into CPU memory
    let mut cpu = CPU::new();
    cpu.memory[0x200..0x200 + rom.len()].clone_from_slice(&rom[..]);

    // Test print 0

    // cpu.memory[0x200] = 0xF0;
    // cpu.memory[0x201] = 0x29;

    // cpu.memory[0x202] = 0xD0;
    // cpu.memory[0x203] = 0x05;

    // for _ in 0..4 {
    //     cpu.run_cycle(0);
    // }

    // print_array(cpu.display);

    // Test run_instruction
    cpu.memory[0x1FF] = 1;
    for _ in 0..1_000 {
        cpu.run_cycle(0)
    }
    print_array(cpu.display);
}
