use crate::register::Register;

pub enum Chunks {
  NumWords(u32),
  Blocks(Blocks),
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

pub struct Transfer {
  channel: u32,
  start_address: Register,
  chunks: Chunks,
  direction: Direction,
  step: Step,
  sync_mode: u32,
}

impl Transfer {
  pub fn new(channel: u32, start_address: Register, chunks: Chunks,
             direction: Direction, step: Step, sync_mode: u32) -> Self {
    Transfer {
      channel,
      start_address,
      chunks,
      direction,
      step,
      sync_mode,
    }
  }
  pub fn channel(&self) -> u32 {
    self.channel
  }
  pub fn direction(&self) -> &Direction {
    &self.direction
  }
}
