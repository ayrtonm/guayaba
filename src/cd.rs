use std::io;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Read;
use std::fs::File;
use std::collections::VecDeque;
use crate::register::Register;
use crate::dma::DMAChannel;

pub struct CD {
  contents: Option<Box<[u8]>>,
  command_buffer: VecDeque<u8>,
  parameter_buffer: VecDeque<u8>,
}

impl CD {
  pub fn new(filename: Option<&String>) -> Self {
    let contents = filename.map(
      |filename| {
        let mut buffer = Vec::new();
        //TODO: do proper error handling here
        let mut file = File::open(filename).unwrap();
        file.seek(SeekFrom::Start(0));
        file.read_to_end(&mut buffer);
        buffer.into_boxed_slice()
      });
    CD {
      contents,
      command_buffer: VecDeque::new(),
      parameter_buffer: VecDeque::new(),
    }
  }
  pub fn send_parameter(&mut self, val: u8) {
    println!("CD received parameter {:#x}", val);
    self.parameter_buffer.push_back(val);
  }
  pub fn send_command(&mut self, cmd: u8) {
    println!("CD received command {:#x}", cmd);
    self.command_buffer.push_back(cmd);
  }
  pub fn exec_command(&mut self) {
    self.command_buffer.pop_front().map(|cmd| {
      match cmd {
        _ => {
          println!("CD executed {:#x}", cmd);
        },
      }
    });
  }
}

impl DMAChannel for CD {
  fn send(&mut self, data: Vec<Register>) {
    todo!("implement DMAChannel for CD")
  }
  fn receive(&self) -> Register {
    todo!("implement DMAChannel for CD")
  }
}
