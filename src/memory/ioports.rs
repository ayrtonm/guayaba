use crate::common::ReadArray;
use crate::memory::Memory;
use crate::memory::MemAction;
use crate::memory::MemResponse;
use crate::register::Register;
use crate::register::Aliases;
use crate::dma::Transfer;
use crate::dma::Chunks;
use crate::dma::Blocks;
use crate::dma::Direction;
use crate::dma::Step;

#[macro_export]
macro_rules! get_io_response {
  ($address:expr, $function:ident, $self:expr) => {
    match $address {
      0x1f80_1810..=0x1f80_1813 => {
        MemResponse::GPUREAD
      },
      0x1f80_1814..=0x1f80_1817 => {
        MemResponse::GPUSTAT
      },
      _ => {
        MemResponse::Value($self.io_ports.as_ref().$function($address - Memory::IO_PORTS))
      },
    }
  };
}

#[macro_export]
macro_rules! get_io_action {
  ($address:expr, $value:expr, $function:ident, $self:expr) => {
    match $address {
      //GPU registers
      0x1f80_1810..=0x1f80_1813 => {
        Some(
          MemAction::GpuGp0(
            $self.io_ports.as_ref().read_word(
              0x1f80_1810 - Memory::IO_PORTS)))
      },
      0x1f80_1814..=0x1f80_1817 => {
        Some(
          MemAction::GpuGp1(
            $self.io_ports.as_ref().read_word(
              0x1f80_1814 - Memory::IO_PORTS)))
      },
      //DMA registers
      0x1f80_1080..=0x1f80_10f3 => {
        None
      },
      //DMA interrupt register
      0x1f80_10f4..=0x1f80_10f7 => {
        None
      },
      _ => {
        None
      },
    }
  };
}

impl Memory {
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
    let sync_mode = (self.io_ports.as_ref().read_word(channel_control) >> 9) & 3;
    let direction = match self.io_ports.as_ref().read_word(channel_control).nth_bit_bool(0) {
      false => Direction::ToRAM,
      true => Direction::FromRAM,
    };
    let step = match self.io_ports.as_ref().read_word(channel_control).nth_bit_bool(1) {
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
