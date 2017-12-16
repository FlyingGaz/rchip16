pub mod instruction;

use rand::{weak_rng, XorShiftRng};

use gpu::Gpu;
use apu::Apu;
use rom::Rom;
use util::*;

use self::instruction::*;

pub struct Cpu {
    /// CPU registers
    regs: Registers,
    /// The `ROM` / `RAM` memory, it has a size of 64 KB
    memory: Vec<u8>,
    /// The GPU
    gpu: Gpu,
    /// The APU
    apu: Apu,
    /// The random number generator
    rng: XorShiftRng,
    /// Flag to signal that the cpu is waiting for `VBLNK`
    wait_vblank: bool,
}

struct Registers {
    /// program counter `PC`
    pc: u16,
    /// stack pointer `SP` (start at 0xFDF0/512 B)
    sp: u16,
    /// general purpose registers (`R0` .. `RF`)
    r: [i16; 16],
    /// flag register `FLAGS` (`carry`, `zero`, `overflow`, `negative`)
    flags: u8,
}

impl Cpu {
    pub fn new(gpu: Gpu, apu: Apu, rom: &Rom) -> Cpu {
        let regs = Registers {
            pc: rom.start(),
            sp: 0xFDF0,
            r: [0; 16],
            flags: 0,
        };

        let mut memory = vec![0; 2usize.pow(16)];
        memory[..rom.size() as usize].copy_from_slice(rom.rom());

        Cpu {
            regs: regs,
            memory: memory,
            gpu: gpu,
            apu: apu,
            rng: weak_rng(),
            wait_vblank: false,
        }
    }

    /// Execute one CPU cycle
    pub fn step(&mut self) {
        self.wait_vblank = false;

        // Fetch `pc`, increase `pc` and run instruction at `pc`
        let pc = self.regs.pc;
        self.regs.pc += 4;
        if let Err(e) = run_instruction(self, pc as usize) {
            panic!("Invalid instruction at 0x{:02X} ({})", pc, e);
        }

        self.gpu.set_vblank(false);
    }

    pub fn format_instruction(&self, addr: u16) -> Result<String, String> {
        format_instruction(self, addr as usize)
    }

    /// Read one value from the memory at the specified address
    pub fn read<T: Copy>(&self, addr: u16) -> T {
        *deserialize(&self.memory[addr as usize..])
    }

    /// Write one value to the memory at the specified address
    pub fn write<T>(&mut self, addr: u16, val: T) {
        let addr = addr as usize;
        let buf = serialize(&val);
        self.memory[addr..addr + buf.len()].copy_from_slice(buf);
    }

    pub fn render(&mut self, buffer: &mut [u32]) {
        self.gpu.render(buffer);
    }

    /// Get the carry flag
    pub fn carry(&self) -> bool {
        bitflag(self.regs.flags, 1)
    }

    /// Set the carry flag
    pub fn set_carry(&mut self, val: bool) {
        set_bitflag(&mut self.regs.flags, 1, val);
    }

    /// Get the zero flag
    pub fn zero(&self) -> bool {
        bitflag(self.regs.flags, 2)
    }

    /// Set the zero flag
    pub fn set_zero(&mut self, val: bool) {
        set_bitflag(&mut self.regs.flags, 2, val);
    }

    /// Get the overflow flag
    pub fn overflow(&self) -> bool {
        bitflag(self.regs.flags, 6)
    }

    /// Set the overflow flag
    pub fn set_overflow(&mut self, val: bool) {
        set_bitflag(&mut self.regs.flags, 6, val);
    }

    /// Get the negative flag
    pub fn negative(&self) -> bool {
        bitflag(self.regs.flags, 7)
    }

    /// Set the negative flag
    pub fn set_negative(&mut self, val: bool) {
        set_bitflag(&mut self.regs.flags, 7, val);
    }

    pub fn pc(&self) -> u16 {
        self.regs.pc
    }

    pub fn sp(&self) -> u16 {
        self.regs.sp
    }

    pub fn r(&self, index: u8) -> i16 {
        self.regs.r[index as usize]
    }

    pub fn set_r(&mut self, index: u8, value: i16) {
        self.regs.r[index as usize] = value;
    }

    pub fn set_input(&mut self, (one, two): (u8, u8)) {
        self.memory[0xFFF0] = one;
        self.memory[0xFFF2] = two;
    }

    pub fn wait_vblank(&self) -> bool {
        self.wait_vblank
    }
}
