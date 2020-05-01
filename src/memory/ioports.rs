use crate::common::ReadArray;
use crate::common::WriteArray;
use crate::memory::Memory;
use crate::register::Register;
use crate::register::BitBang;
use crate::dma::Transfer;
use crate::dma::Chunks;
use crate::dma::Blocks;
use crate::dma::Direction;
use crate::dma::Step;

#[macro_export]
macro_rules! get_io_response {
  ($address:expr, $function:ident, $self:expr) => {
    {
      let aligned_address = $address & 0xffff_fffc;
      match aligned_address {
        //CD registers
        Memory::CD_PORT => {
          let offset = $address - Memory::IO_PORTS;
          let value = $self.io_ports.as_ref().$function(offset);
          println!("CD read {:#x} from {:#x}", value, $address);
          MemResponse::Value(value)
        },
        //GPU registers
        Memory::GPU_GP0 => {
          MemResponse::GPUREAD
        },
        Memory::GPU_GP1 => {
          MemResponse::GPUSTAT
        },
        _ => {
          let offset = $address - Memory::IO_PORTS;
          MemResponse::Value($self.io_ports.as_ref().$function(offset))
        },
      }
    }
  };
}

#[macro_export]
macro_rules! get_io_action {
  ($address:expr, $value:expr, $function:ident, $self:expr) => {
    {
      let aligned_address = $address & 0xffff_fffc;
      let aligned_offset = aligned_address - Memory::IO_PORTS;
      match aligned_address {
        //CD registers
        Memory::CD_PORT => {
          println!("CD wrote {:#x} to {:#x}", $value, $address);
          None
        },
        //GPU registers
        Memory::GPU_GP0 => {
          Some(
            MemAction::GpuGp0(
              $self.io_ports.as_ref()
                            .read_word(aligned_offset)))
        },
        Memory::GPU_GP1 => {
          Some(
            MemAction::GpuGp1(
              $self.io_ports.as_ref()
                            .read_word(aligned_offset)))
        },
        //DMA channel controls
        Memory::DMA_CHANNEL_0 |
        Memory::DMA_CHANNEL_1 |
        Memory::DMA_CHANNEL_2 |
        Memory::DMA_CHANNEL_3 |
        Memory::DMA_CHANNEL_4 |
        Memory::DMA_CHANNEL_5 |
        Memory::DMA_CHANNEL_6 => {
          let channel_num = (aligned_address - Memory::DMA_CHANNEL_0) >> 4;
          let control_register = $self.io_ports.as_ref()
                                               .read_word(aligned_offset);
          let sync_mode = control_register.sync_mode();
          if control_register.nth_bit_bool(24) {
            match sync_mode {
              0 => {
                if control_register.nth_bit_bool(28) {
                  Some(MemAction::DMA($self.create_dma_transfer(channel_num)))
                } else {
                  None
                }
              },
              1 | 2 => {
                Some(MemAction::DMA($self.create_dma_transfer(channel_num)))
              },
              _ => unreachable!("DMA channel {} is not configured properly", channel_num),
            }
          } else {
            None
          }
        },
        //DMA interrupt register
        0x1f80_10f4 => {
          None
        },
        _ => {
          None
        },
      }
    }
  };
}

impl Memory {
  pub fn reset_dma_channel(&mut self, channel: u32) {
    let address = Memory::DMA_CHANNEL_0 + (channel * 0x10) - Memory::IO_PORTS;
    let mut control_register = self.io_ports.as_ref().read_word(address);
    let new_register = *control_register.clear(28).clear(24);
    self.io_ports.as_mut().write_word(address, new_register);
  }
  //pack the current state of I/O ports into a Transfer struct for a given channel
  pub(super) fn create_dma_transfer(&mut self, channel: u32) -> Transfer {
    assert!(channel < 7);
    //these are addresses to locations in memory
    let base_addr = 0x0000_0080 + (channel * 0x0000_0010);
    let block_control = base_addr + 4;
    let channel_control = block_control + 4;
  
    //these are the values of locations in memory
    let start_address = self.io_ports.as_ref().read_word(base_addr) & 0x00ff_fffc;
    let block_control = self.io_ports.as_ref().read_word(block_control);
    let control_register = self.io_ports.as_ref().read_word(channel_control);
    let sync_mode = control_register.sync_mode();
    let direction = match control_register.nth_bit_bool(0) {
      false => Direction::ToRAM,
      true => Direction::FromRAM,
    };
    let step = match control_register.nth_bit_bool(1) {
      false => Step::Forward,
      true => Step::Backward,
    };
    let chunks = match sync_mode {
      0 => {
        let words = block_control & 0x0000_ffff;
        Chunks::NumWords(match words {
          0 => 0x0001_0000,
          _ => words,
        })
      },
      1 => {
        let size = block_control & 0x0000_ffff;
        let amount = block_control >> 16;
        let max_size = match channel {
          0 => 0x20,
          1 => 0x20,
          2 => 0x10,
          4 => 0x10,
          _ => unreachable!("DMA channel {} is not configured properly", channel),
        };
        Chunks::Blocks(
          Blocks::new(
            if size < max_size {
              size
            } else {
              max_size
            } as u16,
            amount as u16
          )
        )
      },
      2 => Chunks::LinkedList,
      3 => unreachable!("DMA channel {} is not configured properly", channel),
      _ => unreachable!("DMA channel {} is not configured properly", channel),
    };
    Transfer::new(channel, start_address, chunks, direction, step, sync_mode)
  }
}

pub trait DMAControl {
  fn sync_mode(&self) -> u32;
}

impl DMAControl for Register {
  fn sync_mode(&self) -> u32 {
    (self >> 9) & (3 as u32)
  }
}
