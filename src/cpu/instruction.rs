use std::fmt::Write;

use rand::Rng;

use cpu::Cpu;
use util::*;

macro_rules! instructions {
    ($( $byte:pat => ($mnemonic:expr, [$( $parm:ident ),*], $action:expr) ),*) => {
        pub fn run_instruction(cpu: &mut Cpu, addr: usize) -> Result<(), String> {
            Ok(match cpu.memory[addr] {
                $( $byte => run_instruction!($action, [$($parm),*], cpu, cpu.memory[addr..]), )*
                _ => return Err(format!("Unknown Opcode 0x{:02X}", cpu.memory[addr]))
            })
        }

        pub fn format_instruction(cpu: &Cpu, addr: usize) -> Result<String, String> {
            #[allow(unused_mut, unused_must_use)]
            Ok(match cpu.memory[addr] {
                $( $byte => format_instruction!($mnemonic, [$($parm),*], cpu.memory[addr..]), )*
                _ => return Err(format!("Unknown Opcode 0x{:02X}", cpu.memory[addr]))
            })
        }
    };
}

macro_rules! run_instruction {
    ($action:expr, [$( $parm:ident ),*], $cpu:expr, $bytes:expr) => {{
        $( let $parm = parm!($parm, $bytes); )*
        $action($cpu, $( $parm ),* )
    }};
}

macro_rules! format_instruction {
    ($mnemonic:expr, [cond, $( $parm:ident ),*], $bytes:expr) => {{
        let mut ret = format!("{}", $mnemonic);
        format_parm!(cond, $bytes, &mut ret);
        $( ret.push(' '); format_parm!($parm, $bytes, &mut ret); )*
        ret
    }};
    ($mnemonic:expr, [$( $parm:ident ),*], $bytes:expr) => {{
        let mut ret = format!("{}", $mnemonic);
        $( ret.push(' '); format_parm!($parm, $bytes, &mut ret); )*
        ret
    }};
}

macro_rules! parm {
    (rx, $bytes:expr) => { half_bytes($bytes[1]).1 };
    (ry, $bytes:expr) => { half_bytes($bytes[1]).0 };
    (rz, $bytes:expr) => { half_bytes($bytes[2]).1 };
    (hhll, $bytes:expr) => { *deserialize::<u16>(&$bytes[2..]) };
    (cond, $bytes:expr) => { $bytes[1] };
    (ad, $bytes:expr) => { $bytes[1] };
    (ll, $bytes:expr) => { $bytes[2] };
    (hh, $bytes:expr) => { $bytes[3] };
}

macro_rules! format_parm {
    (rx, $bytes:expr, $writer:expr) => { write!($writer, "R{:X}", parm!(rx, $bytes)) };
    (ry, $bytes:expr, $writer:expr) => { write!($writer, "R{:X}", parm!(ry, $bytes)) };
    (rz, $bytes:expr, $writer:expr) => { write!($writer, "R{:X}", parm!(rz, $bytes)) };
    (cond, $bytes:expr, $writer:expr) => {
        let cond = match parm!(cond, $bytes) {
            0x0 => "Z",
            0x1 => "NZ",
            0x2 => "N",
            0x3 => "NN",
            0x4 => "P",
            0x5 => "O",
            0x6 => "NO",
            0x7 => "A",
            0x8 => "AE",
            0x9 => "B",
            0xA => "BE",
            0xB => "G",
            0xC => "GE",
            0xD => "L",
            0xE => "LE",
            _ => "UNKNOWN",
        };
        write!($writer, "{}", cond)
    };
    ($parm:ident, $bytes:expr, $writer:expr) => { write!($writer, "{}", parm!($parm, $bytes)) };
}

