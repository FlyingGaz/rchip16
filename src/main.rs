extern crate cpal;
extern crate minifb;
extern crate rand;

mod cpu;
mod gpu;
mod apu;
mod rom;
mod debugger;
mod util;

use std::env;
use std::thread;
use std::time::{Duration, Instant};

use minifb::{Key, Scale, Window, WindowOptions};

use cpu::*;
use gpu::*;
use apu::*;
use rom::*;
use debugger::*;
use util::*;

fn main() {
    let rom_file = match env::args().nth(1) {
        Some(f) => f,
        None => panic!("No rom file specified"),
    };

    let rom = match Rom::load(&rom_file) {
        Ok(rom) => rom,
        Err(e) => panic!("Error loading rom file: {}", e),
    };

    println!(" version: {}", rom.version());
    println!("    size: {}", rom.size());
    println!("   start: {}", rom.start());
    let (expected, actual) = rom.checksum();
    println!("checksum: {} {}", actual, if actual == expected { "OK" } else { "NOT OK" });

    let supported_version = Version(1, 3);
    if rom.version() > supported_version {
        println!(" warning: only version {} and lower are supported", supported_version);
    }

    let gpu = Gpu::new();
    let apu = Apu::new(0.1);
    let mut cpu = Cpu::new(gpu, apu, &rom);
    let mut debugger = Debugger::new();

    let limited = !env::args().any(|a| a == "--unlimited");
    if env::args().any(|a| a == "--break") {
        debugger.set_break();
    }

    let title = format!("rchip16 - {}", rom_file);
    let options = WindowOptions { scale: Scale::X2, ..WindowOptions::default() };
    let mut win = Window::new(&title, 320, 240, options).unwrap();
    let mut winbuf = vec![0; 320 * 240];

    let frame_instr = 1_000_000 / 60;
    let frame_time = Duration::from_millis(1_000 / 60);

    while win.is_open() && !win.is_key_down(Key::Escape) {
        let start = Instant::now();

        if win.is_key_down(Key::F12) {
            debugger.set_break();
        }

        cpu.set_input(read_input(&win));

        /* BENCH */ let dbg_start = Instant::now();
        for _ in 0..frame_instr {
            debugger.step(&mut cpu);
            cpu.step();
            if cpu.wait_vblank() {
                break;
            }
        }

        cpu.render(&mut winbuf);
        win.update_with_buffer(&winbuf).unwrap();

        let delta = start.elapsed();
        if limited && delta < frame_time {
            thread::sleep(frame_time - delta);
        }
    }
}

/// Read inputs for controller 1 & 2
fn read_input(win: &Window) -> (u8, u8) {
    use Key::*;

    // Keys are set in the order Up, Down, Left, Right, Select, Start, A, B
    let (mut one, mut two) = (0, 0);
    for (i, &key) in [Up, Down, Left, Right, RightShift, Enter, N, M].iter().enumerate() {
        set_bitflag(&mut one, i as u8, win.is_key_down(key));
    }
    for (i, &key) in [W, S, A, D, LeftShift, Tab, X, C].iter().enumerate() {
        set_bitflag(&mut two, i as u8, win.is_key_down(key));
    }

    (one, two)
}
