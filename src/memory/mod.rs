use std::io;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Read;
use std::fs::metadata;
use std::fs::File;
use crate::common::ReadArray;
use crate::common::WriteArray;
use crate::register::Register;
use crate::register::BitBang;
use crate::dma::Transfer;
use crate::dma::DMAChannel;

#[macro_use]
mod ioports;
use ioports::DMAControl;

pub enum MemAction {
  DMA(Transfer),
  GpuGp0(Register),
  GpuGp1(Register),
}

pub enum MemResponse {
  Value(Register),
  GPUREAD,
  GPUSTAT,
}

pub const KB: usize = 1024;
pub const MB: usize = 1024 * KB;
const PHYS_MASK: [u32; 8] = [0xffff_ffff, 0xffff_ffff, 0xffff_ffff, 0xffff_ffff,
                             0x7fff_ffff, 0x1fff_ffff, 0xffff_ffff, 0xffff_ffff];
macro_rules! read_memory {
  ($address:expr, $function:ident, $self:expr) => {
    {
      let idx = ($address >> 29) as usize;
      let phys_addr = $address & PHYS_MASK[idx];
      match phys_addr {
        (Memory::MAIN_RAM..=Memory::MAIN_RAM_END) => {
          MemResponse::Value($self.main_ram.as_ref().$function(phys_addr - Memory::MAIN_RAM))
        },
        (Memory::EXPANSION_1..=Memory::EXPANSION_1_END) => {
          MemResponse::Value($self.expansion_1.as_ref().$function(phys_addr - Memory::EXPANSION_1))
        },
        (Memory::SCRATCHPAD..=Memory::SCRATCHPAD_END) => {
          MemResponse::Value($self.scratchpad.as_ref().$function(phys_addr - Memory::SCRATCHPAD))
        },
        (Memory::IO_PORTS..=Memory::IO_PORTS_END) => {
          get_io_response!(phys_addr, $function, $self)
        },
        (Memory::EXPANSION_2..=Memory::EXPANSION_2_END) => {
          MemResponse::Value($self.expansion_2.as_ref().$function(phys_addr - Memory::EXPANSION_2))
        },
        (Memory::EXPANSION_3..=Memory::EXPANSION_3_END) => {
          MemResponse::Value($self.expansion_3.as_ref().$function(phys_addr - Memory::EXPANSION_3))
        },
        (Memory::BIOS..=Memory::BIOS_END) => {
          MemResponse::Value($self.bios.as_ref().as_ref().$function(phys_addr - Memory::BIOS))
        },
        (Memory::CACHE_CONTROL..=Memory::CACHE_CONTROL_END) => {
          MemResponse::Value($self.cache_control.as_ref().$function($address - Memory::CACHE_CONTROL))
        },
        _ => {
          panic!("{} [{:#x}] = [{:#x}] is illegal", stringify!($function), $address, phys_addr);
        },
      }
    }
  };
}

macro_rules! write_memory {
  ($address:expr, $value:expr, $function:ident, $self:expr) => {
    {
      let idx = ($address >> 29) as usize;
      let phys_addr = $address & PHYS_MASK[idx];
      match phys_addr {
        (Memory::MAIN_RAM..=Memory::MAIN_RAM_END) => {
          $self.main_ram.as_mut().$function(phys_addr - Memory::MAIN_RAM, $value);
          None
        },
        (Memory::EXPANSION_1..=Memory::EXPANSION_1_END) => {
          $self.expansion_1.as_mut().$function(phys_addr - Memory::EXPANSION_1, $value);
          None
        },
        (Memory::SCRATCHPAD..=Memory::SCRATCHPAD_END) => {
          $self.scratchpad.as_mut().$function(phys_addr - Memory::SCRATCHPAD, $value);
          None
        },
        (Memory::IO_PORTS..=Memory::IO_PORTS_END) => {
          $self.io_ports.as_mut().$function(phys_addr - Memory::IO_PORTS, $value);
          get_io_action!(phys_addr, $value, $function, $self)
        },
        (Memory::EXPANSION_2..=Memory::EXPANSION_2_END) => {
          $self.expansion_2.as_mut().$function(phys_addr - Memory::EXPANSION_2, $value);
          None
        },
        (Memory::EXPANSION_3..=Memory::EXPANSION_3_END) => {
          $self.expansion_3.as_mut().$function(phys_addr - Memory::EXPANSION_3, $value);
          None
        },
        (Memory::BIOS..=Memory::BIOS_END) => {
          $self.bios.as_mut().as_mut().$function(phys_addr - Memory::BIOS, $value);
          None
        },
        (Memory::CACHE_CONTROL..=Memory::CACHE_CONTROL_END) => {
          $self.cache_control.as_mut().$function($address - Memory::CACHE_CONTROL, $value);
          None
        },
        _ => {
          panic!("{} [{:#x}] = [{:#x}] = {:#x} is illegal", stringify!($function), $address, phys_addr, $value);
        },
      }
    }
  };
}

pub struct Memory {
  //these correspond to physical memory locations
  main_ram: Box<[u8]>,
  expansion_1: Box<[u8]>,
  scratchpad: [u8; KB],
  io_ports: [u8; 8 * KB],
  expansion_2: [u8; 8 * KB],
  expansion_3: Box<[u8]>,
  bios: Box<[u8; 512 * KB]>,
  cache_control: [u8; 512],
}

impl DMAChannel for Memory {
  fn send(&mut self, data: Vec<Register>) {
    todo!("implement DMAChannel for Memory")
  }
  fn receive(&self) -> Register {
    todo!("implement DMAChannel for Memory")
  }
}