instructions! {
    0x00 => ("NOP", [], |_| {}),
    0x01 => ("CLS", [], cls),
    0x02 => ("VBLNK", [], vblnk),
    0x03 => ("BGC", [ll], bgc),
    0x04 => ("SPR", [ll, hh], spr),
    0x05 => ("DRW", [rx, ry, hhll], drw),
    0x06 => ("DRW", [rx, ry, rz], drw_r),
    0x07 => ("RND", [rx, hhll], rnd),
    0x08 => ("FLIP", [hh], flip),
    0x09 => ("SND0", [], snd0),
    0x0A => ("SND1", [hhll], snd1),
    0x0B => ("SND2", [hhll], snd2),
    0x0C => ("SND3", [hhll], snd3),
    0x0D => ("SNP", [rx, hhll], snp),
    0x0E => ("SNG", [ad, ll, hh], sng),
    0x10 => ("JMP", [hhll], jmp),
    0x11 => ("JMC", [hhll], |cpu, hhll| jx(cpu, 0x9, hhll)),
    0x12 => ("J", [cond, hhll], jx),
    0x13 => ("JME", [rx, ry, hhll], jme),
    0x14 => ("CALL", [hhll], call),
    0x15 => ("RET", [], ret),
    0x16 => ("JMP_R", [rx], jmp_r),
    0x17 => ("C", [cond, hhll], cx),
    0x18 => ("CALL", [rx], call_r),
    0x20 => ("LDI", [rx, hhll], ldi_r),
    0x21 => ("LDI", [hhll], ldi_sp),
    0x22 => ("LDM", [rx, hhll], ldm),
    0x23 => ("LDM", [rx, ry], ldm_r),
    0x24 => ("MOV", [rx, ry], mov),
    0x30 => ("STM", [rx, hhll], stm),
    0x31 => ("STM", [rx, ry], stm_r),
    0x40 => ("ADDI", [rx, hhll], |cpu, rx, hhll| math(cpu, rx, hhll, add)),
    0x41 => ("ADD", [rx, ry], |cpu, rx, ry| math_r(cpu, rx, ry, rx, add)),
    0x42 => ("ADD", [rx, ry, rz], |cpu, rx, ry, rz| math_r(cpu, rx, ry, rz, add)),
    0x50 => ("SUBI", [rx, hhll], |cpu, rx, hhll| math(cpu, rx, hhll, sub)),
    0x51 => ("SUB", [rx, ry], |cpu, rx, ry| math_r(cpu, rx, ry, rx, sub)),
    0x52 => ("SUB", [rx, ry, rz], |cpu, rx, ry, rz| math_r(cpu, rx, ry, rz, sub)),
    0x53 => ("CMPI", [rx, hhll], |cpu, rx, hhll| check(cpu, rx, hhll, sub)),
    0x54 => ("CMP", [rx, ry], |cpu, rx, ry| check_r(cpu, rx, ry, sub)),
    0x60 => ("ANDI", [rx, hhll], |cpu, rx, hhll| math(cpu, rx, hhll, |_, a, b| a & b)),
    0x61 => ("AND", [rx, ry], |cpu, rx, ry| math_r(cpu, rx, ry, rx, |_, a, b| a & b)),
    0x62 => ("AND", [rx, ry, rz], |cpu, rx, ry, rz| math_r(cpu, rx, ry, rz, |_, a, b| a & b)),
    0x63 => ("TSTI", [rx, hhll], |cpu, rx, hhll| check(cpu, rx, hhll, |_, a, b| a & b)),
    0x64 => ("TST", [rx, ry], |cpu, rx, ry| check_r(cpu, rx, ry, |_, a, b| a & b)),
    0x70 => ("ORI", [rx, hhll], |cpu, rx, hhll| math(cpu, rx, hhll, |_, a, b| a | b)),
    0x71 => ("OR", [rx, ry], |cpu, rx, ry| math_r(cpu, rx, ry, rx, |_, a, b| a | b)),
    0x72 => ("OR", [rx, ry, rz], |cpu, rx, ry, rz| math_r(cpu, rx, ry, rz, |_, a, b| a | b)),
    0x80 => ("XORI", [rx, hhll], |cpu, rx, hhll| math(cpu, rx, hhll, |_, a, b| a ^ b)),
    0x81 => ("XOR", [rx, ry], |cpu, rx, ry| math_r(cpu, rx, ry, rx, |_, a, b| a ^ b)),
    0x82 => ("XOR", [rx, ry, rz], |cpu, rx, ry, rz| math_r(cpu, rx, ry, rz, |_, a, b| a ^ b)),
    0x90 => ("MULI", [rx, hhll], |cpu, rx, hhll| math(cpu, rx, hhll, mul)),
    0x91 => ("MUL", [rx, ry], |cpu, rx, ry| math_r(cpu, rx, ry, rx, mul)),
    0x92 => ("MUL", [rx, ry, rz], |cpu, rx, ry, rz| math_r(cpu, rx, ry, rz, mul)),
    0xA0 => ("DIVI", [rx, hhll], |cpu, rx, hhll| math(cpu, rx, hhll, div)),
    0xA1 => ("DIV", [rx, ry], |cpu, rx, ry| math_r(cpu, rx, ry, rx, div)),
    0xA2 => ("DIV", [rx, ry, rz], |cpu, rx, ry, rz| math_r(cpu, rx, ry, rz, div)),
    0xA3 => ("MODI", [rx, hhll], |cpu, rx, hhll| math(cpu, rx, hhll, |_, a, b| (a % b + b) % b)),
    0xA4 => ("MOD", [rx, ry], |cpu, rx, ry| math_r(cpu, rx, ry, rx, |_, a, b| (a % b + b) % b)),
    0xA5 => ("MOD", [rx, ry, rz], |cpu, rx, ry, rz| math_r(cpu, rx, ry, rz, |_, a, b| (a % b + b) % b)),
    0xA6 => ("REMI", [rx, hhll], |cpu, rx, hhll| math(cpu, rx, hhll, |_, a, b| a % b)),
    0xA7 => ("REM", [rx, ry], |cpu, rx, ry| math_r(cpu, rx, ry, rx, |_, a, b| a % b)),
    0xA8 => ("REM", [rx, ry, rz], |cpu, rx, ry, rz| math_r(cpu, rx, ry, rz, |_, a, b| a % b)),
    0xB0 => ("SHL", [rx, hhll], |cpu, rx, hhll| math(cpu, rx, hhll, |_, a, b| a << b)),
    0xB1 => ("SHR", [rx, hhll], |cpu, rx, hhll| math(cpu, rx, hhll, |_, a, b| (a as u16 >> b) as i16)),
    0xB2 => ("SAR", [rx, hhll], |cpu, rx, hhll| math(cpu, rx, hhll, |_, a, b| a >> b)),
    0xB3 => ("SHL", [rx, ry], |cpu, rx, ry| math_r(cpu, rx, ry, rx, |_, a, b| a << b)),
    0xB4 => ("SHR", [rx, ry], |cpu, rx, ry| math_r(cpu, rx, ry, rx, |_, a, b| (a as u16 >> b) as i16)),
    0xB5 => ("SAR", [rx, ry], |cpu, rx, ry| math_r(cpu, rx, ry, rx, |_, a, b| a >> b)),
    0xC0 => ("PUSH", [rx], push),
    0xC1 => ("POP", [rx], pop),
    0xC2 => ("PUSHALL", [], pushall),
    0xC3 => ("POPALL", [], popall),
    0xC4 => ("PUSHF", [], pushf),
    0xC5 => ("POPF", [], popf),
    0xD0 => ("PAL", [hhll], pal),
    0xD1 => ("PAL", [rx], pal_r),
    0xE0 => ("NOTI", [rx, hhll], |cpu, rx, hhll| math(cpu, rx, hhll, |_, _, b| !b)),
    0xE1 => ("NOT", [rx], |cpu, rx| math_r(cpu, rx, 0, rx, |_, a, _| !a)),
    0xE2 => ("NOT", [rx, ry], |cpu, rx, ry| math_r(cpu, 0, ry, rx, |_, _, b| !b)),
    0xE3 => ("NEGI", [rx, hhll], |cpu, rx, hhll| math(cpu, rx, hhll, |_, _, b| -b)),
    0xE4 => ("NEG", [rx], |cpu, rx| math_r(cpu, rx, 0, rx, |_, a, _| -a)),
    0xE5 => ("NEG", [rx, ry], |cpu, rx, ry| math_r(cpu, 0, ry, rx, |_, _, b| -b))
}

