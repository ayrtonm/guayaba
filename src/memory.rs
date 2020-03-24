use std::io;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Read;
use std::fs::metadata;
use std::fs::File;
use crate::common::read_word_from_array;
use crate::common::write_word_to_array;
use crate::register::Register;

pub const KB: usize = 1024;

pub struct Memory {
  main_ram: [u8; 2 * KB],
  expansion_1: [u8; 8 * KB],
  scratchpad: [u8; KB],
  io_ports: [u8; 8 * KB],
  expansion_2: [u8; 8 * KB],
  expansion_3: [u8; 2 * KB],
  bios: Box<[u8; 512 * KB]>,
  cache_control: [u8; 512],
}

impl Memory {
  pub fn blank() -> Memory {
    Memory {
      main_ram: [0; 2 * KB],
      expansion_1: [0; 8 * KB],
      scratchpad: [0; KB],
      io_ports: [0; 8 * KB],
      expansion_2: [0; 8 * KB],
      expansion_3: [0; 2 * KB],
      bios: Box::new([0; 512 * KB]),
      cache_control: [0; 512],
    }
  }
  pub fn new(bios_filename: &String) -> io::Result<Self> {
    let mut bios_contents = [0; 512 * KB];
    let mut bios_file = File::open(bios_filename)?;
    let filesize = metadata(bios_filename)?.len();
    assert_eq!(filesize, 512 * KB as u64, "Invalid BIOS file size");
    bios_file.seek(SeekFrom::Start(0))?;
    bios_file.read_exact(&mut bios_contents)?;
    let bios = Box::new(bios_contents);
    Ok(Memory {
      main_ram: [0; 2 * KB],
      expansion_1: [0; 8 * KB],
      scratchpad: [0; KB],
      io_ports: [0; 8 * KB],
      expansion_2: [0; 8 * KB],
      expansion_3: [0; 2 * KB],
      bios,
      cache_control: [0; 512],
    })
  }
  const MAIN_RAM: u32 = 0;
  const MAIN_RAM_END: u32 = Memory::MAIN_RAM + (2 * KB as u32) - 1;

  const EXPANSION_1: u32 = 0x1f00_0000;
  const EXPANSION_1_END: u32 = Memory::EXPANSION_1 + (8 * KB as u32) - 1;

  const SCRATCHPAD: u32 = 0x1f80_0000;
  const SCRATCHPAD_END: u32 = Memory::SCRATCHPAD + (KB as u32) - 1;

  const IO_PORTS: u32 = 0x1f80_1000;
  const IO_PORTS_END: u32 = Memory::IO_PORTS + (8 * KB as u32) - 1;
 
  const EXPANSION_2: u32 = 0x1f80_2000;
  const EXPANSION_2_END: u32 = Memory::EXPANSION_2 + (8 * KB as u32) - 1;

  const EXPANSION_3: u32 = 0x1fa0_0000;
  const EXPANSION_3_END: u32 = Memory::EXPANSION_3 + (2 * KB as u32) - 1;

  const BIOS: u32 = 0x1fc0_0000;
  const BIOS_END: u32 = Memory::BIOS + (512 * KB as u32) - 1;

  const CACHE_CONTROL: u32 = 0xfffe_0000;
  const CACHE_CONTROL_END: u32 = Memory::CACHE_CONTROL + 512 - 1;

  pub fn read_word(&self, address: &Register) -> Register {
    let phys_addr = address.get_value() & 0x1fff_ffff;
    Register::new(match phys_addr {
      (Memory::MAIN_RAM..=Memory::MAIN_RAM_END) => {
        read_word_from_array(&self.main_ram, phys_addr - Memory::MAIN_RAM)
      },
      (Memory::EXPANSION_1..=Memory::EXPANSION_1_END) => {
        read_word_from_array(&self.expansion_1, phys_addr - Memory::EXPANSION_1)
      },
      (Memory::SCRATCHPAD..=Memory::SCRATCHPAD_END) => {
        read_word_from_array(&self.scratchpad, phys_addr - Memory::SCRATCHPAD)
      },
      (Memory::IO_PORTS..=Memory::IO_PORTS_END) => {
        read_word_from_array(&self.io_ports, phys_addr - Memory::IO_PORTS)
      },
      (Memory::EXPANSION_2..=Memory::EXPANSION_2_END) => {
        read_word_from_array(&self.expansion_2, phys_addr - Memory::EXPANSION_2)
      },
      (Memory::EXPANSION_3..=Memory::EXPANSION_3_END) => {
        read_word_from_array(&self.expansion_3, phys_addr - Memory::EXPANSION_3)
      },
      (Memory::BIOS..=Memory::BIOS_END) => {
        read_word_from_array(&*self.bios, phys_addr - Memory::BIOS)
      },
      (Memory::CACHE_CONTROL..=Memory::CACHE_CONTROL_END) => {
        read_word_from_array(&self.cache_control, phys_addr - Memory::CACHE_CONTROL)
      },
      _ => {
        panic!("tried to access an unmapped section of memory at {}", phys_addr)
      },
    })
  }
  pub fn write_word(&mut self, address: &Register, value: &Register) {
    let phys_addr = address.get_value() & 0x1fff_ffff;
    let value = value.get_value();
    match phys_addr {
      (Memory::MAIN_RAM..=Memory::MAIN_RAM_END) => {
        write_word_to_array(&mut self.main_ram, phys_addr - Memory::MAIN_RAM, value);
      },
      (Memory::EXPANSION_1..=Memory::EXPANSION_1_END) => {
        write_word_to_array(&mut self.expansion_1, phys_addr - Memory::EXPANSION_1, value);
      },
      (Memory::SCRATCHPAD..=Memory::SCRATCHPAD_END) => {
        write_word_to_array(&mut self.scratchpad, phys_addr - Memory::SCRATCHPAD, value);
      },
      (Memory::IO_PORTS..=Memory::IO_PORTS_END) => {
        write_word_to_array(&mut self.io_ports, phys_addr - Memory::IO_PORTS, value);
      },
      (Memory::EXPANSION_2..=Memory::EXPANSION_2_END) => {
        write_word_to_array(&mut self.expansion_2, phys_addr - Memory::EXPANSION_2, value);
      },
      (Memory::EXPANSION_3..=Memory::EXPANSION_3_END) => {
        write_word_to_array(&mut self.expansion_3, phys_addr - Memory::EXPANSION_3, value);
      },
      (Memory::BIOS..=Memory::BIOS_END) => {
        write_word_to_array(&mut *self.bios, phys_addr - Memory::BIOS, value);
      },
      (Memory::CACHE_CONTROL..=Memory::CACHE_CONTROL_END) => {
        write_word_to_array(&mut self.cache_control, phys_addr - Memory::CACHE_CONTROL, value);
      },
      _ => {
        panic!("tried to access an unmapped section of memory at {}", phys_addr)
      },
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  //check first instruction in this BIOS file
  fn scph1001_first_instr() {
    let bios = "/home/ayrton/dev/rps/scph1001.bin".to_string();
    let mem = Memory::new(&bios).unwrap();
    assert_eq!(mem.read_word(0xbfc0_0000), 0x3c08_0013);
  }

  #[test]
  #[should_panic]
  fn unmapped_read_panics() {
    let mem = Memory::blank();
    mem.read_word(Memory::BIOS_END);
  }

  #[test]
  fn memory_is_modified() {
    let mut mem = Memory::blank();
    mem.write_word(Memory::MAIN_RAM + 5, 10);
    assert_eq!(mem.read_word(Memory::MAIN_RAM + 5), 10);
  }
}
