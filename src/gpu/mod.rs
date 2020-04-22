use std::collections::VecDeque;
use crate::memory::MB;
use crate::register::Register;
use crate::register::BitManipulation;
use crate::dma::DMAChannel;

mod command;
use command::Command;

pub struct GPU {
  gpustat: GPUStatus,
  gpuread: Register,
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
    self.gpuread
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
  pub fn new() -> Self {
    let command_buffer = VecDeque::new();
    GPU {
      gpustat: GPUStatus::new(),
      gpuread: 0,
      vram: vec![0; MB].into_boxed_slice(),
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
    self.gpustat.0
  }
  pub fn exec_next_gp0_command(&mut self) {
    let command = self.command_buffer.pop_front();
    match command {
      Some(command) => {
        match command.id() {
          0x00 => {
          },
          0x04..=0x1e | 0xe0 | 0xe7..=0xef => {
          },
          0xe1 => {
            let mask = 0x0000_83ff;
            let command = command.serialize() & mask;
            self.gpustat.as_mut().clear_mask(mask).set_mask(command);
          },
          0xe2 => {
            let command = command.serialize();
            self.texture_mask_x = command.lowest_bits(5);
            self.texture_mask_y = (command >> 5).lowest_bits(5);
            self.texture_offset_x = (command >> 10).lowest_bits(5);
            self.texture_offset_y = (command >> 15).lowest_bits(5);
          },
          0xe3 => {
            let command = command.serialize();
            self.drawing_min_x = command.lowest_bits(10);
            self.drawing_min_y = (command >> 10).lowest_bits(9);
          },
          0xe4 => {
            let command = command.serialize();
            self.drawing_max_x = command.lowest_bits(10);
            self.drawing_max_y = (command >> 10).lowest_bits(9);
          },
          0xe5 => {
            let command = command.serialize();
            self.drawing_offset_x = command.lowest_bits(11);
            self.drawing_offset_y = (command >> 11).lowest_bits(11);
          },
          0xe6 => {
            let command = command.serialize();
            let mask = command.lowest_bits(2) << 11;
            self.gpustat.as_mut().clear(11).clear(12).set_mask(mask);
          },
          _ => {
            todo!("implement this GP0 command {:#x}", command.id());
          },
        }
      },
      None => {
      },
    }
  }
  pub fn write_to_gp0(&mut self, value: Register) {
    //println!("GP0 received {:#x}", value);
    if !self.waiting_for_parameters {
      let cmd = Command::new(value);
      if cmd.completed() {
        println!("GP0 received command {:#x?}", cmd);
        self.command_buffer.push_back(cmd);
      } else {
        self.partial_command = Some(cmd);
        self.waiting_for_parameters = true;
      }
    } else {
      let mut cmd = self.partial_command.take().expect("Expected a partial command in the GPU");
      cmd.append_parameters(value);
      if cmd.completed() {
        println!("GP0 received command {:#x?}", cmd);
        self.command_buffer.push_back(cmd);
        self.waiting_for_parameters = false;
      } else {
        self.partial_command = Some(cmd);
      }
    }
  }
  pub fn write_to_gp1(&mut self, value: Register) {
    println!("GP1 received {:#x}", value);
    let command = value >> 24;
    match command {
      0x00 => {
        *self.gpustat.as_mut() = 0x1480_2000;
      },
      0x04 => {
        let mask = 0x6000_0000;
        let new_values = (value & 3) << 29;
        self.gpustat.as_mut().clear_mask(mask).set_mask(new_values);
      },
      0x05 => {
        self.display_x = command.lowest_bits(10);
        self.display_y = (command >> 10).lowest_bits(9);
      },
      0x06 => {
        self.display_range_x1 = command.lowest_bits(12);
        self.display_range_x2 = (command >> 12).lowest_bits(12);
      },
      0x07 => {
        self.display_range_y1 = command.lowest_bits(10);
        self.display_range_y2 = (command >> 10).lowest_bits(10);
      },
      0x08 => {
        let mask = 0x003f_4000;
        let new_values = ((value & 0x3f) << 17) | (value & 0x40) << 16 | (value & 0x80) << 14;
        self.gpustat.as_mut().clear_mask(mask).set_mask(new_values);
      },
      _ => {
        todo!("implement this GP1 command {:#x}", command);
      },
    }
  }
  fn num_bytes(&self) -> usize {
    self.command_buffer.iter().fold(0, |acc, cmd| acc + cmd.num_bytes())
  }
}
