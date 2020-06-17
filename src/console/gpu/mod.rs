use std::collections::VecDeque;
use super::memory::MB;
use crate::register::BitTwiddle;
use super::dma::DMAChannel;

mod command;
mod gp0;
mod gp1;
use command::Command;

pub struct GPU {
  logging: bool,
  gpustat: GPUStatus,
  gpuread: VecDeque<u32>,
  vram: Box<[u8]>,
  command_buffer: VecDeque<Command>,
  waiting_for_parameters: bool,
  partial_command: Option<Command>,
  drawing_min_x: u32,
  drawing_min_y: u32,
  drawing_max_x: u32,
  drawing_max_y: u32,
  drawing_offset_x: u32,
  drawing_offset_y: u32,
  texture_mask_x: u32,
  texture_mask_y: u32,
  texture_offset_x: u32,
  texture_offset_y: u32,
  display_x: u32,
  display_y: u32,
  display_range_x1: u32,
  display_range_x2: u32,
  display_range_y1: u32,
  display_range_y2: u32,
}

impl DMAChannel for GPU {
  fn send(&mut self, data: Vec<u32>) {
    data.iter().for_each(|&word| self.write_to_gp0(word))
  }
  fn receive(&self) -> u32 {
    //self.gpuread
    0
  }
}

struct GPUStatus(u32);

impl GPUStatus {
  fn new() -> Self {
    GPUStatus(0x1c00_0000)
  }
  fn as_mut(&mut self) -> &mut u32 {
    &mut self.0
  }
  //this may be useful for when the GPU (i.e. OpengGL part) needs to access the info in gpustat
  fn texture_page_x(&self) -> u32 {
    self.0.lowest_bits(4)
  }
}

impl GPU {
  pub fn new(logging: bool) -> Self {
    let command_buffer = VecDeque::new();
    GPU {
      logging,
      gpustat: GPUStatus::new(),
      gpuread: VecDeque::new(),
      //512 lines of 2048 bytes
      vram: vec![0; 1 * MB].into_boxed_slice(),
      command_buffer,
      waiting_for_parameters: false,
      partial_command: None,
      //display settings
      drawing_min_x: 0,
      drawing_min_y: 0,
      drawing_max_x: 0,
      drawing_max_y: 0,
      drawing_offset_x: 0,
      drawing_offset_y: 0,
      texture_mask_x: 0,
      texture_mask_y: 0,
      texture_offset_x: 0,
      texture_offset_y: 0,
      display_x: 0,
      display_y: 0,
      display_range_x1: 0,
      display_range_x2: 0,
      display_range_y1: 0,
      display_range_y2: 0,
    }
  }
  pub fn gpustat(&self) -> u32 {
    //this is a dirty hack
    *self.gpustat.0.clone().clear(19).clear(14).clear(31).set(26).set(27).set(28)
  }
  pub fn gpuread(&mut self) -> u32 {
    self.gpuread.pop_front().map_or(0, |value| value)
  }
}