/// Test a given condition and return the result
fn condition(cpu: &mut Cpu, cond: u8) -> bool {
    match cond {
        0x0 => cpu.zero(), // Equal
        0x1 => !cpu.zero(), // Not equal
        0x2 => cpu.negative(), // Negative
        0x3 => !cpu.negative(), // Not negative
        0x4 => !cpu.negative() && !cpu.zero(), // Positive
        0x5 => cpu.overflow(), // Overflow
        0x6 => !cpu.overflow(), // No overflow
        0x7 => !cpu.carry() && !cpu.zero(), // Above
        0x8 => !cpu.carry(), // Above equal
        0x9 => cpu.carry(), // Below
        0xA => cpu.carry() || cpu.zero(), // Below equal
        0xB => cpu.overflow() == cpu.negative() && !cpu.zero(), // Signed greater than
        0xC => cpu.overflow() == cpu.negative(), // Signed greater than equal
        0xD => cpu.overflow() != cpu.negative(), // Signed less than
        0xE => cpu.overflow() != cpu.negative() || cpu.zero(), // Signed less Than equal
        _ => false
    }
}

/// Perform a given math operation (e.g. `addi`)
fn math<F: Fn(&mut Cpu, i16, i16) -> i16>(cpu: &mut Cpu, rx: u8, hhll: u16, f: F) {
    cpu.regs.flags = 0;
    let a = cpu.r(rx);
    let res = f(cpu, a, hhll as i16);
    cpu.set_zero(res == 0);
    cpu.set_negative(res < 0);
    cpu.set_r(rx, res);
}

