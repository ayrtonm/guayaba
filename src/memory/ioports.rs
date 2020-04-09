use crate::common::*;
use crate::memory::Memory;
use crate::dma::Transfer;
use crate::dma::Chunks;
use crate::dma::Blocks;
use crate::dma::Direction;
use crate::dma::Step;

impl Memory {
  //pack the current state of I/O ports into a Transfer struct for a given channel
  pub(super) fn create_dma_transfer(&mut self, channel: u32) -> Transfer {
    assert!(channel < 7);
    //these are addresses to locations in memory
    let base_addr = 0x0000_0080 + (channel * 0x0000_0010);
    let block_control = base_addr + 4;
    let channel_control = block_control + 4;
  
    //these are the values of locations in memory
    let start_address = read_word_from_array(&self.io_ports, base_addr) & 0x00ff_fffc;
    let block_control = read_word_from_array(&self.io_ports, block_control);
    let sync_mode = (read_word_from_array(&self.io_ports, channel_control) >> 9) & 3;
    let direction = match read_word_from_array(&self.io_ports, channel_control) & 1 {
      0 => Direction::ToRAM,
      1 => Direction::FromRAM,
      _ => unreachable!(""),
    };
    let step = match read_word_from_array(&self.io_ports, channel_control) & 2 {
      0 => Step::Forward,
      2 => Step::Backward,
      _ => unreachable!(""),
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
