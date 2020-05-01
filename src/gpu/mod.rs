use std::collections::VecDeque;
use crate::memory::MB;
use crate::register::Register;
use crate::register::BitBang;
use crate::dma::DMAChannel;

mod command;
mod gp0;
mod gp1;
use command::Command;

pub struct GPU {
  logging: bool,
  gpustat: GPUStatus,
  gpuread: VecDeque<Register>,
  vram: Box<[u8]>,
  command_buffer: VecDeque<Command>,
  waiting_for_parameters: bool,
  partial_command: Option<Command>,
  drawing_min_x: Register,
  drawing_min_y: Register,
  drawing_max_x: Register,
  drawing_max_y: Register,
  drawing_offset_x: Register,
  drawing_offset_y: Register,
  texture_mask_x: Register,
  texture_mask_y: Register,
  texture_offset_x: Register,
  texture_offset_y: Register,
  display_x: Register,
  display_y: Register,
  display_range_x1: Register,
  display_range_x2: Register,
  display_range_y1: Register,
  display_range_y2: Register,
}

impl DMAChannel for GPU {
  fn send(&mut self, data: Vec<Register>) {
    data.iter().for_each(|&word| self.write_to_gp0(word))
  }
  fn receive(&self) -> Register {
    //self.gpuread
    0
  }
}

struct GPUStatus(Register);

impl GPUStatus {
  fn new() -> Self {
    GPUStatus(0x1c00_0000)
  }
  fn as_mut(&mut self) -> &mut Register {
    &mut self.0
  }
  //this may be useful for when the GPU (i.e. OpengGL part) needs to access the info in gpustat
  fn texture_page_x(&self) -> Register {
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
  pub fn gpustat(&self) -> Register {
    //this is a dirty hack
    *self.gpustat.0.clone().clear(19).clear(14).clear(31).set(26).set(27).set(28)
  }
  pub fn gpuread(&mut self) -> Register {
    self.gpuread.pop_front().map_or(0, |value| value)
  }
}
