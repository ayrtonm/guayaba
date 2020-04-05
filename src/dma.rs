use crate::register::Register;

pub enum Chunks {
  num_words(u32),
  blocks(Blocks),
  LinkedList,
}

pub struct Blocks {
  block_size: u16,
  num_blocks: u16,
}

impl Blocks {
  pub fn new(block_size: u16, num_blocks: u16) -> Self {
    Blocks {
      block_size,
      num_blocks,
    }
  }
}

pub enum Direction {
  ToRAM,
  FromRAM,
}

pub enum Step {
  Forward,
  Backward,
}

#[derive(Default,Debug)]
pub struct DMA {
}

pub struct Transfer {
  start_address: Register,
  chunks: Chunks,
  direction: Direction,
  step: Step,
  sync_mode: u32,
}

impl Transfer {
  pub fn new(start_address: Register, chunks: Chunks, direction: Direction,
             step: Step, sync_mode: u32) -> Self {
    Transfer {
      start_address,
      chunks,
      direction,
      step,
      sync_mode,
    }
  }
}
