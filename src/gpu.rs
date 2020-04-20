use std::collections::VecDeque;
use crate::memory::MB;
use crate::register::Register;
use crate::register::BitManipulation;
use crate::dma::DMAChannel;

#[derive(Debug)]
pub struct Command {
  id: u8,
  parameters: Vec<u8>,
}

impl Command {
  pub fn new(cmd: u32) -> Self {
    let param1 = ((cmd >> 16) & 0x0000_00ff) as u8;
    let param2 = ((cmd >> 8) & 0x0000_00ff) as u8;
    let param3 = (cmd & 0x0000_00ff) as u8;
    Command {
      id: (cmd >> 24) as u8,
      parameters: vec![param1, param2, param3],
    }
  }
  pub fn serialize(&self) -> Register {
    assert!(self.parameters.len() == 3);
    ((self.id as Register) << 24) |
    ((self.parameters[0] as Register) << 16) |
    ((self.parameters[1] as Register) << 8) |
    (self.parameters[2] as Register)
  }
  pub fn id(&self) -> u8 {
    self.id
  }
  pub fn append_parameters(&mut self, param: u32) {
    let param1 = (param >> 24) as u8;
    let param2 = ((param >> 16) & 0x0000_00ff) as u8;
    let param3 = ((param >> 8) & 0x0000_00ff) as u8;
    let param4 = (param & 0x0000_00ff) as u8;
    self.parameters.push(param1);
    self.parameters.push(param2);
    self.parameters.push(param3);
    self.parameters.push(param4);
  }
  pub fn completed(&self) -> bool {
    match self.id {
      0x20 | 0x22 => {
        self.parameters.len() == 15
      },
      0x28 | 0x2a => {
        self.parameters.len() == 19
      },
      0x24 | 0x25 | 0x26 | 0x27 => {
        self.parameters.len() == 27
      },
      0x2C | 0x2D | 0x2E | 0x2F => {
        self.parameters.len() == 35
      },
      0x30 | 0x32 => {
        self.parameters.len() == 23
      },
      0x38 | 0x3a => {
        self.parameters.len() == 31
      },
      0x34 | 0x36 => {
        self.parameters.len() == 35
      },
      0x3c | 0x3e => {
        self.parameters.len() == 47
      },
      0x40 | 0x42 => {
        self.parameters.len() == 11
      },
      0x48 | 0x4a | 0x58 | 0x5a => {
        (self.parameters.len() >= 15) &&
        self.parameters.iter()
                       .rev()
                       .take(4)
                       .all(|&p| p == 0x55)
      },
      0x60 | 0x62 => {
        self.parameters.len() == 11
      },
      0x68 | 0x6a | 0x70 | 0x72 | 0x78 | 0x7a => {
        self.parameters.len() == 7
      },
      0x64 | 0x65 | 0x66 | 0x67 => {
        self.parameters.len() == 15
      },
      0x6c | 0x6d | 0x6e | 0x6f |
      0x74 | 0x75 | 0x76 | 0x77 |
      0x7c | 0x7d | 0x7e | 0x7f => {
        self.parameters.len() == 11
      },
      0xe1 | 0xe2 | 0xe3 | 0xe4 | 0xe5 | 0xe6 | 0x01 => {
        self.parameters.len() == 3
      },
      0x02 => {
        self.parameters.len() == 11
      },
      0x80..=0x9f => {
        self.parameters.len() == 15
      },
      0xa0..=0xbf => {
        //this is some function of the parameters
        if self.parameters.len() < 11 {
          false
        } else {
          //xsize and ysize are measured in halfwords
          let xsize = (self.parameters[10] as u32) + ((self.parameters[9] as u32) << 8);
          let ysize = (self.parameters[8] as u32) + ((self.parameters[7] as u32) << 8);
          //paramter length is in bytes
          let num_words = ((xsize as u64) * (ysize as u64)) << 1;
          self.parameters.len() >= num_words as usize
        }
      },
      0xc0..=0xdf => {
        //this is some function of the parameters
        todo!("implement this GPU command {:x}", self.id)
      },
      0x1f => {
        self.parameters.len() == 3
      },
      0x00 | 04..=0x1e | 0xe0 | 0xe7..=0xef => {
        true
      },
      0x50 | 0x52 => {
        self.parameters.len() == 15
      },
      _ => {
        todo!("implement this GPU command {:x}", self.id)
      },
    }
  }
}

trait Size {
  fn num_bytes(&self) -> usize;
}

impl Size for Command {
  fn num_bytes(&self) -> usize {
    1 + self.parameters.len()
  }
}

impl Size for VecDeque<Command> {
  fn num_bytes(&self) -> usize {
    self.iter().fold(0, |acc, cmd| acc + cmd.num_bytes())
  }
}

pub struct GPU {
  gpustat: GPUStatus,
  gpuread: Register,
  vram: Box<[u8]>,
  command_buffer: VecDeque<Command>,
  waiting_for_parameters: bool,
  partial_command: Option<Command>,
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
  fn texture_page_x(&self) -> Register {
    self.0 & 0x0f
  }
  fn set_texture_page_x(&mut self, value: Register) {
    assert!(value < 0x10);
    self.0 = (self.0 & 0xffff_fff0) | value;
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
          0xe1 => {
            let mask = 0x0000_83ff;
            let command = command.serialize() & mask;
            self.gpustat.as_mut().clear_mask(mask).set_mask(command);
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
}
