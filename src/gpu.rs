use std::cmp::{max, min};
use std::mem::swap;

use util::*;

static DEFAULT_PALETTE: [u32; 16] = [
    0x000000, // Black (transparent in `FG`)
    0x000000, // Black
    0x888888, // Gray
    0xBF3932, // Red
    0xDE7AAE, // Pink
    0x4C3D21, // Dark brown
    0x905F25, // Brown
    0xE49452, // Orange
    0xEAD979, // Yellow
    0x537A3B, // Green
    0xABD54A, // Light green
    0x252E38, // Dark blue
    0x00467F, // Blue
    0x68ABCC, // Light blue
    0xBCDEE4, // Sky blue
    0xFFFFFF, // White
];

pub struct Gpu {
    /// Frame foreground buffer
    fg: [u8; 320 * 240],
    /// Frame background color
    bg: u8,
    /// The width of a sprite
    spritew: u8,
    /// The height of a sprite
    spriteh: u8,
    /// Flag to signal that the sprite should be flipped horizontally
    hflip: bool,
    /// Flag to signal that the sprite should be flipped vertically
    vflip: bool,
    /// Color palette
    palette: [u32; 16],
    /// Flag to signal that a new frame has been drawn
    vblank: bool,
}

impl Gpu {
    pub fn new() -> Gpu {
        Gpu {
            fg: [0; 320 * 240],
            bg: 0,
            spritew: 0,
            spriteh: 0,
            hflip: false,
            vflip: false,
            vblank: false,
            palette: DEFAULT_PALETTE,
        }
    }

    pub fn clear(&mut self) {
        self.fg = [0; 320 * 240];
        self.bg = 0;
    }

    pub fn vblank(&self) -> bool {
        self.vblank
    }

    pub fn set_vblank(&mut self, val: bool) {
        self.vblank = val;
    }

    pub fn set_bg(&mut self, n: u8) {
        self.bg = n;
    }

    pub fn set_sprite_size(&mut self, w: u8, h: u8) {
        self.spritew = w;
        self.spriteh = h;
    }

    pub fn set_hflip(&mut self, val: bool) {
        self.hflip = val;
    }

    pub fn set_vflip(&mut self, val: bool) {
        self.vflip = val;
    }

    pub fn set_palette(&mut self, buf: [u32; 16]) {
        self.palette = buf;
    }

    /// Draw a sprite to the foreground
    pub fn draw(&mut self, x: i16, y: i16, buf: &[u8]) -> bool {
        let mut overlap = false;

        let (x, y) = (x as i32, y as i32);
        let (w, h) = (self.spritew as i32, self.spriteh as i32);

        for j in max(0, y)..min(240, y + h) {
            for i in (max(-1, x)..min(320, x + w * 2)).filter(|z| (z - x) % 2 == 0) {
                let a = if !self.hflip { (i - x) / 2 } else { w - (i - x) / 2 - 1 } as usize;
                let b = if !self.vflip { j - y } else { h - (j - y) - 1 } as usize;
                let (mut high, mut low) = half_bytes(buf[a + b * w as usize]);
                if self.hflip { swap(&mut high, &mut low) };
                let p = i + j * 320;
                if i >= 0 && high != 0 {
                    let p = p as usize;
                    if self.fg[p] != 0 {
                        overlap = true;
                    }
                    self.fg[p] = high;
                }
                if i < 319 && low != 0 {
                    let p = (p + 1) as usize;
                    if self.fg[p] != 0 {
                        overlap = true;
                    }
                    self.fg[p] = low;
                }
            }
        }

        overlap
    }

    /// Render the frame to a buffer of the size 320x240
    pub fn render(&mut self, buffer: &mut [u32]) {
        let bgc = self.palette[self.bg as usize];

        for (buf, &fg) in buffer.iter_mut().zip(self.fg.iter()) {
            *buf = if fg != 0 { self.palette[fg as usize] } else { bgc };
        }

        self.vblank = true;
    }
}