/// Perform a given math operation (e.g. `add_r2`, `add_r3`)
fn math_r<F: Fn(&mut Cpu, i16, i16) -> i16>(cpu: &mut Cpu, rx: u8, ry: u8, rz: u8, f: F) {
    cpu.regs.flags = 0;
    let a = cpu.r(rx);
    let b = cpu.r(ry);
    let res = f(cpu, a, b);
    cpu.set_zero(res == 0);
    cpu.set_negative(res < 0);
    cpu.set_r(rz, res);
}

/// Perform a given math operation and discard the result (e.g. `cmpi`)
fn check<F: Fn(&mut Cpu, i16, i16) -> i16>(cpu: &mut Cpu, rx: u8, hhll: u16, f: F) {
    cpu.regs.flags = 0;
    let a = cpu.r(rx);
    let res = f(cpu, a, hhll as i16);
    cpu.set_zero(res == 0);
    cpu.set_negative(res < 0);
}

/// Perform a given math operation and discard the result (e.g. `cmp_r2`)
fn check_r<F: Fn(&mut Cpu, i16, i16) -> i16>(cpu: &mut Cpu, rx: u8, ry: u8, f: F) {
    let b = cpu.r(ry);
    check(cpu, rx, b as u16, f);
}

/// Add two values and set the flags accordingly
fn add(cpu: &mut Cpu, a: i16, b: i16) -> i16 {
    let (res, o) = a.overflowing_add(b);
    cpu.set_overflow(o);
    cpu.set_carry((a as u16).checked_add(b as u16).is_none());
    res
}

/// Sub two values and set the flags accordingly
fn sub(cpu: &mut Cpu, a: i16, b: i16) -> i16 {
    let (res, o) = a.overflowing_sub(b);
    cpu.set_overflow(o);
    cpu.set_carry((a as u16).checked_sub(b as u16).is_none());
    res
}

/// Multiply two values and set the flags accordingly
fn mul(cpu: &mut Cpu, a: i16, b: i16) -> i16 {
    let (res, o) = a.overflowing_mul(b);
    cpu.set_overflow(o);
    cpu.set_carry((a as u16).checked_mul(b as u16).is_none());
    res
}

/// Divide two values and set the flags accordingly
fn div(cpu: &mut Cpu, a: i16, b: i16) -> i16 {
    let (res, o) = a.overflowing_div(b);
    cpu.set_overflow(o);
    cpu.set_carry(a % b > 0);
    res
}

fn cls(cpu: &mut Cpu) {
    cpu.gpu.clear();
}

fn vblnk(cpu: &mut Cpu) {
    if !cpu.gpu.vblank() {
        cpu.regs.pc -= 4;
        cpu.wait_vblank = true;
    }
}

fn bgc(cpu: &mut Cpu, n: u8) {
    cpu.gpu.set_bg(n);
}

fn spr(cpu: &mut Cpu, ll: u8, hh: u8) {
    cpu.gpu.set_sprite_size(ll, hh);
}

fn drw(cpu: &mut Cpu, rx: u8, ry: u8, hhll: u16) {
    let x = cpu.r(rx);
    let y = cpu.r(ry);
    let overlap = {
        let sprite = &cpu.memory.as_slice()[hhll as usize..];
        cpu.gpu.draw(x, y, sprite)
    };
    cpu.set_carry(overlap);
}

fn drw_r(cpu: &mut Cpu, rx: u8, ry: u8, rz: u8) {
    let addr = cpu.r(rz) as u16;
    drw(cpu, rx, ry, addr);
}

fn rnd(cpu: &mut Cpu, rx: u8, hhll: u16) {
    let n = cpu.rng.gen_range(0, hhll as u32 + 1);
    cpu.set_r(rx, n as i16);
}

fn flip(cpu: &mut Cpu, n: u8) {
    cpu.gpu.set_hflip(n > 1);
    cpu.gpu.set_vflip(n % 2 > 0);
}

fn snd0(cpu: &mut Cpu) {
    cpu.apu.stop();
}

fn snd1(cpu: &mut Cpu, hhll: u16) {
    cpu.apu.play(500, hhll, false);
}

fn snd2(cpu: &mut Cpu, hhll: u16) {
    cpu.apu.play(1000, hhll, false);
}

fn snd3(cpu: &mut Cpu, hhll: u16) {
    cpu.apu.play(1500, hhll, false);
}