impl Memory {
  pub fn new(bios_filename: &String) -> io::Result<Self> {
    let mut bios_contents = [0; 512 * KB];
    let mut bios_file = File::open(bios_filename)?;
    let filesize = metadata(bios_filename)?.len();
    assert_eq!(filesize, 512 * KB as u64, "Invalid BIOS file size");
    bios_file.seek(SeekFrom::Start(0))?;
    bios_file.read_exact(&mut bios_contents)?;
    let bios = Box::new(bios_contents);
    //initialize I/O ports
    let mut io_ports = [0; 8 * KB];
    io_ports.as_mut().write_word(0x1f8010f0 - Memory::IO_PORTS, 0x0765_4321);
    Ok(Memory {
      main_ram: vec![0; 2 * MB].into_boxed_slice(),
      expansion_1: vec![0; 8 * MB].into_boxed_slice(),
      scratchpad: [0; KB],
      io_ports,
      expansion_2: [0; 8 * KB],
      expansion_3: vec![0; 2 * MB].into_boxed_slice(),
      bios,
      cache_control: [0; 512],
    })
  }
  const MAIN_RAM: Register = 0;
  const MAIN_RAM_END: Register = Memory::MAIN_RAM + (2 * MB as Register) - 1;

  const EXPANSION_1: Register = 0x1f00_0000;
  const EXPANSION_1_END: Register = Memory::EXPANSION_1 + (8 * MB as Register) - 1;

  const SCRATCHPAD: Register = 0x1f80_0000;
  const SCRATCHPAD_END: Register = Memory::SCRATCHPAD + (KB as Register) - 1;

  const IO_PORTS: Register = 0x1f80_1000;
  const IO_PORTS_END: Register = Memory::IO_PORTS + (8 * KB as Register) - 1;
 
  const EXPANSION_2: Register = 0x1f80_2000;
  const EXPANSION_2_END: Register = Memory::EXPANSION_2 + (8 * KB as Register) - 1;

  const EXPANSION_3: Register = 0x1fa0_0000;
  const EXPANSION_3_END: Register = Memory::EXPANSION_3 + (2 * MB as Register) - 1;

  const BIOS: Register = 0x1fc0_0000;
  const BIOS_END: Register = Memory::BIOS + (512 * KB as Register) - 1;

  const CACHE_CONTROL: Register = 0xfffe_0000;
  const CACHE_CONTROL_END: Register = Memory::CACHE_CONTROL + 512 - 1;

  //FIXME: fix alignment restrictions, what happens when read is misaligned?
  //TODO: technically this doesn't sign extend the GPU response
  pub fn read_byte_sign_extended(&self, address: Register) -> MemResponse {
    match read_memory!(address, read_byte, self) {
      MemResponse::Value(value) => {
        MemResponse::Value(value.byte_sign_extended())
      },
      MemResponse::GPUREAD => {
        MemResponse::GPUREAD
      },
      MemResponse::GPUSTAT => {
        MemResponse::GPUSTAT
      },
    }
  }
  pub fn read_half_sign_extended(&self, address: Register) -> MemResponse {
    assert_eq!(address & 0x0000_0001, 0);
    match read_memory!(address, read_half, self) {
      MemResponse::Value(value) => {
        MemResponse::Value(value.half_sign_extended())
      },
      MemResponse::GPUREAD => {
        MemResponse::GPUREAD
      },
      MemResponse::GPUSTAT => {
        MemResponse::GPUSTAT
      },
    }
  }
  pub fn read_byte(&self, address: Register) -> MemResponse {
    read_memory!(address, read_byte, self)
  }
  pub fn read_half(&self, address: Register) -> MemResponse {
    assert_eq!(address & 0x0000_0001, 0);
    read_memory!(address, read_half, self)
  }
  pub fn read_word(&self, address: Register) -> MemResponse {
    assert_eq!(address & 0x0000_0003, 0);
    read_memory!(address, read_word, self)
  }
  pub fn write_byte(&mut self, address: Register, value: Register) -> Option<MemAction> {
    write_memory!(address, value, write_byte, self)
  }
  pub fn write_half(&mut self, address: Register, value: Register) -> Option<MemAction> {
    assert_eq!(address & 0x0000_0001, 0);
    write_memory!(address, value, write_half, self)
  }
  pub fn write_word(&mut self, address: Register, value: Register) -> Option<MemAction>  {
    assert_eq!(address & 0x0000_0003, 0);
    write_memory!(address, value, write_word, self)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  impl Memory {
    pub fn blank() -> Memory {
      Memory {
        main_ram: vec![0; 2 * MB].into_boxed_slice(),
        expansion_1: vec![0; 8 * MB].into_boxed_slice(),
        scratchpad: [0; KB],
        io_ports: [0; 8 * KB],
        expansion_2: [0; 8 * KB],
        expansion_3: vec![0; 2 * MB].into_boxed_slice(),
        bios: Box::new([0; 512 * KB]),
        cache_control: [0; 512],
      }
    }
  }
  #[test]
  //check first instruction in this BIOS file
  fn scph1001_first_instr() {
    let bios = "/home/ayrton/dev/rspsx/scph1001.bin".to_string();
    let mem = Memory::new(&bios).unwrap();
    let initial_pc = 0xbfc0_0000;
    assert_eq!(mem.read_word(initial_pc), 0x3c08_0013);
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
    let address = Memory::MAIN_RAM + 4;
    let value = 10;
    mem.write_word(address, value);
    assert_eq!(mem.read_word(address), value);
  }

  #[test]
  #[should_panic]
  fn unaligned_write_paincs() {
    let mut mem = Memory::blank();
    let address = Memory::MAIN_RAM + 5;
    let value = 10;
    mem.write_word(address, value);
  }
}
