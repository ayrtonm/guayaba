use crate::register::Register;

pub trait DMAChannel {
  fn send(&mut self, data: Vec<Register>);
  fn receive(&self) -> Register;
}

#[derive(Debug)]
pub enum Chunks {
  NumWords(u32),
  Blocks(Blocks),
  LinkedList,
}

#[derive(Debug)]
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
  pub fn block_size(&self) -> u16 {
    self.block_size
  }
  pub fn num_blocks(&self) -> u16 {
    self.num_blocks
  }
}

#[derive(Debug)]
pub enum Direction {
  ToRAM,
  FromRAM,
}

#[derive(Debug)]
pub enum Step {
  Forward,
  Backward,
}

#[derive(Debug)]
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
  pub fn step(&self) -> &Step {
    &self.step
  }
  pub fn start_address(&self) -> u32 {
    self.start_address
  }
  pub fn channel(&self) -> u32 {
    self.channel
  }
  pub fn direction(&self) -> &Direction {
    &self.direction
  }
  pub fn chunks(&self) -> &Chunks {
    &self.chunks
  }
}
