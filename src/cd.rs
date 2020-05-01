use std::io;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Read;
use std::fs::File;
use std::collections::VecDeque;
use crate::register::Register;
use crate::dma::DMAChannel;

pub struct CD {
  contents: Box<Vec<u8>>,
  command_buffer: VecDeque<u8>,
}

impl CD {
  pub fn new(filename: &String) -> io::Result<Self> {
    let mut buffer = Vec::new();
    let mut file = File::open(filename)?;
    file.seek(SeekFrom::Start(0))?;
    file.read_to_end(&mut buffer)?;
    Ok(CD {
      contents: Box::new(buffer),
      command_buffer: VecDeque::new(),
    })
  }
  pub fn send_command(&mut self, cmd: u8) {
    println!("CD received {:#x}", cmd);
    self.command_buffer.push_back(cmd);
  }
  pub fn exec_command(&mut self) {
    match self.command_buffer.pop_front() {
      Some(cmd) => {
        println!("{:#x}", cmd);
      },
      None => {
      },
    }
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
