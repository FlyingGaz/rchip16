use std::fmt;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;
use std::io::Result;

use crate::util::*;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct Version(pub u8, pub u8);

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.0, self.1)
    }
}

pub struct Rom {
    version: Version,
    size: u32,
    start: u16,
    checksum: u32,
    rom: Vec<u8>,
}

impl Rom {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Rom> {
        let mut file = File::open(path)?;
        let mut magic = [0; 4];
        file.read_exact(&mut magic)?;

        // check the rom file for the magic number
        if &magic == b"CH16" {
            let mut header = [0; 12];
            file.read_exact(&mut header)?;
            let size = *deserialize(&header[2..]);
            let mut rom = vec![0; size as usize];
            file.read_exact(rom.as_mut_slice())?;
            Ok(Rom {
                version: { let (h, l) = half_bytes(header[1]); Version(h, l) },
                size,
                start: *deserialize(&header[6..]),
                checksum: *deserialize(&header[8..]),
                rom,
            })
        } else {
            let mut rom = vec![0; 4];
            rom.copy_from_slice(&magic);
            let len = file.read_to_end(&mut rom)?;
            Ok(Rom {
                version: Version(0, 0),
                size: len as u32 + 4,
                start: 0,
                checksum: 0,
                rom,
            })
        }
    }

    pub fn version(&self) -> Version {
        self.version
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn start(&self) -> u16 {
        self.start
    }

    pub fn checksum(&self) -> (u32, u32) {
        (self.checksum, crc32(&self.rom))
    }

    pub fn rom(&self) -> &Vec<u8> {
        &self.rom
    }
}
