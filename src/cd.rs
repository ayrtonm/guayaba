use std::io;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Read;
use std::fs::File;
use crate::register::Register;
use crate::dma::DMAChannel;

pub struct CD {
  contents: Box<Vec<u8>>,
}

impl CD {
  pub fn new(filename: &String) -> io::Result<Self> {
    let mut buffer = Vec::new();
    let mut file = File::open(filename)?;
    file.seek(SeekFrom::Start(0))?;
    file.read_to_end(&mut buffer)?;
    Ok(CD {
      contents: Box::new(buffer),
    })
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