fn snp(cpu: &mut Cpu, rx: u8, hhll: u16) {
    let addr = cpu.r(rx) as u16;
    let hz = cpu.read(addr);
    cpu.apu.play(hz, hhll, true);
}

fn sng(cpu: &mut Cpu, ad: u8, sr: u8, vt: u8) {
    let (a, d) = half_bytes(ad);
    let (s, r) = half_bytes(sr);
    let (v, t) = half_bytes(vt);
    cpu.apu.settings(a, d, s, r, v, t);
}

fn jmp(cpu: &mut Cpu, hhll: u16) {
    cpu.regs.pc = hhll;
}

fn jx(cpu: &mut Cpu, cond: u8, hhll: u16) {
    if condition(cpu, cond) {
        jmp(cpu, hhll);
    }
}

fn jme(cpu: &mut Cpu, rx: u8, ry: u8, hhll: u16) {
    if cpu.r(rx) == cpu.r(ry) {
        jmp(cpu, hhll);
    }
}

fn call(cpu: &mut Cpu, hhll: u16) {
    let pc = cpu.regs.pc;
    let sp = cpu.regs.sp;
    cpu.write(sp, pc as i16);
    cpu.regs.sp += 2;
    cpu.regs.pc = hhll;
}

fn ret(cpu: &mut Cpu) {
    cpu.regs.sp -= 2;
    cpu.regs.pc = cpu.read(cpu.regs.sp);
}

fn jmp_r(cpu: &mut Cpu, rx: u8) {
    let addr = cpu.r(rx) as u16;
    jmp(cpu, addr);
}

fn cx(cpu: &mut Cpu, cond: u8, hhll: u16) {
    if condition(cpu, cond) {
        call(cpu, hhll);
    }
}

fn call_r(cpu: &mut Cpu, rx: u8) {
    let addr = cpu.r(rx) as u16;
    call(cpu, addr);
}

fn ldi_r(cpu: &mut Cpu, rx: u8, hhll: u16) {
    cpu.set_r(rx, hhll as i16);
}

fn ldi_sp(cpu: &mut Cpu, hhll: u16) {
    cpu.regs.sp = hhll;
}

fn ldm(cpu: &mut Cpu, rx: u8, hhll: u16) {
    let val = cpu.read(hhll);
    cpu.set_r(rx, val);
}

fn ldm_r(cpu: &mut Cpu, rx: u8, ry: u8) {
    let addr = cpu.r(ry) as u16;
    ldm(cpu, rx, addr);
}

fn mov(cpu: &mut Cpu, rx: u8, ry: u8) {
    let val = cpu.r(ry);
    cpu.set_r(rx, val);
}

fn stm(cpu: &mut Cpu, rx: u8, hhll: u16) {
    let val = cpu.r(rx);
    cpu.write(hhll, val);
}

fn stm_r(cpu: &mut Cpu, rx: u8, ry: u8) {
    let addr = cpu.r(ry) as u16;
    stm(cpu, rx, addr);
}

fn push(cpu: &mut Cpu, rx: u8) {
    let sp = cpu.regs.sp;
    let val = cpu.r(rx);
    cpu.write(sp, val);
    cpu.regs.sp += 2;
}

fn pop(cpu: &mut Cpu, rx: u8) {
    cpu.regs.sp -= 2;
    let sp = cpu.regs.sp;
    let val = cpu.read(sp);
    cpu.set_r(rx, val);
}

fn pushall(cpu: &mut Cpu) {
    for rx in 0..16 {
        push(cpu, rx);
    }
}

fn popall(cpu: &mut Cpu) {
    for rx in (0..16).rev() {
        pop(cpu, rx);
    }
}

fn pushf(cpu: &mut Cpu) {
    let sp = cpu.regs.sp;
    let val = cpu.regs.flags;
    cpu.write(sp, val as i16);
    cpu.regs.sp += 2;
}

fn popf(cpu: &mut Cpu) {
    cpu.regs.sp -= 2;
    let sp = cpu.regs.sp;
    let val = cpu.read(sp);
    cpu.regs.flags = val;
}

fn pal(cpu: &mut Cpu, hhll: u16) {
    let mut palette = [0; 16];
    let m = &cpu.memory.as_slice()[hhll as usize..];
    for i in 0..16 {
        palette[i] = (m[i * 3] as u32) << 16 | (m[i * 3 + 1] as u32) << 8 | m[i * 3 + 2] as u32;
    }
    cpu.gpu.set_palette(palette);
}

fn pal_r(cpu: &mut Cpu, rx: u8) {
    let addr = cpu.r(rx) as u16;
    pal(cpu, addr);
}
