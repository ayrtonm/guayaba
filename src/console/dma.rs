pub trait DMAChannel {
  fn send(&mut self, data: Vec<u32>);
  fn receive(&self) -> u32;
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
//1 + 4 + 1 + 1 + 4 + 1
pub struct Transfer {
  channel_num: u8,
  start_address: u32,
  chunks: Chunks,
  direction: Direction,
  step: Step,
  sync_mode: u8,
}

impl Transfer {
  pub fn new(channel_num: u8, start_address: u32, chunks: Chunks,
             direction: Direction, step: Step, sync_mode: u8) -> Self {
    Transfer {
      channel_num,
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
  pub fn channel_num(&self) -> u8 {
    self.channel_num
  }
  pub fn direction(&self) -> &Direction {
    &self.direction
  }
  pub fn chunks(&self) -> &Chunks {
    &self.chunks
  }
}
