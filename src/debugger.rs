use std::io;
use std::io::Write;
use std::collections::HashSet;
use std::process;

use crate::cpu::*;

pub struct Debugger {
    run: bool,
    break_pc: HashSet<u16>,
    break_op: HashSet<String>,
}

impl Debugger {
    pub fn new() -> Debugger {
        Debugger { run: false, break_pc: HashSet::new(), break_op: HashSet::new() }
    }

    /// Set the break flag
    pub fn set_break(&mut self) {
        self.run = true;
    }

    /// Perform one step and run the debugger if requested
    pub fn step(&mut self, cpu: &mut Cpu) {

        if !self.break_pc.is_empty() {
            if self.break_pc.contains(&cpu.pc()) {
                self.run = true;
            }
        }

        if !self.break_op.is_empty() {
            if let Ok(instr) = cpu.format_instruction(cpu.pc()) {
                if self.break_op.iter().any(|op| instr.starts_with(op)) {
                    self.run = true;
                }
            }
        }

        if self.run {
            self.run = false;
            self.run(cpu);
        }
    }

    /// Run the debugger
    pub fn run(&mut self, cpu: &mut Cpu) {
        print_regs(cpu);
        print_current_instructions(cpu);

        loop {
            print!("> ");
            io::stdout().flush().unwrap();
            let mut line = String::new();
            io::stdin().read_line(&mut line).unwrap();
            let mut input = line.split_whitespace();
            match input.next() {
                None | Some("s") => { self.run = true; break },
                Some("c") => break,
                Some("bc") => match input.next().and_then(|s| s.parse().ok()) {
                    Some(pc) => if self.break_pc.contains(&pc) {
                        println!("breakpoint removed from PC 0x{:04X}", pc);
                        self.break_pc.remove(&pc);
                    } else {
                        println!("breakpoint added to PC 0x{:04X}", pc);
                        self.break_pc.insert(pc);
                    },
                    None => println!("invalid argument"),
                },
                Some("bo") => match input.next().map(|s| s.to_uppercase()) {
                    Some(op) => if self.break_op.contains(&op) {
                            println!("breakpoint removed from OPCODE {}", op);
                            self.break_op.remove(&op);
                    } else {
                            println!("breakpoint added to OPCODE {}", op);
                            self.break_op.insert(op);
                    },
                    None => println!("invalid argument"),
                },
                Some("q") => process::exit(0),
                Some(_) => println!("unknown command"),
            }
        }
    }
}

fn format_instruction(cpu: &Cpu, addr: u16) -> String {
    match cpu.format_instruction(addr) {
        Ok(instr) => instr,
        Err(_) => "UNKNOWN INSTRUCTION".into(),
    }
}

/// Print the instructions around the current program counter
fn print_current_instructions(cpu: &Cpu) {
    let pc = cpu.pc();
    if pc > 0 {
        println!("{:>19} {}", "|", format_instruction(cpu, pc - 4));
    }
    println!("  PC 0x{:04X} ----> | {}", pc, format_instruction(cpu, pc));
    for i in 1..4 {
        println!("{:>19} {}", "|", format_instruction(cpu, pc + i * 4));
    }
}

/// Print all registers
fn print_regs(cpu: &Cpu) {
    println!("|--------|--------|--------|--------|--------|--------|--------|--------|");
    println!("| pc     | sp     |        |        | carry  | zero   | overfl | negati |");
    println!("| {:>6} | {:>6} |        |        | {:<6} | {:<6} | {:<6} | {:<6} |",
             cpu.pc(), cpu.sp(), cpu.carry(), cpu.zero(), cpu.overflow(), cpu.negative());
    println!("|--------|--------|--------|--------|--------|--------|--------|--------|");
    for i in 0..8 {
        print!("| R{:X}     ", i);
    }
    println!("|");
    for i in 0..8 {
        print!("| {:>6} ", cpu.r(i));
    }
    println!("|");
    println!("|--------|--------|--------|--------|--------|--------|--------|--------|");
    for i in 8..16 {
        print!("| R{:X}     ", i);
    }
    println!("|");
    for i in 8..16 {
        print!("| {:>6} ", cpu.r(i));
    }
    println!("|");
    println!("|--------|--------|--------|--------|--------|--------|--------|--------|");
}
