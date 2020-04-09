use std::io;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Read;
use std::fs::File;
use crate::common::read_word_from_array;
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
  fn read_word(&self, address: u32) -> u32 {
    read_word_from_array(&self.contents, address)
  }
  //print the first n words in the input file
  pub fn preview(&self, n: u32) {
    println!("Previewing the contents of the input CD");
    for i in 0..n {
      println!("{:#x} ", self.read_word(i * 4));
    }
  }
}

impl DMAChannel for CD {
  fn send(&mut self, data: Vec<Register>) {
  }
  fn receive(&self) -> Register {
    0
  }
}
