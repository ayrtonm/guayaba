use std::io::{Seek, SeekFrom, Read};
use std::fs::File;
use std::collections::VecDeque;
use super::dma::DMAChannel;

pub struct CD {
  contents: Option<Box<[u8]>>,
  command_buffer: VecDeque<u8>,
  parameter_buffer: VecDeque<u8>,
  response_buffer: VecDeque<u8>,
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
      response_buffer: VecDeque::new(),
    }
  }
  pub fn read_response(&mut self) -> u32 {
    self.response_buffer.pop_front()
                        .map_or(0, |response| response as u32)
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
        0x19 => {
          let sub_function = self.parameter_buffer.pop_front();
          match sub_function {
            Some(sub_function) => {
              match sub_function {
                0x20 => {
                  let yy = 0;
                  let mm = 0;
                  let dd = 0;
                  let version = 0;
                  self.response_buffer.push_back(yy);
                  self.response_buffer.push_back(mm);
                  self.response_buffer.push_back(dd);
                  self.response_buffer.push_back(version);
                },
                _ => {
                },
              }
            },
            None => {
              todo!("what happens when there is no sub command");
            },
          }
        },
        _ => {
          println!("CD executed {:#x}", cmd);
        },
      }
    });
  }
}

impl DMAChannel for CD {
  fn send(&mut self, data: Vec<u32>) {
    todo!("implement DMAChannel for CD")
  }
  fn receive(&self) -> u32 {
    todo!("implement DMAChannel for CD")
  }
}
