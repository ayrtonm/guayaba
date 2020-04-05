use std::collections::VecDeque;
use crate::memory::MB;
use crate::register::Register;

struct Command {
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
      _ => {
        todo!("implement this")
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
  vram: Box<[u8]>,
  command_buffer: VecDeque<Command>,
  waiting_for_parameters: bool,
  partial_command: Option<Command>,
}

impl GPU {
  pub fn new() -> Self {
    let command_buffer = VecDeque::new();
    GPU {
      vram: vec![0; MB].into_boxed_slice(),
      command_buffer,
      waiting_for_parameters: false,
      partial_command: None,
    }
  }
  pub fn write_to_gp0(&mut self, value: Register) {
    if self.waiting_for_parameters {
      let mut cmd = self.partial_command.take().unwrap();
      cmd.append_parameters(value);
      if cmd.completed() {
        self.command_buffer.push_back(cmd);
      } else {
        self.partial_command = Some(cmd);
      }
    } else {
      let cmd = Command::new(value);
      if cmd.completed() {
        self.command_buffer.push_back(cmd);
      } else {
        self.partial_command = Some(cmd);
        self.waiting_for_parameters = true;
      }
    }
  }
}
